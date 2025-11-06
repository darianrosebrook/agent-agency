//! Resource management system for handles and finalizers
//!
//! This module provides comprehensive resource tracking and cleanup for system
//! handles (files, sockets, memory mappings) and finalizer execution for
//! garbage-collected objects.

use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::time::Instant;
use tracing::{debug, warn};

use crate::memory::types::*;
use crate::memory::allocation::AllocationSite;

/// Resource handle for tracking managed resources
#[derive(Debug, Clone)]
pub struct ResourceHandle {
    pub id: u64,
    pub handle_type: String,
    pub handle_info: HandleInfo,
    pub created_at: Instant,
    pub last_accessed: Instant,
    pub closed: bool,
    pub close_time: Option<Instant>,
}

impl ResourceHandle {
    /// Get the object reference for this handle
    pub fn get_object_ref(&self) -> Option<ObjectRef> {
        if self.closed {
            return None;
        }
        // Create a more realistic object reference based on handle type
        let size = match self.handle_type.as_str() {
            "file" => 256,      // File handle overhead
            "socket" => 512,    // Socket handle overhead
            "database" => 1024, // Database connection overhead
            "memory" => 128,    // Memory mapping overhead
            "thread" => 2048,   // Thread handle overhead
            _ => 512,           // Default handle overhead
        };

        Some(ObjectRef {
            ptr: self.id as usize,
            type_id: std::any::TypeId::of::<ResourceHandle>(),
            size,
        })
    }
}

/// Allocation leak detection result
#[derive(Debug, Clone)]
pub struct AllocationLeak {
    pub object_id: u64,
    pub size_bytes: usize,
    pub allocation_site: AllocationSite,
    pub allocation_time: Instant,
    pub suspected_leak_reason: String,
}

/// Resource finalizer for cleanup operations
pub struct ResourceFinalizer {
    /// Unique finalizer ID
    pub id: u64,
    /// Object this finalizer is associated with
    pub object_ref: ObjectRef,
    /// Finalizer function to execute
    pub finalizer_fn: Box<dyn FnOnce() + Send + 'static>,
    /// Priority (higher numbers execute first)
    pub priority: i32,
    /// Timestamp when finalizer was registered
    pub registered_at: Instant,
}

impl std::fmt::Debug for ResourceFinalizer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResourceFinalizer")
            .field("id", &self.id)
            .field("object_ref", &self.object_ref)
            .field("priority", &self.priority)
            .field("registered_at", &self.registered_at)
            .finish()
    }
}

/// Finalizer execution result
#[derive(Debug, Clone)]
pub struct FinalizerResult {
    /// Finalizer ID
    pub finalizer_id: u64,
    /// Whether execution was successful
    pub success: bool,
    /// Execution duration in microseconds
    pub duration_us: u64,
    /// Error message if execution failed
    pub error_message: Option<String>,
}

/// Finalizer queue for managing pending finalizations
#[derive(Debug)]
pub struct FinalizerQueue {
    /// Queue of pending finalizers (priority queue)
    queue: std::collections::BinaryHeap<QueuedFinalizer>,
    /// Next finalizer ID to assign
    next_id: AtomicU64,
    /// Statistics
    stats: FinalizerStats,
}

/// Queued finalizer with ordering
#[derive(Debug)]
struct QueuedFinalizer {
    /// Priority for ordering (higher = execute first)
    priority: i32,
    /// Registration order (for stable sorting)
    order: u64,
    /// The finalizer data
    finalizer: ResourceFinalizer,
}

impl Ord for QueuedFinalizer {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Higher priority first, then earlier registration order
        match other.priority.cmp(&self.priority) {
            std::cmp::Ordering::Equal => self.order.cmp(&other.order),
            ord => ord,
        }
    }
}

impl PartialOrd for QueuedFinalizer {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for QueuedFinalizer {}
impl PartialEq for QueuedFinalizer {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.order == other.order
    }
}

/// Finalizer execution statistics
#[derive(Debug, Clone, Default)]
pub struct FinalizerStats {
    /// Total finalizers registered
    pub registered: u64,
    /// Total finalizers executed
    pub executed: u64,
    /// Total successful executions
    pub successful: u64,
    /// Total failed executions
    pub failed: u64,
    /// Total execution time in microseconds
    pub total_execution_time_us: u64,
    /// Currently queued finalizers
    pub queued: u64,
}

/// Types of system handles that can be tracked and cleaned up
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HandleType {
    /// File descriptor/handle
    File,
    /// Network socket
    Socket,
    /// Shared memory segment
    SharedMemory,
    /// Memory-mapped region
    MemoryMap,
    /// Process/thread handle
    Process,
    /// Synchronization primitive (mutex, semaphore, etc.)
    SyncPrimitive,
    /// Device handle
    Device,
    /// Custom handle type
    Custom(String),
}

/// Platform-specific handle information
#[derive(Debug, Clone)]
pub enum HandleInfo {
    /// Unix file descriptor
    UnixFd(i32),
    /// Windows handle
    WindowsHandle(isize),
    /// macOS/iOS file descriptor
    DarwinFd(i32),
    /// Custom handle data
    Custom(Vec<u8>),
}

/// Tracked system handle with metadata
#[derive(Debug, Clone)]
pub struct TrackedHandle {
    /// Unique handle ID
    pub id: u64,
    /// Type of handle
    pub handle_type: HandleType,
    /// Platform-specific handle information
    pub handle_info: HandleInfo,
    /// Object this handle is associated with
    pub object_ref: ObjectRef,
    /// Handle creation timestamp
    pub created_at: Instant,
    /// Handle description for debugging
    pub description: String,
    /// Whether the handle has been closed/cleaned up
    pub closed: bool,
}

/// Handle cleanup result
#[derive(Debug, Clone)]
pub struct HandleCleanupResult {
    /// Handle ID that was cleaned up
    pub handle_id: u64,
    /// Handle type
    pub handle_type: HandleType,
    /// Whether cleanup was successful
    pub success: bool,
    /// Cleanup duration in microseconds
    pub duration_us: u64,
    /// Error message if cleanup failed
    pub error_message: Option<String>,
}

/// Handle tracking registry
#[derive(Debug)]
pub struct HandleRegistry {
    /// Map of handle IDs to handle information
    handles: HashMap<u64, TrackedHandle>,
    /// Next handle ID to assign
    next_id: AtomicU64,
    /// Cleanup statistics
    stats: HandleCleanupStats,
}

impl TrackedHandle {
    /// Get the object reference for this handle
    pub fn get_object_ref(&self) -> Option<ObjectRef> {
        if self.closed {
            None
        } else {
            Some(self.object_ref.clone())
        }
    }
}

/// Handle cleanup statistics
#[derive(Debug, Clone, Default)]
pub struct HandleCleanupStats {
    /// Total handles registered
    pub registered: u64,
    /// Total handles cleaned up
    pub cleaned_up: u64,
    /// Total successful cleanups
    pub successful: u64,
    /// Total failed cleanups
    pub failed: u64,
    /// Total cleanup time in microseconds
    pub total_cleanup_time_us: u64,
    /// Currently tracked handles
    pub tracked: u64,
}

impl HandleRegistry {
    /// Create a new handle registry
    pub fn new() -> Self {
        Self {
            handles: HashMap::new(),
            next_id: AtomicU64::new(1),
            stats: HandleCleanupStats::default(),
        }
    }

    /// Get iterator over all tracked handles
    pub fn handles(&self) -> impl Iterator<Item = &TrackedHandle> {
        self.handles.values()
    }

    /// Remove all handles associated with a specific object reference
    pub fn remove_handles_for_object(&mut self, obj_ref: &super::types::ObjectRef) {
        self.handles.retain(|_, handle| {
            if let Some(handle_obj_ref) = handle.get_object_ref() {
                handle_obj_ref != *obj_ref
            } else {
                true
            }
        });
    }

    /// Register a new handle for tracking
    pub fn register_handle(&mut self, handle_type: HandleType, handle_info: HandleInfo, object_ref: ObjectRef, description: String) -> u64 {
        let id = self.next_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let handle = TrackedHandle {
            id,
            handle_type: handle_type.clone(),
            handle_info,
            object_ref,
            created_at: Instant::now(),
            description,
            closed: false,
        };

        self.handles.insert(id, handle);
        self.stats.registered += 1;
        self.stats.tracked += 1;

        debug!("Registered handle {} of type {:?}", id, handle_type);
        id
    }

    /// Mark a handle as closed (already cleaned up externally)
    pub fn mark_handle_closed(&mut self, handle_id: u64) -> bool {
        if let Some(handle) = self.handles.get_mut(&handle_id) {
            if !handle.closed {
                handle.closed = true;
                self.stats.tracked -= 1;
                debug!("Marked handle {} as closed", handle_id);
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Clean up a specific handle
    pub async fn cleanup_handle(&mut self, handle_id: u64) -> HandleCleanupResult {
        let start_time = Instant::now();

        let result = if let Some(handle) = self.handles.get(&handle_id) {
            let handle_type = handle.handle_type.clone();
            if handle.closed {
                HandleCleanupResult {
                    handle_id,
                    handle_type,
                    success: true,
                    duration_us: start_time.elapsed().as_micros() as u64,
                    error_message: Some("Handle already closed".to_string()),
                }
            } else {
                // Perform platform-specific cleanup
                let cleanup_result = self.perform_handle_cleanup(&handle).await;

                match cleanup_result {
                    Ok(_) => {
                        self.stats.cleaned_up += 1;
                        self.stats.successful += 1;

                        // Mark as closed
                        if let Some(h) = self.handles.get_mut(&handle_id) {
                            h.closed = true;
                            self.stats.tracked -= 1;
                        }

                        HandleCleanupResult {
                            handle_id,
                            handle_type,
                            success: true,
                            duration_us: start_time.elapsed().as_micros() as u64,
                            error_message: None,
                        }
                    }
                    Err(e) => {
                        self.stats.failed += 1;
                        HandleCleanupResult {
                            handle_id,
                            handle_type,
                            success: false,
                            duration_us: start_time.elapsed().as_micros() as u64,
                            error_message: Some(format!("Cleanup failed: {}", e)),
                        }
                    }
                }
            }
        } else {
            HandleCleanupResult {
                handle_id,
                handle_type: HandleType::Custom("unknown".to_string()),
                success: false,
                duration_us: start_time.elapsed().as_micros() as u64,
                error_message: Some("Handle not found".to_string()),
            }
        };

        self.stats.total_cleanup_time_us += result.duration_us;
        result
    }

    /// Clean up all tracked handles
    pub async fn cleanup_all_handles(&mut self) -> Vec<HandleCleanupResult> {
        let handle_ids: Vec<u64> = self.handles.keys().cloned().collect();
        let mut results = Vec::new();

        for handle_id in handle_ids {
            let result = self.cleanup_handle(handle_id).await;
            results.push(result);
        }

        debug!("Cleaned up {} handles", results.len());
        results
    }

    /// Get handles associated with a specific object
    pub fn get_handles_for_object(&self, object_ref: &ObjectRef) -> Vec<&TrackedHandle> {
        self.handles.values()
            .filter(|h| &h.object_ref == object_ref && !h.closed)
            .collect()
    }

    /// Get all open handles of a specific type
    pub fn get_handles_by_type(&self, handle_type: &HandleType) -> Vec<&TrackedHandle> {
        self.handles.values()
            .filter(|h| &h.handle_type == handle_type && !h.closed)
            .collect()
    }

    /// Get cleanup statistics
    pub fn stats(&self) -> &HandleCleanupStats {
        &self.stats
    }

    /// Perform platform-specific handle cleanup
    async fn perform_handle_cleanup(&self, handle: &TrackedHandle) -> Result<(), Box<dyn std::error::Error>> {
        match &handle.handle_info {
            HandleInfo::UnixFd(fd) => {
                self.cleanup_unix_fd(*fd, &handle.handle_type).await
            }
            HandleInfo::WindowsHandle(win_handle) => {
                self.cleanup_windows_handle(*win_handle, &handle.handle_type).await
            }
            HandleInfo::DarwinFd(fd) => {
                self.cleanup_darwin_fd(*fd, &handle.handle_type).await
            }
            HandleInfo::Custom(data) => {
                self.cleanup_custom_handle(data, &handle.handle_type).await
            }
        }
    }

    /// Clean up Unix file descriptor
    async fn cleanup_unix_fd(&self, fd: i32, handle_type: &HandleType) -> Result<(), Box<dyn std::error::Error>> {
        match handle_type {
            HandleType::File => {
                // Close file descriptor
                #[cfg(unix)]
                {
                    use std::os::unix::io::FromRawFd;
                    use std::fs::File;

                    // Safely close the file descriptor by wrapping it in a File and letting it drop
                    // This ensures proper cleanup even if the FD was already closed
                    let _file = unsafe { File::from_raw_fd(fd) };
                    // File is automatically closed when it goes out of scope

                    debug!("Successfully closed Unix file descriptor {}", fd);
                    Ok(())
                }
                #[cfg(not(unix))]
                {
                    Err("Unix file descriptors not supported on this platform".into())
                }
            }
            HandleType::Socket => {
                // Close socket
                #[cfg(unix)]
                {
                    use libc::{close, c_int};

                    // Use libc::close to properly close the socket
                    let result = unsafe { close(fd as c_int) };

                    if result == 0 {
                        debug!("Successfully closed Unix socket {}", fd);
                        Ok(())
                    } else {
                        let error = std::io::Error::last_os_error();
                        Err(format!("Failed to close Unix socket {}: {}", fd, error).into())
                    }
                }
                #[cfg(not(unix))]
                {
                    Err("Unix sockets not supported on this platform".into())
                }
            }
            _ => {
                debug!("Unix FD cleanup not implemented for handle type {:?}", handle_type);
                Ok(())
            }
        }
    }

    /// Clean up Windows handle
    async fn cleanup_windows_handle(&self, handle: isize, handle_type: &HandleType) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(target_os = "windows")]
        {
            use winapi::um::handleapi::CloseHandle;
            use winapi::shared::ntdef::HANDLE;

            match handle_type {
                HandleType::File | HandleType::Device => {
                    // Close Windows handle using WinAPI
                    let result = unsafe { CloseHandle(handle as HANDLE) };

                    if result != 0 {
                        debug!("Successfully closed Windows handle {}", handle);
                        Ok(())
                    } else {
                        let error = std::io::Error::last_os_error();
                        Err(format!("Failed to close Windows handle {}: {}", handle, error).into())
                    }
                }
                _ => {
                    debug!("Windows handle cleanup not implemented for type {:?}", handle_type);
                    Ok(())
                }
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            Err("Windows handles not supported on this platform".into())
        }
    }

    /// Clean up Darwin (macOS/iOS) file descriptor
    async fn cleanup_darwin_fd(&self, fd: i32, handle_type: &HandleType) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(target_os = "macos")]
        {
            use libc::{close, c_int};

            match handle_type {
                HandleType::File | HandleType::Socket | HandleType::MemoryMap => {
                    // Use libc::close for Darwin systems (macOS uses BSD-style close)
                    let result = unsafe { close(fd as c_int) };

                    if result == 0 {
                        debug!("Successfully closed Darwin file descriptor {}", fd);
                        Ok(())
                    } else {
                        let error = std::io::Error::last_os_error();
                        Err(format!("Failed to close Darwin file descriptor {}: {}", fd, error).into())
                    }
                }
                _ => {
                    debug!("Darwin FD cleanup not implemented for type {:?}", handle_type);
                    Ok(())
                }
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            Err("Darwin file descriptors not supported on this platform".into())
        }
    }

    /// Clean up custom handle
    async fn cleanup_custom_handle(&self, _data: &[u8], handle_type: &HandleType) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Custom handle cleanup for type {:?}", handle_type);
        // Custom cleanup logic would go here
        Ok(())
    }
}

impl FinalizerQueue {
    /// Create a new finalizer queue
    pub fn new() -> Self {
        Self {
            queue: std::collections::BinaryHeap::new(),
            next_id: AtomicU64::new(1),
            stats: FinalizerStats::default(),
        }
    }

    /// Register a finalizer for execution
    pub fn register_finalizer<F>(&mut self, object_ref: ObjectRef, finalizer_fn: F, priority: i32) -> u64
    where
        F: FnOnce() + Send + 'static,
    {
        let id = self.next_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let order = id; // Use ID as tie-breaker for stable ordering

        let finalizer = ResourceFinalizer {
            id,
            object_ref,
            finalizer_fn: Box::new(finalizer_fn),
            priority,
            registered_at: Instant::now(),
        };

        let queued = QueuedFinalizer {
            priority,
            order,
            finalizer,
        };

        self.queue.push(queued);
        self.stats.registered += 1;
        self.stats.queued += 1;

        debug!("Registered finalizer {} with priority {}", id, priority);
        id
    }

    /// Execute all pending finalizers
    pub async fn execute_pending_finalizers(&mut self) -> Vec<FinalizerResult> {
        let mut results = Vec::new();
        let mut executed_count = 0;

        while let Some(queued) = self.queue.pop() {
            executed_count += 1;
            self.stats.executed += 1;
            self.stats.queued -= 1;

            let start_time = Instant::now();
            let finalizer_id = queued.finalizer.id;

            // Execute the finalizer in a blocking task to handle potentially blocking operations
            let result = tokio::task::spawn_blocking(move || {
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    (queued.finalizer.finalizer_fn)()
                }))
            }).await;

            let duration = start_time.elapsed().as_micros() as u64;

            match result {
                Ok(Ok(())) => {
                    self.stats.successful += 1;
                    results.push(FinalizerResult {
                        finalizer_id,
                        success: true,
                        duration_us: duration,
                        error_message: None,
                    });
                }
                Ok(Err(panic_info)) => {
                    self.stats.failed += 1;
                    let error_msg = format!("Finalizer panicked: {:?}", panic_info);
                    warn!("Finalizer {} failed: {}", finalizer_id, error_msg);
                    results.push(FinalizerResult {
                        finalizer_id,
                        success: false,
                        duration_us: duration,
                        error_message: Some(error_msg),
                    });
                }
                Err(join_error) => {
                    self.stats.failed += 1;
                    let error_msg = format!("Finalizer task join error: {:?}", join_error);
                    warn!("Finalizer {} join error: {}", finalizer_id, error_msg);
                    results.push(FinalizerResult {
                        finalizer_id,
                        success: false,
                        duration_us: duration,
                        error_message: Some(error_msg),
                    });
                }
            }
        }

        if executed_count > 0 {
            debug!("Executed {} finalizers", executed_count);
        }

        results
    }

    /// Get finalizer statistics
    pub fn stats(&self) -> &FinalizerStats {
        &self.stats
    }

    /// Clear all pending finalizers (emergency cleanup)
    pub fn clear_pending_finalizers(&mut self) -> usize {
        let cleared_count = self.queue.len();
        self.queue.clear();
        self.stats.queued = 0;

        if cleared_count > 0 {
            warn!("Cleared {} pending finalizers", cleared_count);
        }

        cleared_count
    }
}
