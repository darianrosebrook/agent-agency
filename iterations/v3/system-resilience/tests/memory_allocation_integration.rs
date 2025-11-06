//! Integration tests for memory allocation and tracking functionality
//!
//! These tests verify that the memory management system correctly:
//! - Records allocations with proper metadata
//! - Tracks allocations by task and site
//! - Handles deallocations accurately
//! - Maintains consistent statistics
//! - Provides accurate reporting

use system_resilience::memory::{
    AllocationSiteTracker, AllocationSite, TaskAllocationStats,
    AllocationRecord, MemoryManager, MemoryManagementConfig,
};
use std::sync::Arc;
use tokio::sync::RwLock;

#[cfg(test)]
mod memory_allocation_tracking_tests {
    use super::*;

    /// Test basic allocation recording and tracking
    #[tokio::test]
    async fn test_basic_allocation_tracking() {
        let tracker = Arc::new(RwLock::new(AllocationSiteTracker::new()));

        // Record several allocations
        let site1 = AllocationSite {
            file: "test.rs".to_string(),
            line: 42,
            column: 1,
            function: "test_function".to_string(),
            module: "test_module".to_string(),
            task_id: Some("task_1".to_string()),
        };

        let site2 = AllocationSite {
            file: "test.rs".to_string(),
            line: 43,
            column: 1,
            function: "test_function".to_string(),
            module: "test_module".to_string(),
            task_id: Some("task_1".to_string()),
        };

        {
            let mut tracker_guard = tracker.write().await;

            // Record allocations
            tracker_guard.record_allocation(0x1000, 64, 8, site1.clone());
            tracker_guard.record_allocation(0x2000, 128, 16, site1.clone());
            tracker_guard.record_allocation(0x3000, 256, 32, site2.clone());
        }

        // Verify allocations are recorded
        let tracker_guard = tracker.read().await;

        // Check total allocations
        assert_eq!(tracker_guard.total_allocations(), 3);
        assert_eq!(tracker_guard.total_deallocations(), 0);

        // Check site statistics
        let site1_stats = tracker_guard.get_site_stats("test.rs", 42).unwrap();
        assert_eq!(site1_stats.total_allocations, 2);
        assert_eq!(site1_stats.total_bytes, 64 + 128);

        let site2_stats = tracker_guard.get_site_stats("test.rs", 43).unwrap();
        assert_eq!(site2_stats.total_allocations, 1);
        assert_eq!(site2_stats.total_bytes, 256);

        // Check task statistics
        let task_stats = tracker_guard.get_task_stats("task_1").unwrap();
        assert_eq!(task_stats.total_allocations, 3);
        assert_eq!(task_stats.total_bytes, 64 + 128 + 256);
        assert_eq!(task_stats.current_memory_bytes, 64 + 128 + 256);
        assert_eq!(task_stats.peak_memory_bytes, 64 + 128 + 256);
    }

    /// Test allocation and deallocation cycle
    #[tokio::test]
    async fn test_allocation_deallocation_cycle() {
        let tracker = Arc::new(RwLock::new(AllocationSiteTracker::new()));

        let site = AllocationSite {
            file: "test.rs".to_string(),
            line: 100,
            column: 1,
            function: "test_allocation".to_string(),
            module: "test_module".to_string(),
            task_id: Some("task_dealloc".to_string()),
        };

        // Record allocations
        {
            let mut tracker_guard = tracker.write().await;
            tracker_guard.record_allocation(0x1000, 100, 8, site.clone());
            tracker_guard.record_allocation(0x2000, 200, 16, site.clone());
            tracker_guard.record_allocation(0x3000, 300, 32, site.clone());
        }

        // Verify initial state
        {
            let tracker_guard = tracker.read().await;
            let task_stats = tracker_guard.get_task_stats("task_dealloc").unwrap();
            assert_eq!(task_stats.current_memory_bytes, 600);
            assert_eq!(task_stats.peak_memory_bytes, 600);
        }

        // Deallocate one allocation
        {
            let mut tracker_guard = tracker.write().await;
            tracker_guard.record_deallocation(0x2000); // 200 bytes
        }

        // Verify deallocation tracking
        {
            let tracker_guard = tracker.read().await;
            assert_eq!(tracker_guard.total_allocations(), 3);
            assert_eq!(tracker_guard.total_deallocations(), 1);

            let task_stats = tracker_guard.get_task_stats("task_dealloc").unwrap();
            assert_eq!(task_stats.current_memory_bytes, 400); // 600 - 200
            assert_eq!(task_stats.peak_memory_bytes, 600); // Peak unchanged
        }

        // Deallocate remaining allocations
        {
            let mut tracker_guard = tracker.write().await;
            tracker_guard.record_deallocation(0x1000); // 100 bytes
            tracker_guard.record_deallocation(0x3000); // 300 bytes
        }

        // Verify final state
        {
            let tracker_guard = tracker.read().await;
            assert_eq!(tracker_guard.total_allocations(), 3);
            assert_eq!(tracker_guard.total_deallocations(), 3);

            let task_stats = tracker_guard.get_task_stats("task_dealloc").unwrap();
            assert_eq!(task_stats.current_memory_bytes, 0);
            assert_eq!(task_stats.peak_memory_bytes, 600);
        }
    }

    /// Test multiple tasks allocation tracking
    #[tokio::test]
    async fn test_multi_task_allocation_tracking() {
        let tracker = Arc::new(RwLock::new(AllocationSiteTracker::new()));

        // Create allocations for different tasks
        let task1_site = AllocationSite {
            file: "task1.rs".to_string(),
            line: 10,
            column: 1,
            function: "task1_func".to_string(),
            module: "test_module".to_string(),
            task_id: Some("task_1".to_string()),
        };

        let task2_site = AllocationSite {
            file: "task2.rs".to_string(),
            line: 20,
            column: 1,
            function: "task2_func".to_string(),
            module: "test_module".to_string(),
            task_id: Some("task_2".to_string()),
        };

        let no_task_site = AllocationSite {
            file: "global.rs".to_string(),
            line: 5,
            column: 1,
            function: "global_func".to_string(),
            module: "test_module".to_string(),
            task_id: None,
        };

        // Record allocations
        {
            let mut tracker_guard = tracker.write().await;
            tracker_guard.record_allocation(0x1000, 64, 8, task1_site.clone());
            tracker_guard.record_allocation(0x2000, 128, 16, task1_site.clone());
            tracker_guard.record_allocation(0x3000, 256, 32, task2_site.clone());
            tracker_guard.record_allocation(0x4000, 512, 64, no_task_site.clone());
        }

        // Verify task-specific tracking
        let tracker_guard = tracker.read().await;

        // Task 1 stats
        let task1_stats = tracker_guard.get_task_stats("task_1").unwrap();
        assert_eq!(task1_stats.total_allocations, 2);
        assert_eq!(task1_stats.total_bytes, 64 + 128);
        assert_eq!(task1_stats.current_memory_bytes, 64 + 128);

        // Task 2 stats
        let task2_stats = tracker_guard.get_task_stats("task_2").unwrap();
        assert_eq!(task2_stats.total_allocations, 1);
        assert_eq!(task2_stats.total_bytes, 256);
        assert_eq!(task2_stats.current_memory_bytes, 256);

        // No task should not appear in task stats
        assert!(tracker_guard.get_task_stats("no_task").is_none());

        // Get top memory tasks
        let top_tasks = tracker_guard.get_top_memory_tasks(2);
        assert_eq!(top_tasks.len(), 2);
        assert_eq!(top_tasks[0].task_id, "task_2"); // 256 bytes (highest)
        assert_eq!(top_tasks[1].task_id, "task_1"); // 192 bytes
    }

    /// Test allocation site statistics
    #[tokio::test]
    async fn test_allocation_site_statistics() {
        let tracker = Arc::new(RwLock::new(AllocationSiteTracker::new()));

        let site = AllocationSite {
            file: "stats_test.rs".to_string(),
            line: 50,
            column: 1,
            function: "test_stats".to_string(),
            module: "test_module".to_string(),
            task_id: Some("stats_task".to_string()),
        };

        // Record multiple allocations at same site
        {
            let mut tracker_guard = tracker.write().await;
            for i in 0..5 {
                let ptr = 0x1000 + (i * 0x100);
                tracker_guard.record_allocation(ptr, 100 + i * 10, 8, site.clone());
            }
        }

        // Verify site statistics
        let tracker_guard = tracker.read().await;
        let site_stats = tracker_guard.get_site_stats("stats_test.rs", 50).unwrap();

        assert_eq!(site_stats.total_allocations, 5);
        assert_eq!(site_stats.total_bytes, 100 + 110 + 120 + 130 + 140); // 600 bytes

        let expected_avg = 600.0 / 5.0;
        assert!((site_stats.average_size - expected_avg).abs() < 0.001);

        // Get all site stats
        let all_sites = tracker_guard.get_all_site_stats();
        assert_eq!(all_sites.len(), 1);
        assert_eq!(all_sites[0].location, "stats_test.rs:50");
    }

    /// Test memory pressure response integration
    #[tokio::test]
    async fn test_memory_pressure_integration() {
        let config = MemoryManagementConfig {
            monitor_config: system_resilience::memory::MemoryLimitConfig {
                max_heap_mb: 1024,
                max_stack_mb: 128,
                warning_threshold_mb: 512,
                critical_threshold_mb: 768,
                enable_gc_pressure: true,
                gc_pressure_threshold_mb: 256,
                monitoring_interval_ms: 1000,
            },
            enable_object_pooling: true,
            database_connection_pool_size: 10,
            llm_client_pool_size: 5,
            enable_leak_detection: false,
            leak_detection_threshold_mb: 100,
        };

        let manager = Arc::new(MemoryManager::new(config));

        // Note: In a real test, we'd simulate memory pressure by allocating
        // large amounts of memory, but for this unit test we'll just
        // verify the manager can be created and basic stats work

        // Verify memory stats are available
        let stats = manager.get_memory_stats();
        assert!(stats.allocated_bytes >= 0);
        assert!(stats.active_allocations >= 0);

        // Verify memory pressure detection works
        let pressure = manager.get_memory_pressure();
        // Pressure should be Low in a test environment
        assert!(matches!(pressure, system_resilience::memory::MemoryPressure::Low));
    }

    /// Test concurrent allocation tracking
    #[tokio::test]
    async fn test_concurrent_allocation_tracking() {
        let tracker = Arc::new(RwLock::new(AllocationSiteTracker::new()));

        let site = AllocationSite {
            file: "concurrent.rs".to_string(),
            line: 1,
            column: 1,
            function: "concurrent_test".to_string(),
            module: "test_module".to_string(),
            task_id: Some("concurrent_task".to_string()),
        };

        // Spawn multiple tasks to record allocations concurrently
        let mut handles = vec![];

        for i in 0..10 {
            let tracker_clone = tracker.clone();
            let site_clone = site.clone();

            let handle = tokio::spawn(async move {
                let ptr = 0x1000 + (i * 0x100);
                let size = 64 + (i * 8);

                {
                    let mut tracker_guard = tracker_clone.write().await;
                    tracker_guard.record_allocation(ptr, size, 8, site_clone);
                }

                // Small delay to increase chance of race conditions
                tokio::time::sleep(std::time::Duration::from_millis(1)).await;

                // Record some deallocations
                if i % 2 == 0 {
                    let mut tracker_guard = tracker_clone.write().await;
                    tracker_guard.record_deallocation(ptr);
                }
            });

            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // Verify final state
        let tracker_guard = tracker.read().await;

        // Should have 10 allocations
        assert_eq!(tracker_guard.total_allocations(), 10);

        // Should have 5 deallocations (every other one)
        assert_eq!(tracker_guard.total_deallocations(), 5);

        // Check task stats
        let task_stats = tracker_guard.get_task_stats("concurrent_task").unwrap();
        assert_eq!(task_stats.total_allocations, 10);
        // Note: TaskAllocationStats doesn't track deallocations separately

        // Current memory should be 5 allocations remaining (odd indices not deallocated)
        let expected_current = 72 + 88 + 104 + 120 + 136; // i=1,3,5,7,9 (odd indices not deallocated)
        assert_eq!(task_stats.current_memory_bytes, expected_current);
    }

    /// Test allocation record integrity
    #[tokio::test]
    async fn test_allocation_record_integrity() {
        let tracker = Arc::new(RwLock::new(AllocationSiteTracker::new()));

        let site = AllocationSite {
            file: "integrity.rs".to_string(),
            line: 42,
            column: 1,
            function: "test_integrity".to_string(),
            module: "test_module".to_string(),
            task_id: Some("integrity_task".to_string()),
        };

        let ptr = 0xDEADBEEF;
        let size = 1337;
        let alignment = 16;

        // Record allocation
        {
            let mut tracker_guard = tracker.write().await;
            tracker_guard.record_allocation(ptr, size, alignment, site.clone());
        }

        // Verify record integrity
        let tracker_guard = tracker.read().await;

        // Get allocations for site
        let site_allocations = tracker_guard.get_allocations_for_site("integrity.rs", 42);
        assert_eq!(site_allocations.len(), 1);

        let record = &site_allocations[0];
        assert_eq!(record.ptr, ptr);
        assert_eq!(record.size, size);
        assert_eq!(record.alignment, alignment);
        assert_eq!(record.site.file, site.file);
        assert_eq!(record.site.line, site.line);
        assert_eq!(record.site.function, site.function);
        assert_eq!(record.site.task_id, site.task_id);
        assert!(!record.deallocated);

        // Verify task tracking
        let task_stats = tracker_guard.get_task_stats("integrity_task").unwrap();
        assert_eq!(task_stats.allocation_sites.len(), 1);
        assert_eq!(task_stats.allocation_sites[0], "integrity.rs:42");
    }

    /// Test memory leak detection basics
    #[tokio::test]
    async fn test_memory_leak_detection_basics() {
        let tracker = Arc::new(RwLock::new(AllocationSiteTracker::new()));

        let site = AllocationSite {
            file: "leak_test.rs".to_string(),
            line: 100,
            column: 1,
            function: "potential_leak".to_string(),
            module: "test_module".to_string(),
            task_id: Some("leak_task".to_string()),
        };

        // Record allocations without deallocating
        {
            let mut tracker_guard = tracker.write().await;
            for i in 0..10 {
                let ptr = 0x1000 + (i * 0x100);
                tracker_guard.record_allocation(ptr, 100, 8, site.clone());
            }
        }

        // Simulate time passage (in a real system, this would detect old allocations)
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // Verify allocations are tracked
        let tracker_guard = tracker.read().await;
        assert_eq!(tracker_guard.total_allocations(), 10);
        assert_eq!(tracker_guard.total_deallocations(), 0);

        let task_stats = tracker_guard.get_task_stats("leak_task").unwrap();
        assert_eq!(task_stats.total_allocations, 10);
        assert_eq!(task_stats.current_memory_bytes, 1000); // 10 * 100 bytes

        // In a real leak detection system, we'd check for allocations
        // that are older than a threshold, but this test just verifies
        // the tracking foundation is working
    }
}
