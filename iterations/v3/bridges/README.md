# Agent Bridges

**Unified Swift Bridge Architecture for Agent Agency**

A production-grade, modality-segmented SwiftPM package providing stable C FFI interfaces for machine learning models and computer vision tasks. Designed for high-performance, thread-safe integration with Rust applications.

## Architecture Overview

```
bridges/
├── Package.swift              # Unified package configuration
├── README.md                  # This documentation
├── Sources/
│   ├── Core/                  # Shared protocols and utilities
│   │   ├── BridgeProtocol.swift
│   │   ├── MemoryManagement.swift
│   │   └── ErrorHandling.swift
│   │
│   ├── Audio/                 # Speech & Audio Processing
│   │   ├── SpeechToText/
│   │   │   ├── Whisper/       # CoreML Whisper implementation
│   │   │   └── SpeechFramework/ # Apple's Speech Framework
│   │   └── AudioProcessing/   # Audio utilities
│   │
│   ├── Vision/                # 👁️ Computer Vision
│   │   ├── ObjectDetection/
│   │   │   └── YOLO/          # YOLO object detection
│   │   ├── OCR/
│   │   │   └── VisionOCR/     # Vision Framework OCR
│   │   └── ImageProcessing/   # Image utilities
│   │
│   ├── Text/                  # Text Processing
│   │   ├── LanguageModels/
│   │   │   ├── Mistral/       # Mistral LLM
│   │   │   └── Tokenization/  # Text tokenization
│   │   └── Generation/
│   │       └── Diffusion/     # Stable Diffusion
│   │
│   ├── System/                # Infrastructure
│   │   ├── ModelManagement/   # Model lifecycle & caching
│   │   └── Performance/       # Monitoring & profiling
│   │
│   └── FFI/                   # C ABI Layer
│       ├── include/
│       │   └── bridges_ffi.h  # Stable C headers
│       ├── BridgesFFI.c       # C shim implementation
│       └── BridgesFFIShim.swift # Swift FFI bridge
│
├── Tests/                     # Test suites
│   ├── AudioTests/
│   ├── VisionTests/
│   ├── TextTests/
│   └── IntegrationTests/
│
└── Examples/                  # Usage examples
```

## Key Features

### **Stable C ABI**
- Namespaced function symbols (`agentbridge_*`)
- Opaque handles instead of raw pointers
- Thread-safe handle management
- Comprehensive error propagation

### **Modular Architecture**
- Granular targets for build performance
- Feature flags for conditional compilation
- Clear separation by modality
- Easy extension for new models

### **Production Ready**
- Memory safety with autorelease pools
- Comprehensive error handling
- Performance monitoring
- Model asset management

### **Developer Experience**
- Swift protocols for consistency
- Comprehensive documentation
- Test-driven development
- CI/CD integration ready

## Usage

### Basic Setup

```swift
import AgentBridges

// Initialize the bridge system
let result = agentbridge_init()
guard result == 0 else {
    fatalError("Failed to initialize bridges")
}

// Your bridge operations here...

// Cleanup when done
agentbridge_shutdown()
```

### Text Generation with Mistral

```c
#include "bridges_ffi.h"

// Create model
ModelRef modelRef;
char* error = NULL;
int result = agentbridge_text_mistral_create("/path/to/mistral.model", &modelRef, &error);
if (result != 0) {
    fprintf(stderr, "Failed to create Mistral model: %s\n", error);
    agentbridge_free_string(error);
    return 1;
}

// Generate text
char* generatedText = NULL;
result = agentbridge_text_mistral_generate(
    modelRef,
    "Hello, how are you?",
    100,  // maxTokens
    0.7,  // temperature
    &generatedText,
    &error
);

if (result == 0) {
    printf("Generated: %s\n", generatedText);
    agentbridge_free_string(generatedText);
} else {
    fprintf(stderr, "Generation failed: %s\n", error);
    agentbridge_free_string(error);
}

// Cleanup
agentbridge_model_destroy(modelRef);
```

### Audio Transcription with Whisper

```c
// Create Whisper model
ModelRef whisperRef;
result = agentbridge_audio_whisper_create(
    "/path/to/whisper/models",
    "base",  // model size
    &whisperRef,
    &error
);

// Transcribe audio
char* transcript = NULL;
char* segmentsJson = NULL;
float confidence = 0.0;

result = agentbridge_audio_whisper_transcribe(
    whisperRef,
    "/path/to/audio.wav",
    "en",  // language (NULL for auto-detect)
    &transcript,
    &segmentsJson,
    &confidence,
    &error
);

if (result == 0) {
    printf("Transcript: %s (confidence: %.2f)\n", transcript, confidence);
    // Parse segmentsJson for detailed timing information
}

// Cleanup
agentbridge_free_string(transcript);
agentbridge_free_string(segmentsJson);
agentbridge_model_destroy(whisperRef);
```

### Object Detection with YOLO

```c
// Load image data
NSData* imageData = [NSData dataWithContentsOfFile:@"/path/to/image.jpg"];

// Create YOLO model
ModelRef yoloRef;
result = agentbridge_vision_yolo_create("/path/to/yolo.model", &yoloRef, &error);

// Detect objects
char* detectionsJson = NULL;
int32_t detectionCount = 0;

result = agentbridge_vision_yolo_detect(
    yoloRef,
    (const uint8_t*)imageData.bytes,
    (int32_t)imageData.length,
    0.5,  // confidence threshold
    &detectionsJson,
    &detectionCount,
    &error
);

if (result == 0) {
    printf("Found %d detections\n", detectionCount);
    // Parse detectionsJson for bounding boxes and labels
}

// Cleanup
agentbridge_free_string(detectionsJson);
agentbridge_model_destroy(yoloRef);
```

## Building

### SwiftPM Build

```bash
# Build all targets
swift build

# Build with specific features
swift build -Xswiftc -DENABLE_AUDIO_BRIDGES -Xswiftc -DENABLE_VISION_BRIDGES

# Build release
swift build -c release
```

### Integration with Rust

Add to your `Cargo.toml`:

```toml
[dependencies]
agent-agency-apple-silicon = { path = "../apple-silicon" }

[build-dependencies]
cc = "1.0"
```

In your `build.rs`:

```rust
fn main() {
    // Link to Swift bridge library
    println!("cargo:rustc-link-lib=dylib=AgentBridges");
    println!("cargo:rustc-link-search=native=/path/to/swift/build/products");

    // Generate FFI bindings if needed
    // bindgen::Builder::default()
    //     .header("bridges/Sources/FFI/include/bridges_ffi.h")
    //     .generate()
    //     .expect("Unable to generate bindings")
    //     .write_to_file("src/bridges.rs")
    //     .expect("Couldn't write bindings!");
}
```

## Testing

### Unit Tests

```bash
# Run all tests
swift test

# Run specific test suites
swift test --filter AudioTests
swift test --filter VisionTests
swift test --filter TextTests
```

### Integration Tests

```bash
# Run FFI integration tests
swift test --filter IntegrationTests

# Test with Rust integration
cargo test --package agent-agency-council integration_tests
```

### Performance Testing

```bash
# Profile specific operations
swift test --filter PerformanceTests

# Memory leak detection
swift test --filter MemoryLeakTests
```

## Architecture Decisions

### Thread Safety
- **Opaque Handles**: All model references are `uint64_t` IDs managed by thread-safe registries
- **No Raw Pointers**: Never expose `*mut c_void` across FFI boundaries
- **Queue Confinement**: CoreML handles are confined to dedicated queues

### Memory Management
- **Autorelease Pools**: All FFI operations wrapped in `@autoreleasepool`
- **RAII Handles**: Automatic cleanup when handles go out of scope
- **Leak Detection**: Built-in memory tracking and leak tests

### Error Handling
- **Structured Errors**: Consistent error types with severity levels
- **Error Propagation**: Errors bubble up through FFI with detailed messages
- **Logging Integration**: Comprehensive error logging and monitoring

### Build Performance
- **Granular Targets**: Fine-grained dependencies reduce rebuilds
- **Conditional Compilation**: Feature flags enable/disable modalities
- **Incremental Builds**: Only rebuild affected targets

## Contributing

### Adding New Bridges

1. **Choose Modality**: Audio, Vision, Text, or System
2. **Create Target Structure**:
   ```
   Sources/[Modality]/[Domain]/[Implementation]/
   ├── Bridge.swift
   ├── Tests/
   └── Examples/
   ```
3. **Implement Protocols**: Conform to `BridgeProtocol`
4. **Add FFI Interface**: Extend `bridges_ffi.h` and `BridgesFFIShim.swift`
5. **Add Tests**: Unit tests, integration tests, performance tests
6. **Update Package.swift**: Add new target and dependencies

### Code Standards

- **Swift 5.10+**: Use latest language features
- **Swift Concurrency**: Prefer async/await over completion handlers
- **Error Handling**: Use structured errors, not NSError
- **Memory Safety**: ARC-compliant, no retain cycles
- **Thread Safety**: Document thread requirements for all APIs

## Performance Characteristics

### Latency Targets
- **Text Generation**: <500ms for 50 tokens (Mistral-7B)
- **Audio Transcription**: <200ms per second of audio
- **Object Detection**: <100ms per image (416x416)
- **OCR**: <300ms per image

### Memory Usage
- **Base Overhead**: ~50MB for bridge system
- **Per Model**: 100MB-4GB depending on model size
- **Peak Memory**: GPU/ANE memory during inference

### Throughput
- **Concurrent Requests**: 4-8 simultaneous operations
- **Queue Depth**: Configurable based on hardware
- **Resource Pooling**: Automatic model instance reuse

## Troubleshooting

### Common Issues

**"Undefined symbols" during linking:**
- Ensure all targets are built and linked
- Check feature flags match between Swift and Rust builds

**Memory leaks:**
- Use Instruments to profile memory usage
- Check autorelease pool scoping
- Verify handle cleanup

**Threading issues:**
- Ensure FFI calls are made from appropriate threads
- Check handle registry thread safety

**Model loading failures:**
- Verify model paths and permissions
- Check CoreML model compatibility
- Review model format (mlmodel vs mlmodelc)

### Debug Mode

Enable debug logging:

```swift
// In your bridge code
import os.log

let logger = Logger(subsystem: "com.agent.bridges", category: "debug")
logger.debug("Operation started with parameters: \(params)")
```

## License

This package is part of the Agent Agency project. See project LICENSE file for details.

## Support

- **Issues**: File GitHub issues with `[bridges]` prefix
- **Discussions**: Use GitHub Discussions for questions
- **Documentation**: See inline code documentation and examples

---

**Built for performance, safety, and maintainability in production ML systems.**
