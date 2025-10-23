// ============================================================================
// Memory Management - Safe Memory Handling for FFI
// ============================================================================

import Foundation

/// Thread-safe memory management utilities for FFI operations
public class MemoryManager {

    /// Allocate and return a C string copy of a Swift string
    /// - Parameter string: Swift string to copy
    /// - Returns: C string pointer (caller must free with freeString)
    public static func createCString(from string: String) -> UnsafeMutablePointer<CChar> {
        return strdup(string)
    }

    /// Free a C string allocated by createCString
    /// - Parameter ptr: C string pointer to free
    public static func freeString(_ ptr: UnsafeMutablePointer<CChar>?) {
        guard let ptr = ptr else { return }
        free(ptr)
    }

    /// Allocate and return a C array of int32 values
    /// - Parameter array: Swift array to copy
    /// - Returns: C array pointer (caller must free with freeInt32Array)
    public static func createInt32Array(from array: [Int32]) -> UnsafeMutablePointer<Int32> {
        let buffer = UnsafeMutablePointer<Int32>.allocate(capacity: array.count)
        buffer.initialize(from: array, count: array.count)
        return buffer
    }

    /// Free a C int32 array allocated by createInt32Array
    /// - Parameters:
    ///   - ptr: Array pointer to free
    ///   - count: Number of elements (for validation)
    public static func freeInt32Array(_ ptr: UnsafeMutablePointer<Int32>?, count: Int) {
        guard let ptr = ptr else { return }
        ptr.deinitialize(count: count)
        ptr.deallocate()
    }

    /// Allocate and return a C array of uint8 values
    /// - Parameter array: Swift array to copy
    /// - Returns: C array pointer (caller must free with freeUInt8Array)
    public static func createUInt8Array(from array: [UInt8]) -> UnsafeMutablePointer<UInt8> {
        let buffer = UnsafeMutablePointer<UInt8>.allocate(capacity: array.count)
        buffer.initialize(from: array, count: array.count)
        return buffer
    }

    /// Free a C uint8 array allocated by createUInt8Array
    /// - Parameters:
    ///   - ptr: Array pointer to free
    ///   - count: Number of elements (for validation)
    public static func freeUInt8Array(_ ptr: UnsafeMutablePointer<UInt8>?, count: Int) {
        guard let ptr = ptr else { return }
        ptr.deinitialize(count: count)
        ptr.deallocate()
    }

    /// Create a JSON string from an Encodable object
    /// - Parameter object: Object to serialize
    /// - Returns: JSON string (caller must free)
    public static func createJSONString<T: Encodable>(from object: T) throws -> UnsafeMutablePointer<CChar> {
        let encoder = JSONEncoder()
        encoder.outputFormatting = .sortedKeys
        let data = try encoder.encode(object)
        let jsonString = String(data: data, encoding: .utf8) ?? "{}"
        return createCString(from: jsonString)
    }

    /// Parse JSON string into a Decodable object
    /// - Parameters:
    ///   - jsonString: JSON string to parse
    ///   - type: Type to decode to
    /// - Returns: Decoded object
    public static func parseJSON<T: Decodable>(_ jsonString: String, as type: T.Type) throws -> T {
        let data = jsonString.data(using: .utf8) ?? Data()
        let decoder = JSONDecoder()
        return try decoder.decode(type, from: data)
    }
}

/// Extension for autoreleasepool-based memory management
public extension MemoryManager {

    /// Execute a block within an autoreleasepool and return C string result
    /// - Parameter block: Block that returns a Swift string
    /// - Returns: C string pointer (caller must free)
    static func withAutoreleasePool<T>(_ block: () throws -> T) rethrows -> T {
        return try autoreleasepool {
            try block()
        }
    }

    /// Execute FFI operation with proper memory management
    /// - Parameters:
    ///   - operation: Operation block that may throw
    ///   - errorPtr: Pointer to receive error message
    /// - Returns: Operation result or nil on error
    static func executeFFIOperation<T>(
        _ operation: () throws -> T,
        errorPtr: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
    ) -> T? {
        do {
            return try withAutoreleasePool {
                try operation()
            }
        } catch let error as BridgeError {
            let errorMessage = createCString(from: error.localizedDescription)
            errorPtr.pointee = errorMessage
            return nil
        } catch {
            let errorMessage = createCString(from: "Unknown error: \(error.localizedDescription)")
            errorPtr.pointee = errorMessage
            return nil
        }
    }
}

/// Opaque handle registry for thread-safe FFI handle management
public class OpaqueHandleRegistry<T> {
    private let queue = DispatchQueue(label: "com.agent.handle.registry", attributes: .concurrent)
    private var handles: [UInt64: T] = [:]
    private var nextHandle: UInt64 = 1

    /// Register an object and return an opaque handle
    /// - Parameter object: Object to register
    /// - Returns: Opaque handle for the object
    public func register(_ object: T) -> UInt64 {
        queue.sync(flags: .barrier) {
            let handle = nextHandle
            handles[handle] = object
            nextHandle += 1
            return handle
        }
    }

    /// Unregister an object by handle
    /// - Parameter handle: Handle to unregister
    /// - Returns: The unregistered object, or nil if not found
    public func unregister(_ handle: UInt64) -> T? {
        queue.sync(flags: .barrier) {
            handles.removeValue(forKey: handle)
        }
    }

    /// Get an object by handle (without removing it)
    /// - Parameter handle: Handle to look up
    /// - Returns: The object, or nil if not found
    public func get(_ handle: UInt64) -> T? {
        queue.sync {
            handles[handle]
        }
    }

    /// Execute an operation with an object by handle
    /// - Parameters:
    ///   - handle: Handle to look up
    ///   - operation: Operation to perform with the object
    /// - Returns: Operation result, or nil if handle not found
    public func withHandle<U>(_ handle: UInt64, operation: (T) throws -> U) rethrows -> U? {
        guard let object = get(handle) else { return nil }
        return try operation(object)
    }

    /// Get the count of registered handles
    public var count: Int {
        queue.sync {
            handles.count
        }
    }

    /// Clear all registered handles
    public func clear() {
        queue.sync(flags: .barrier) {
            handles.removeAll()
        }
    }
}

/// Global registries for different handle types
public let modelHandleRegistry = OpaqueHandleRegistry<AnyObject>()
public let tokenizerHandleRegistry = OpaqueHandleRegistry<AnyObject>()
public let audioProcessorRegistry = OpaqueHandleRegistry<AnyObject>()
public let imageProcessorRegistry = OpaqueHandleRegistry<AnyObject>()
