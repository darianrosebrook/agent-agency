// ============================================================================
// Vision Image Processing Utilities
// ============================================================================

import Foundation
import CoreGraphics
import CoreImage
import CoreVideo

/// Image processing utilities for computer vision tasks
public class ImageProcessing {

    /// Convert image data to CGImage
    public static func createCGImage(from data: Data) -> CGImage? {
        guard let imageSource = CGImageSourceCreateWithData(data as CFData, nil),
              let cgImage = CGImageSourceCreateImageAtIndex(imageSource, 0, nil) else {
            return nil
        }
        return cgImage
    }

    /// Resize image maintaining aspect ratio
    public static func resizeImage(_ image: CGImage, to targetSize: CGSize) -> CGImage? {
        let imageSize = CGSize(width: image.width, height: image.height)
        let scale = min(targetSize.width / imageSize.width, targetSize.height / imageSize.height)

        let scaledSize = CGSize(
            width: imageSize.width * scale,
            height: imageSize.height * scale
        )

        let offset = CGPoint(
            x: (targetSize.width - scaledSize.width) / 2,
            y: (targetSize.height - scaledSize.height) / 2
        )

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

        context.setFillColor(CGColor.black)
        context.fill(CGRect(origin: .zero, size: targetSize))

        let drawRect = CGRect(
            x: offset.x,
            y: offset.y,
            width: scaledSize.width,
            height: scaledSize.height
        )

        context.draw(image, in: drawRect)
        return context.makeImage()
    }

    /// Convert CGImage to pixel buffer
    public static func createPixelBuffer(from image: CGImage) -> CVPixelBuffer? {
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
        defer { CVPixelBufferUnlockBaseAddress(pixelBuffer, CVPixelBufferLockFlags(rawValue: 0)) }

        guard let baseAddress = CVPixelBufferGetBaseAddress(pixelBuffer) else {
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
            return nil
        }

        let rect = CGRect(x: 0, y: 0, width: width, height: height)
        context.draw(image, in: rect)

        return pixelBuffer
    }

    /// Apply ImageNet normalization to pixel buffer
    public static func applyImageNetNormalization(to pixelBuffer: CVPixelBuffer) {
        CVPixelBufferLockBaseAddress(pixelBuffer, CVPixelBufferLockFlags(rawValue: 0))
        defer { CVPixelBufferUnlockBaseAddress(pixelBuffer, CVPixelBufferLockFlags(rawValue: 0)) }

        guard let baseAddress = CVPixelBufferGetBaseAddress(pixelBuffer) else { return }

        let width = CVPixelBufferGetWidth(pixelBuffer)
        let height = CVPixelBufferGetHeight(pixelBuffer)
        let bytesPerRow = CVPixelBufferGetBytesPerRow(pixelBuffer)

        let mean: [Float] = [0.485, 0.456, 0.406]
        let std: [Float] = [0.229, 0.224, 0.225]

        let srcPtr = baseAddress.assumingMemoryBound(to: UInt8.self)
        var floatPixels = [Float](repeating: 0, count: width * height * 3)

        for y in 0..<height {
            for x in 0..<width {
                let pixelIndex = y * bytesPerRow + x * 4
                let b = Float(srcPtr[pixelIndex]) / 255.0
                let g = Float(srcPtr[pixelIndex + 1]) / 255.0
                let r = Float(srcPtr[pixelIndex + 2]) / 255.0

                let rNorm = (r - mean[0]) / std[0]
                let gNorm = (g - mean[1]) / std[1]
                let bNorm = (b - mean[2]) / std[2]

                let outputIndex = (y * width + x) * 3
                floatPixels[outputIndex] = rNorm
                floatPixels[outputIndex + 1] = gNorm
                floatPixels[outputIndex + 2] = bNorm
            }
        }

        let destPtr = baseAddress.assumingMemoryBound(to: Float.self)
        floatPixels.withUnsafeBytes { srcBytes in
            memcpy(destPtr, srcBytes.baseAddress!, srcBytes.count)
        }
    }
}

/// Image processing errors
public enum ImageProcessingError: Error {
    case imageCreationFailed
    case resizeFailed
    case pixelBufferCreationFailed
    case processingFailed
}
