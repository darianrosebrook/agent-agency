// ============================================================================
// FastViT Bridge - Image Classification
// ============================================================================

import Foundation
import CoreML
import CoreGraphics
import Accelerate
@_exported import Core
@_exported import System_ModelMgmt

/// FastViT image classification bridge conforming to BridgeProtocol
public class FastViTBridge: BridgeProtocol {
    public let identifier = "FastViTClassification"
    public let version = "1.0.0"
    public let capabilities: Set<String> = [
        "image_classification",
        "vision_analysis",
        "top_k_predictions",
        "imagenet_classes"
    ]

    private var model: MLModel?
    private var modelURL: URL?
    private let queue = DispatchQueue(label: "com.agent.fastvit", attributes: .concurrent)

    /// FastViT T8 input size (224x224)
    private let inputSize = CGSize(width: 224, height: 224)

    public init() {
        // Initialize without model - lazy loading
    }

    public init(modelPath: String) throws {
        // Initialize with specific model path
        let url = URL(fileURLWithPath: modelPath)
        try loadModel(from: url)
    }

    public func initialize(config: BridgeConfig) -> BridgeResult<Void> {
        // Model loading happens on first classification request
        return .success(())
    }

    public func shutdown() -> BridgeResult<Void> {
        queue.sync(flags: .barrier) {
            self.model = nil
            self.modelURL = nil
        }
        return .success(())
    }

    public func healthCheck() -> BridgeResult<BridgeHealth> {
        return queue.sync {
            let isHealthy = model != nil
            return .success(BridgeHealth(
                status: isHealthy ? .healthy : .degraded,
                message: isHealthy ? "FastViT model loaded" : "Model not loaded",
                uptimeSeconds: 0
            ))
        }
    }

    public func getMetrics() -> BridgeResult<BridgeMetrics> {
        // Basic metrics - could be expanded with actual classification stats
        return .success(BridgeMetrics())
    }

    // MARK: - Classification Operations

    /// Classify image data and return top-K predictions
    public func classify(
        imageData: Data,
        topK: Int = 5,
        options: ClassificationOptions = ClassificationOptions()
    ) async throws -> ClassificationResult {
        try await ensureModelLoaded()

        return try queue.sync {
            // Preprocess image
            guard let preprocessedImage = preprocessImage(imageData, targetSize: inputSize) else {
                throw BridgeError.processingFailed("Failed to preprocess image")
            }

            // Create model input
            let inputFeatures: [String: MLFeatureValue] = [
                "image": MLFeatureValue(multiArray: preprocessedImage)
            ]

            let input = try MLDictionaryFeatureProvider(dictionary: inputFeatures)

            // Run inference
            let output = try self.model!.prediction(from: input)

            // Decode predictions
            let predictions = try decodePredictions(
                from: output,
                topK: topK,
                options: options
            )

            return ClassificationResult(
                predictions: predictions,
                topK: topK,
                confidenceThreshold: options.confidenceThreshold
            )
        }
    }

    /// Classify CGImage and return top-K predictions
    public func classify(
        image: CGImage,
        topK: Int = 5,
        options: ClassificationOptions = ClassificationOptions()
    ) async throws -> ClassificationResult {
        try await ensureModelLoaded()

        return try queue.sync {
            guard let preprocessedImage = preprocessCGImage(image, targetSize: inputSize) else {
                throw BridgeError.processingFailed("Failed to preprocess image")
            }

            // Create model input
            let inputFeatures: [String: MLFeatureValue] = [
                "image": MLFeatureValue(multiArray: preprocessedImage)
            ]

            let input = try MLDictionaryFeatureProvider(dictionary: inputFeatures)

            // Run inference
            let output = try self.model!.prediction(from: input)

            // Decode predictions
            let predictions = try decodePredictions(
                from: output,
                topK: topK,
                options: options
            )

            return ClassificationResult(
                predictions: predictions,
                topK: topK,
                confidenceThreshold: options.confidenceThreshold
            )
        }
    }

    /// Get supported ImageNet classes
    public func getSupportedClasses() -> [String] {
        return ImageNetMetadata.imagenetClasses
    }

    /// Get class name for label index
    public func getClassName(for label: Int) -> String {
        guard label >= 0 && label < ImageNetMetadata.imagenetClasses.count else {
            return "unknown"
        }
        return ImageNetMetadata.imagenetClasses[label]
    }

    // MARK: - Private Implementation

    private func ensureModelLoaded() async throws {
        if model != nil { return }

        // Try to load model from cache first
        if let asset = try globalModelManager?.getCachedModel(identifier: "fastvit-t8", channel: .stable) {
            try loadModel(from: asset.localURL)
        } else {
            // Download model if not cached
            let asset = try await globalModelManager!.downloadModel(identifier: "fastvit-t8", channel: .stable)
            try loadModel(from: asset.localURL)
        }
    }

    private func loadModel(from url: URL) throws {
        let config = MLModelConfiguration()
        config.computeUnits = .all  // Use ANE + GPU + CPU

        model = try MLModel(contentsOf: url, configuration: config)
        modelURL = url
    }

    // MARK: - Image Preprocessing

    private func preprocessImage(_ imageData: Data, targetSize: CGSize) -> MLMultiArray? {
        // Create CGImage from data
        guard let imageSource = CGImageSourceCreateWithData(imageData as CFData, nil),
              let cgImage = CGImageSourceCreateImageAtIndex(imageSource, 0, nil) else {
            return nil
        }

        return preprocessCGImage(cgImage, targetSize: targetSize)
    }

    private func preprocessCGImage(_ cgImage: CGImage, targetSize: CGSize) -> MLMultiArray? {
        // Step 1: Resize image maintaining aspect ratio
        guard let resizedImage = resizeImage(cgImage, to: targetSize) else { return nil }

        // Step 2: Convert to RGB pixel buffer
        guard let pixelBuffer = createRGBPixelBuffer(from: resizedImage) else { return nil }

        // Step 3: Apply ImageNet normalization
        applyImageNetNormalization(to: pixelBuffer)

        // Step 4: Convert to MLMultiArray (CHW format for PyTorch models)
        return createMLMultiArray(from: pixelBuffer)
    }

    private func resizeImage(_ image: CGImage, to targetSize: CGSize) -> CGImage? {
        let imageSize = CGSize(width: image.width, height: image.height)
        let scale = min(targetSize.width / imageSize.width, targetSize.height / imageSize.height)

        let scaledSize = CGSize(
            width: imageSize.width * scale,
            height: imageSize.height * scale
        )

        // Center the scaled image in the target area
        let offset = CGPoint(
            x: (targetSize.width - scaledSize.width) / 2,
            y: (targetSize.height - scaledSize.height) / 2
        )

        // Create context for letterboxing
        let colorSpace = CGColorSpaceCreateDeviceRGB()
        let bitmapInfo = CGImageAlphaInfo.premultipliedLast.rawValue | CGBitmapInfo.byteOrder32Big.rawValue

        guard let context = CGContext(
            data: nil,
            width: Int(targetSize.width),
            height: Int(targetSize.height),
            bitsPerComponent: 8,
            bytesPerRow: Int(targetSize.width) * 4,
            space: colorSpace,
            bitmapInfo: bitmapInfo
        ) else {
            return nil
        }

        // Fill with black background (letterboxing)
        context.setFillColor(CGColor.black)
        context.fill(CGRect(origin: .zero, size: targetSize))

        // Draw scaled image centered
        let drawRect = CGRect(
            x: offset.x,
            y: offset.y,
            width: scaledSize.width,
            height: scaledSize.height
        )

        context.draw(image, in: drawRect)

        return context.makeImage()
    }

    private func createRGBPixelBuffer(from image: CGImage) -> CVPixelBuffer? {
        let width = image.width
        let height = image.height

        var pixelBuffer: CVPixelBuffer?

        let status = CVPixelBufferCreate(
            kCFAllocatorDefault,
            width,
            height,
            kCVPixelFormatType_32BGRA,
            nil,
            &pixelBuffer
        )

        guard status == kCVReturnSuccess, let pixelBuffer = pixelBuffer else {
            return nil
        }

        CVPixelBufferLockBaseAddress(pixelBuffer, CVPixelBufferLockFlags(rawValue: 0))

        guard let baseAddress = CVPixelBufferGetBaseAddress(pixelBuffer) else {
            CVPixelBufferUnlockBaseAddress(pixelBuffer, CVPixelBufferLockFlags(rawValue: 0))
            return nil
        }

        let colorSpace = CGColorSpaceCreateDeviceRGB()
        let bitmapInfo = CGImageAlphaInfo.premultipliedLast.rawValue | CGBitmapInfo.byteOrder32Big.rawValue

        guard let context = CGContext(
            data: baseAddress,
            width: width,
            height: height,
            bitsPerComponent: 8,
            bytesPerRow: CVPixelBufferGetBytesPerRow(pixelBuffer),
            space: colorSpace,
            bitmapInfo: bitmapInfo
        ) else {
            CVPixelBufferUnlockBaseAddress(pixelBuffer, CVPixelBufferLockFlags(rawValue: 0))
            return nil
        }

        let rect = CGRect(x: 0, y: 0, width: width, height: height)
        context.draw(image, in: rect)

        CVPixelBufferUnlockBaseAddress(pixelBuffer, CVPixelBufferLockFlags(rawValue: 0))

        return pixelBuffer
    }

    private func applyImageNetNormalization(to pixelBuffer: CVPixelBuffer) {
        CVPixelBufferLockBaseAddress(pixelBuffer, CVPixelBufferLockFlags(rawValue: 0))
        defer { CVPixelBufferUnlockBaseAddress(pixelBuffer, CVPixelBufferLockFlags(rawValue: 0)) }

        guard let baseAddress = CVPixelBufferGetBaseAddress(pixelBuffer) else { return }

        let width = CVPixelBufferGetWidth(pixelBuffer)
        let height = CVPixelBufferGetHeight(pixelBuffer)
        let bytesPerRow = CVPixelBufferGetBytesPerRow(pixelBuffer)

        // ImageNet mean and std values (RGB)
        let mean: [Float] = [0.485, 0.456, 0.406]
        let std: [Float] = [0.229, 0.224, 0.225]

        // Process pixels using Accelerate framework for performance
        let srcPtr = baseAddress.assumingMemoryBound(to: UInt8.self)
        var floatPixels = [Float](repeating: 0, count: width * height * 3)

        // Convert BGRA to RGB and to float, then normalize
        for y in 0..<height {
            for x in 0..<width {
                let pixelIndex = y * bytesPerRow + x * 4
                let b = Float(srcPtr[pixelIndex]) / 255.0
                let g = Float(srcPtr[pixelIndex + 1]) / 255.0
                let r = Float(srcPtr[pixelIndex + 2]) / 255.0
                // Skip alpha channel

                // Normalize using ImageNet statistics
                let rNorm = (r - mean[0]) / std[0]
                let gNorm = (g - mean[1]) / std[1]
                let bNorm = (b - mean[2]) / std[2]

                let outputIndex = (y * width + x) * 3
                floatPixels[outputIndex] = rNorm
                floatPixels[outputIndex + 1] = gNorm
                floatPixels[outputIndex + 2] = bNorm
            }
        }

        // Copy normalized values back to pixel buffer
        let destPtr = baseAddress.assumingMemoryBound(to: Float.self)
        let _ = floatPixels.withUnsafeBytes { srcBytes in
            memcpy(destPtr, srcBytes.baseAddress!, srcBytes.count)
        }
    }

    private func createMLMultiArray(from pixelBuffer: CVPixelBuffer) -> MLMultiArray? {
        CVPixelBufferLockBaseAddress(pixelBuffer, CVPixelBufferLockFlags(rawValue: 0))
        defer { CVPixelBufferUnlockBaseAddress(pixelBuffer, CVPixelBufferLockFlags(rawValue: 0)) }

        let width = CVPixelBufferGetWidth(pixelBuffer)
        let height = CVPixelBufferGetHeight(pixelBuffer)

        // Create MLMultiArray with shape [1, 3, height, width] (NCHW format)
        let shape: [NSNumber] = [1, 3, NSNumber(value: height), NSNumber(value: width)]

        guard let mlArray = try? MLMultiArray(shape: shape, dataType: .float32) else {
            return nil
        }

        guard let baseAddress = CVPixelBufferGetBaseAddress(pixelBuffer) else {
            return nil
        }

        let srcPtr = baseAddress.assumingMemoryBound(to: Float.self)

        // Convert HWC to CHW format (3, height, width)
        for c in 0..<3 { // RGB channels
            for h in 0..<height {
                for w in 0..<width {
                    let srcIndex = (h * width + w) * 3 + c
                    let dstIndex = c * height * width + h * width + w
                    mlArray[dstIndex] = NSNumber(value: srcPtr[srcIndex])
                }
            }
        }

        return mlArray
    }

    // MARK: - Prediction Decoding

    private func decodePredictions(
        from output: MLFeatureProvider,
        topK: Int,
        options: ClassificationOptions
    ) throws -> [Prediction] {
        guard let outputArray = output.featureValue(for: "classLabelProbs")?.multiArrayValue else {
            throw BridgeError.invalidModelOutput("Missing classLabelProbs feature")
        }

        // Convert MLMultiArray to array of (index, probability) tuples
        var predictions: [(Int, Float)] = []
        
        for i in 0..<outputArray.count {
            let probability = outputArray[i].floatValue
            if probability >= options.confidenceThreshold {
                predictions.append((i, probability))
            }
        }

        // Sort by probability (descending) and take top-K
        let topPredictions = predictions
            .sorted { $0.1 > $1.1 }
            .prefix(topK)

        return topPredictions.map { (index, probability) in
            Prediction(
                label: index,
                className: getClassName(for: index),
                confidence: probability
            )
        }
    }
}

// MARK: - Supporting Types

/// Classification options
public struct ClassificationOptions {
    public let confidenceThreshold: Float

    public init(confidenceThreshold: Float = 0.01) {
        self.confidenceThreshold = confidenceThreshold
    }
}

/// Classification result
public struct ClassificationResult: Codable {
    public let predictions: [Prediction]
    public let topK: Int
    public let confidenceThreshold: Float
}

/// Individual prediction
public struct Prediction: Codable {
    public let label: Int
    public let className: String
    public let confidence: Float
}

/// ImageNet metadata
private enum ImageNetMetadata {
    static let imagenetClasses = [
        "tench", "goldfish", "great white shark", "tiger shark", "hammerhead", "electric ray",
        "stingray", "cock", "hen", "ostrich", "brambling", "goldfinch", "house finch", "junco",
        "indigo bunting", "robin", "bulbul", "jay", "magpie", "chickadee", "water ouzel", "kite",
        "bald eagle", "vulture", "great grey owl", "european fire salamander", "common newt",
        "eft", "spotted salamander", "axolotl", "bullfrog", "tree frog", "tailed frog",
        "loggerhead", "leatherback turtle", "mud turtle", "terrapin", "box turtle", "banded gecko",
        "common iguana", "american chameleon", "whiptail", "agama", "frilled lizard", "alligator lizard",
        "gila monster", "green lizard", "african chameleon", "komodo dragon", "african crocodile",
        "american alligator", "triceratops", "thunder snake", "ringneck snake", "hognose snake",
        "green snake", "king snake", "garter snake", "water snake", "vine snake", "night snake",
        "boa constrictor", "rock python", "indian cobra", "green mamba", "sea snake", "horned viper",
        "diamondback", "sidewinder", "trilobite", "harvestman", "scorpion", "black and gold garden spider",
        "barn spider", "garden spider", "black widow", "tarantula", "wolf spider", "tick", "centipede",
        "black grouse", "ptarmigan", "ruffed grouse", "prairie chicken", "peacock", "quail", "partridge",
        "african grey", "macaw", "sulphur-crested cockatoo", "lorikeet", "coucal", "bee eater",
        "hornbill", "hummingbird", "jacamar", "toucan", "drake", "red-breasted merganser", "goose",
        "black swan", "tusker", "echidna", "platypus", "wallaby", "koala", "wombat", "jellyfish",
        "sea anemone", "brain coral", "flatworm", "nematode", "conch", "snail", "slug", "sea slug",
        "chiton", "chambered nautilus", "Dungeness crab", "rock crab", "fiddler crab", "king crab",
        "american lobster", "spiny lobster", "crayfish", "hermit crab", "isopod", "white stork",
        "black stork", "spoonbill", "flamingo", "little blue heron", "american egret", "bittern",
        "crane", "limpkin", "european gallinule", "american coot", "bustard", "ruddy turnstone",
        "red-backed sandpiper", "redshank", "dowitcher", "oystercatcher", "pelican", "king penguin",
        "albatross", "grey whale", "killer whale", "dugong", "sea lion", "chihuahua", "japanese spaniel",
        "maltese dog", "pekinese", "shih-tzu", "blenheim spaniel", "papillon", "toy terrier",
        "rhodesian ridgeback", "afghan hound", "basset", "beagle", "bloodhound", "bluetick",
        "black-and-tan coonhound", "walker hound", "english foxhound", "redbone", "borzoi",
        "irish wolfhound", "italian greyhound", "whippet", "ibizan hound", "norwegian elkhound",
        "otterhound", "saluki", "scottish deerhound", "weimaraner", "staffordshire bullterrier",
        "american staffordshire terrier", "bedlington terrier", "border terrier", "kerry blue terrier",
        "irish terrier", "norfolk terrier", "norwich terrier", "yorkshire terrier", "wire-haired fox terrier",
        "lakeland terrier", "sealyham terrier", "airedale", "cairn", "australian terrier", "dandie dinmont",
        "boston bull", "miniature schnauzer", "giant schnauzer", "standard schnauzer", "scotch terrier",
        "tibetan terrier", "silky terrier", "soft-coated wheaten terrier", "west highland white terrier",
        "lhasa", "flat-coated retriever", "curly-coated retriever", "golden retriever", "labrador retriever",
        "chesapeake bay retriever", "german short-haired pointer", "vizsla", "english setter",
        "irish setter", "gordon setter", "brittany spaniel", "clumber", "english springer", "welsh springer spaniel",
        "cocker spaniel", "sussex spaniel", "irish water spaniel", "kuvasz", "schipperke", "groenendael",
        "malinois", "briard", "kelpie", "komondor", "old english sheepdog", "shetland sheepdog", "collie",
        "border collie", "bouvier des flandres", "rottweiler", "german shepherd", "doberman", "miniature pinscher",
        "greater swiss mountain dog", "bernese mountain dog", "appenzeller", "entlebucher", "boxer",
        "bull mastiff", "tibetan mastiff", "french bulldog", "great dane", "saint bernard", "eskimo dog",
        "malamute", "siberian husky", "affenpinscher", "basenji", "pug", "leonberg", "newfoundland",
        "great pyrenees", "samoyed", "pomeranian", "chow", "keeshond", "brabancon griffon", "pembroke",
        "cardigan", "toy poodle", "miniature poodle", "standard poodle", "mexican hairless", "timber wolf",
        "white wolf", "red wolf", "coyote", "dingo", "dhole", "african hunting dog", "hyena", "red fox",
        "kit fox", "arctic fox", "grey fox", "tabby", "tiger cat", "persian cat", "siamese cat", "egyptian cat",
        "lion", "tiger", "jaguar", "leopard", "snow leopard", "lynx", "bobcat", "clouded leopard", "margay",
        "ocelot", "serval", "caracal", "raccoon", "ringtail", "coati", "kinkajou", "civet", "binturong",
        "mongoose", "meerkat", "tiger beetle", "ladybug", "ground beetle", "long-horned beetle", "leaf beetle",
        "dung beetle", "rhinoceros beetle", "weevil", "fly", "bee", "ant", "grasshopper", "cricket",
        "walking stick", "cockroach", "mantis", "cicada", "leafhopper", "lacewing", "dragonfly", "damselfly",
        "admiral", "ringlet", "monarch", "cabbage butterfly", "sulphur butterfly", "lycaenid", "starfish",
        "sea urchin", "sea cucumber", "wood rabbit", "hare", "angora", "hamster", "porcupine", "fox squirrel",
        "marmot", "beaver", "guinea pig", "sorrel", "zebra", "hog", "wild boar", "warthog", "hippopotamus",
        "ox", "water buffalo", "bison", "ram", "bighorn", "ibex", "hartebeest", "impala", "gazelle",
        "arabian camel", "llama", "weasel", "mink", "polecat", "black-footed ferret", "otter", "skunk",
        "badger", "armadillo", "three-toed sloth", "orangutan", "gorilla", "chimpanzee", "gibbon",
        "siamang", "guenon", "patas", "baboon", "macaque", "langur", "colobus", "proboscis monkey",
        "marmoset", "capuchin", "howler monkey", "titi", "spider monkey", "squirrel monkey", "madagascar cat",
        "indri", "indian elephant", "african elephant", "lesser panda", "giant panda", "barracouta",
        "eel", "coho", "rock beauty", "anemone fish", "sturgeon", "gar", "lionfish", "puffer", "abacus",
        "abaya", "academic gown", "accordion", "acoustic guitar", "aircraft carrier", "airliner", "airship",
        "altar", "ambulance", "amphibian", "analog clock", "apiary", "apron", "ashcan", "assault rifle",
        "backpack", "bakery", "balance beam", "balloon", "ballpoint", "band aid", "banjo", "bannister",
        "barbell", "barber chair", "barbershop", "barn", "barometer", "barrel", "barrow", "baseball",
        "basketball", "bassinet", "bassoon", "bathing cap", "bath towel", "bathtub", "beach wagon",
        "beacon", "beaker", "bearskin", "beer bottle", "beer glass", "bell cote", "bib", "bicycle-built-for-two",
        "bikini", "binder", "binoculars", "birdhouse", "boathouse", "bobsled", "bolo tie", "bonnet",
        "bookcase", "bookshop", "bottlecap", "bow", "bow tie", "brass", "brassiere", "breakwater",
        "breastplate", "broom", "bucket", "buckle", "bulletproof vest", "bullet train", "butcher shop",
        "cab", "caldron", "candle", "cannon", "canoe", "can opener", "cardigan", "car mirror", "carousel",
        "carpenter's kit", "carton", "car wheel", "cash machine", "cassette", "cassette player", "castle",
        "catamaran", "CD player", "cello", "cellular telephone", "chain", "chainlink fence", "chain mail",
        "chain saw", "chest", "chiffonier", "chime", "china cabinet", "christmas stocking", "church",
        "cinema", "cleaver", "cliff dwelling", "cloak", "clog", "cocktail shaker", "coffee mug", "coffeepot",
        "coil", "combination lock", "computer keyboard", "confectionery", "container ship", "convertible",
        "corkscrew", "cornet", "cowboy boot", "cowboy hat", "cradle", "crane", "crash helmet", "crate",
        "crib", "Crock Pot", "croquet ball", "crutch", "cuirass", "dam", "desk", "desktop computer",
        "dial telephone", "diaper", "digital clock", "digital watch", "dining table", "dishrag",
        "dishwasher", "disk brake", "dock", "dogsled", "dome", "doormat", "drilling platform", "drum",
        "drumstick", "dumbbell", "dutch oven", "electric fan", "electric guitar", "electric locomotive",
        "entertainment center", "envelope", "espresso maker", "face powder", "feather boa", "file",
        "fireboat", "fire engine", "fire screen", "flagpole", "flute", "folding chair", "football helmet",
        "forklift", "fountain", "fountain pen", "four-poster", "freight car", "french horn", "frying pan",
        "fur coat", "garbage truck", "gasmask", "gas pump", "goblet", "go-kart", "golf ball", "golfcart",
        "gondola", "gong", "gown", "grand piano", "greenhouse", "grille", "grocery store", "guillotine",
        "hair slide", "hair spray", "half track", "hammer", "hamper", "hand blower", "hand-held computer",
        "handkerchief", "hard disc", "harmonica", "harp", "harvester", "hatchet", "holster", "home theater",
        "honeycomb", "hook", "hoopskirt", "horizontal bar", "horse cart", "hourglass", "iPod", "iron",
        "jack-o'-lantern", "jean", "jeep", "jersey", "jigsaw puzzle", "jinrikisha", "joystick", "kimono",
        "knee pad", "knot", "lab coat", "ladle", "lampshade", "laptop", "lawn mower", "lens cap", "letter opener",
        "library", "lifeboat", "lighter", "limousine", "liner", "lipstick", "loafer", "lotion", "loudspeaker",
        "loupe", "lumbermill", "magnetic compass", "mailbag", "mailbox", "maillot", "manhole cover", "maraca",
        "marimba", "mask", "matchstick", "maypole", "maze", "measuring cup", "medicine chest", "megalith",
        "microphone", "microwave", "military uniform", "milk can", "minibus", "miniskirt", "minivan",
        "missile", "mitten", "mixing bowl", "mobile home", "Model T", "modem", "monastery", "monitor",
        "moped", "mortar", "mortarboard", "mosque", "mosquito net", "motor scooter", "mountain bike",
        "mountain tent", "mouse", "mousetrap", "moving van", "muzzle", "nail", "neck brace", "necklace",
        "nipple", "notebook", "obelisk", "oboe", "ocarina", "odometer", "oil filter", "organ", "oscilloscope",
        "overskirt", "oxcart", "oxygen mask", "packet", "paddle", "paddlewheel", "padlock", "paintbrush",
        "pajama", "palace", "panpipe", "paper towel", "parachute", "parallel bars", "park bench", "parking meter",
        "passenger car", "patio", "pay-phone", "pedestal", "pencil box", "pencil sharpener", "perfume",
        "petri dish", "photocopier", "pick", "pickelhaube", "picket fence", "pickup", "pier", "piggy bank",
        "pill bottle", "pillow", "ping-pong ball", "pinwheel", "pirate", "pitcher", "plane", "planetarium",
        "plastic bag", "plate rack", "plow", "plunger", "Polaroid camera", "pole", "police van", "poncho",
        "pool table", "pop bottle", "pot", "potter's wheel", "power drill", "prayer rug", "printer",
        "prison", "projectile", "projector", "puck", "punching bag", "purse", "quill", "quilt", "racer",
        "racket", "radiator", "radio", "radio telescope", "rain barrel", "recreational vehicle", "reel",
        "reflex camera", "refrigerator", "remote control", "restaurant", "revolver", "rifle", "rocking chair",
        "rotisserie", "rubber eraser", "rugby ball", "rule", "running shoe", "safe", "safety pin", "saltshaker",
        "sandal", "sarong", "sax", "scabbard", "scale", "school bus", "schooner", "scoreboard", "screen",
        "screw", "screwdriver", "seat belt", "sewing machine", "shield", "shoe shop", "shoji", "shopping basket",
        "shopping cart", "shovel", "shower cap", "shower curtain", "ski", "ski mask", "sleeping bag", "slide rule",
        "sliding door", "slot", "snorkel", "snowmobile", "snowplow", "soap dispenser", "soccer ball", "sock",
        "solar dish", "sombrero", "soup bowl", "space bar", "space heater", "space shuttle", "spatula",
        "speedboat", "spider web", "spindle", "sports car", "spotlight", "stage", "steam locomotive", "steel arch bridge",
        "steel drum", "stethoscope", "stole", "stone wall", "stopwatch", "stove", "strainer", "streetcar",
        "stretcher", "studio couch", "stupa", "submarine", "suit", "sundial", "sunglass", "sunglasses",
        "sunscreen", "suspension bridge", "swab", "sweatshirt", "swimming trunks", "swing", "switch", "syringe",
        "table lamp", "tank", "tape player", "teapot", "teddy", "television", "tennis ball", "thatch",
        "theater curtain", "thimble", "thresher", "throne", "tile roof", "toaster", "tobacco shop", "toilet seat",
        "torch", "totem pole", "tow truck", "toyshop", "tractor", "trailer truck", "tray", "trench coat",
        "tricycle", "trimaran", "tripod", "triumphal arch", "trolleybus", "trombone", "tub", "turnstile",
        "typewriter keyboard", "umbrella", "unicycle", "upright", "vacuum", "vase", "vault", "velvet",
        "vending machine", "vestment", "viaduct", "violin", "volleyball", "waffle iron", "wall clock",
        "wallet", "wardrobe", "warplane", "washbasin", "washer", "water bottle", "water jug", "water tower",
        "whiskey jug", "whistle", "wig", "window screen", "window shade", "windsor tie", "wine bottle",
        "wing", "wok", "wooden spoon", "wool", "worm fence", "wreck", "yawl", "yurt", "web site", "comic book",
        "crossword puzzle", "street sign", "traffic light", "book jacket", "menu", "plate", "guacamole",
        "consomme", "hot pot", "trifle", "ice cream", "ice lolly", "french loaf", "bagel", "pretzel",
        "cheeseburger", "hotdog", "mashed potato", "head cabbage", "broccoli", "cauliflower", "zucchini",
        "spaghetti squash", "acorn squash", "butternut squash", "cucumber", "artichoke", "bell pepper",
        "cardoon", "mushroom", "Granny Smith", "strawberry", "orange", "lemon", "fig", "pineapple",
        "banana", "jackfruit", "custard apple", "pomegranate", "hay", "carbonara", "chocolate sauce",
        "dough", "meat loaf", "pizza", "potpie", "burrito", "red wine", "espresso", "cup", "eggnog",
        "alp", "bubble", "cliff", "coral reef", "geyser", "lakeside", "promontory", "sandbar", "seashore",
        "valley", "volcano", "ballplayer", "groom", "scuba diver", "rapeseed", "daisy", "yellow lady's slipper",
        "corn", "acorn", "hip", "buckeye", "coral fungus", "agaric", "gyromitra", "stinkhorn", "earthstar",
        "hen-of-the-woods", "bolete", "ear", "toilet tissue"
    ]
}

// MARK: - Global Bridge Registration

// Register this bridge globally
private let _registration: Void = {
    globalBridgeRegistry.register(FastViTBridge())
    return ()
}()

// MARK: - Global Model Manager Access

private var globalModelManager: ModelManager?

private func getModelManager() throws -> ModelManager {
    if let manager = globalModelManager {
        return manager
    }

    let manager = try ModelManager()
    globalModelManager = manager
    return manager
}
