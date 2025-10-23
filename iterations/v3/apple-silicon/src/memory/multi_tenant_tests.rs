//! Tests for multi-tenant memory management functionality

use super::*;
use crate::MemoryConfig;
use uuid::Uuid;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tenant_registration() {
        let manager = MultiTenantMemoryManager::new(MemoryConfig {
            max_memory_mb: 1024,
            ..Default::default()
        });

        let tenant_id = Uuid::new_v4();
        let config = TenantMemoryConfig {
            tenant_id,
            max_memory_mb: 256,
            priority: TenantPriority::Normal,
            guaranteed_memory_mb: 64,
            burst_limit_mb: Some(512),
            isolation_level: IsolationLevel::Soft,
        };

        // Test successful registration
        assert!(manager.register_tenant(config).await.is_ok());

        // Verify tenant is registered
        let usage = manager.get_tenant_usage(tenant_id).await.unwrap();
        assert_eq!(usage.tenant_id, tenant_id);
        assert_eq!(usage.allocated_memory_mb, 0);
        assert_eq!(usage.model_count, 0);
    }

    #[tokio::test]
    async fn test_tenant_registration_validation() {
        let manager = MultiTenantMemoryManager::new(MemoryConfig {
            max_memory_mb: 1024,
            ..Default::default()
        });

        let tenant_id = Uuid::new_v4();

        // Test invalid config: guaranteed > max
        let invalid_config = TenantMemoryConfig {
            tenant_id,
            max_memory_mb: 128,
            priority: TenantPriority::Normal,
            guaranteed_memory_mb: 256, // Invalid: greater than max
            burst_limit_mb: None,
            isolation_level: IsolationLevel::Soft,
        };

        assert!(manager.register_tenant(invalid_config).await.is_err());

        // Test invalid config: burst < max
        let invalid_burst_config = TenantMemoryConfig {
            tenant_id: Uuid::new_v4(),
            max_memory_mb: 256,
            priority: TenantPriority::Normal,
            guaranteed_memory_mb: 64,
            burst_limit_mb: Some(128), // Invalid: less than max
            isolation_level: IsolationLevel::Soft,
        };

        assert!(manager.register_tenant(invalid_burst_config).await.is_err());
    }

    #[tokio::test]
    async fn test_memory_allocation() {
        let manager = MultiTenantMemoryManager::new(MemoryConfig {
            max_memory_mb: 1024,
            ..Default::default()
        });

        let tenant_id = Uuid::new_v4();
        let config = TenantMemoryConfig {
            tenant_id,
            max_memory_mb: 256,
            priority: TenantPriority::Normal,
            guaranteed_memory_mb: 64,
            burst_limit_mb: None,
            isolation_level: IsolationLevel::Soft,
        };

        manager.register_tenant(config).await.unwrap();

        // Test successful allocation
        let request = MemoryAllocationRequest {
            tenant_id,
            requested_memory_mb: 128,
            model_name: "test-model".to_string(),
            allocation_type: AllocationType::ModelLoad,
        };

        match manager.request_allocation(request).await.unwrap() {
            MemoryAllocationResponse::Granted { allocation_id } => {
                assert!(!allocation_id.is_empty());
            }
            _ => panic!("Expected granted allocation"),
        }

        // Verify allocation tracking
        let usage = manager.get_tenant_usage(tenant_id).await.unwrap();
        assert_eq!(usage.allocated_memory_mb, 128);
        assert_eq!(usage.used_memory_mb, 128);
        assert_eq!(usage.model_count, 1);
    }

    #[tokio::test]
    async fn test_memory_allocation_limits() {
        let manager = MultiTenantMemoryManager::new(MemoryConfig {
            max_memory_mb: 1024,
            ..Default::default()
        });

        let tenant_id = Uuid::new_v4();
        let config = TenantMemoryConfig {
            tenant_id,
            max_memory_mb: 128, // Small limit
            priority: TenantPriority::Normal,
            guaranteed_memory_mb: 32,
            burst_limit_mb: None,
            isolation_level: IsolationLevel::Hard,
        };

        manager.register_tenant(config).await.unwrap();

        // First allocation should succeed
        let request1 = MemoryAllocationRequest {
            tenant_id,
            requested_memory_mb: 64,
            model_name: "model1".to_string(),
            allocation_type: AllocationType::ModelLoad,
        };
        assert!(matches!(
            manager.request_allocation(request1).await.unwrap(),
            MemoryAllocationResponse::Granted { .. }
        ));

        // Second allocation should be denied (exceeds limit)
        let request2 = MemoryAllocationRequest {
            tenant_id,
            requested_memory_mb: 80, // 64 + 80 > 128
            model_name: "model2".to_string(),
            allocation_type: AllocationType::ModelLoad,
        };

        match manager.request_allocation(request2).await.unwrap() {
            MemoryAllocationResponse::Denied { reason, available_memory_mb } => {
                assert!(reason.contains("limit exceeded"));
                assert_eq!(available_memory_mb, 64); // 128 - 64
            }
            _ => panic!("Expected denied allocation"),
        }
    }

    #[tokio::test]
    async fn test_memory_release() {
        let manager = MultiTenantMemoryManager::new(MemoryConfig {
            max_memory_mb: 1024,
            ..Default::default()
        });

        let tenant_id = Uuid::new_v4();
        let config = TenantMemoryConfig {
            tenant_id,
            max_memory_mb: 256,
            priority: TenantPriority::Normal,
            guaranteed_memory_mb: 64,
            burst_limit_mb: None,
            isolation_level: IsolationLevel::Soft,
        };

        manager.register_tenant(config).await.unwrap();

        // Allocate memory
        let request = MemoryAllocationRequest {
            tenant_id,
            requested_memory_mb: 128,
            model_name: "test-model".to_string(),
            allocation_type: AllocationType::ModelLoad,
        };

        let allocation_id = match manager.request_allocation(request).await.unwrap() {
            MemoryAllocationResponse::Granted { allocation_id } => allocation_id,
            _ => panic!("Expected granted allocation"),
        };

        // Release memory
        manager.release_allocation(tenant_id, &allocation_id, 64).await.unwrap();

        // Verify partial release
        let usage = manager.get_tenant_usage(tenant_id).await.unwrap();
        assert_eq!(usage.allocated_memory_mb, 64); // 128 - 64
        assert_eq!(usage.used_memory_mb, 64);
    }

    #[tokio::test]
    async fn test_tenant_dashboard() {
        let manager = MultiTenantMemoryManager::new(MemoryConfig {
            max_memory_mb: 1024,
            ..Default::default()
        });

        // Register multiple tenants
        let tenant1_id = Uuid::new_v4();
        let tenant1_config = TenantMemoryConfig {
            tenant_id: tenant1_id,
            max_memory_mb: 256,
            priority: TenantPriority::High,
            guaranteed_memory_mb: 64,
            burst_limit_mb: None,
            isolation_level: IsolationLevel::Soft,
        };
        manager.register_tenant(tenant1_config).await.unwrap();

        let tenant2_id = Uuid::new_v4();
        let tenant2_config = TenantMemoryConfig {
            tenant_id: tenant2_id,
            max_memory_mb: 128,
            priority: TenantPriority::Normal,
            guaranteed_memory_mb: 32,
            burst_limit_mb: None,
            isolation_level: IsolationLevel::Soft,
        };
        manager.register_tenant(tenant2_config).await.unwrap();

        // Allocate memory for tenants
        let request1 = MemoryAllocationRequest {
            tenant_id: tenant1_id,
            requested_memory_mb: 128,
            model_name: "model1".to_string(),
            allocation_type: AllocationType::ModelLoad,
        };
        manager.request_allocation(request1).await.unwrap();

        let request2 = MemoryAllocationRequest {
            tenant_id: tenant2_id,
            requested_memory_mb: 64,
            model_name: "model2".to_string(),
            allocation_type: AllocationType::ModelLoad,
        };
        manager.request_allocation(request2).await.unwrap();

        // Get dashboard
        let dashboard = manager.get_tenant_dashboard().await.unwrap();

        // Verify dashboard contents
        assert_eq!(dashboard.summary.total_tenants, 2);
        assert_eq!(dashboard.summary.active_tenants, 2);
        assert_eq!(dashboard.summary.total_allocated_memory_mb, 192); // 128 + 64

        // Check tenant summaries (should be sorted by priority then utilization)
        assert_eq!(dashboard.tenant_summaries.len(), 2);
        assert_eq!(dashboard.tenant_summaries[0].tenant_id, tenant1_id); // Higher priority first
        assert_eq!(dashboard.tenant_summaries[1].tenant_id, tenant2_id);
    }

    #[tokio::test]
    async fn test_isolation_levels() {
        let manager = MultiTenantMemoryManager::new(MemoryConfig {
            max_memory_mb: 1024,
            ..Default::default()
        });

        // Test hard isolation
        let tenant_id_hard = Uuid::new_v4();
        let config_hard = TenantMemoryConfig {
            tenant_id: tenant_id_hard,
            max_memory_mb: 100,
            priority: TenantPriority::Normal,
            guaranteed_memory_mb: 50,
            burst_limit_mb: None,
            isolation_level: IsolationLevel::Hard,
        };
        manager.register_tenant(config_hard).await.unwrap();

        // Test soft isolation
        let tenant_id_soft = Uuid::new_v4();
        let config_soft = TenantMemoryConfig {
            tenant_id: tenant_id_soft,
            max_memory_mb: 200,
            priority: TenantPriority::Normal,
            guaranteed_memory_mb: 50,
            burst_limit_mb: None,
            isolation_level: IsolationLevel::Soft,
        };
        manager.register_tenant(config_soft).await.unwrap();

        // Hard isolation tenant should be denied when exceeding limit
        let request_hard = MemoryAllocationRequest {
            tenant_id: tenant_id_hard,
            requested_memory_mb: 150, // Exceeds 100 MB limit
            model_name: "model-hard".to_string(),
            allocation_type: AllocationType::ModelLoad,
        };

        match manager.request_allocation(request_hard).await.unwrap() {
            MemoryAllocationResponse::Denied { .. } => {} // Expected
            _ => panic!("Expected denied allocation for hard isolation"),
        }
    }

    #[tokio::test]
    async fn test_tenant_priority_allocation() {
        let manager = MultiTenantMemoryManager::new(MemoryConfig {
            max_memory_mb: 300, // Limited global memory
            ..Default::default()
        });

        // Create high and low priority tenants
        let high_priority_id = Uuid::new_v4();
        let high_config = TenantMemoryConfig {
            tenant_id: high_priority_id,
            max_memory_mb: 200,
            priority: TenantPriority::High,
            guaranteed_memory_mb: 50,
            burst_limit_mb: None,
            isolation_level: IsolationLevel::Soft,
        };
        manager.register_tenant(high_config).await.unwrap();

        let low_priority_id = Uuid::new_v4();
        let low_config = TenantMemoryConfig {
            tenant_id: low_priority_id,
            max_memory_mb: 200,
            priority: TenantPriority::Low,
            guaranteed_memory_mb: 50,
            burst_limit_mb: None,
            isolation_level: IsolationLevel::Soft,
        };
        manager.register_tenant(low_config).await.unwrap();

        // Both tenants request large allocations
        let request_high = MemoryAllocationRequest {
            tenant_id: high_priority_id,
            requested_memory_mb: 150,
            model_name: "high-model".to_string(),
            allocation_type: AllocationType::ModelLoad,
        };

        let request_low = MemoryAllocationRequest {
            tenant_id: low_priority_id,
            requested_memory_mb: 150,
            model_name: "low-model".to_string(),
            allocation_type: AllocationType::ModelLoad,
        };

        // Both should succeed initially (global memory allows)
        assert!(matches!(
            manager.request_allocation(request_high).await.unwrap(),
            MemoryAllocationResponse::Granted { .. }
        ));

        assert!(matches!(
            manager.request_allocation(request_low).await.unwrap(),
            MemoryAllocationResponse::Granted { .. }
        ));

    // Test memory pressure by creating a scenario with limited total memory
    // Create a new manager with small memory limit to force critical pressure
    let small_config = crate::MemoryConfig {
        max_memory_mb: 250, // Only 250MB total, forcing critical pressure with 300MB allocation
        check_interval_ms: 1000,
        pressure_monitoring: true,
        cleanup_threshold_percent: 80,
    };
    let manager_small = crate::memory::manager::MultiTenantMemoryManager::new(small_config);

    // Register tenants with the small memory manager
    manager_small.register_tenant(high_config).await.unwrap();
    manager_small.register_tenant(low_config).await.unwrap();

    // First allocation should succeed (within normal pressure range)
    let high_result = manager_small.request_allocation(request_high).await.unwrap();
    assert!(matches!(high_result, crate::memory::manager::MemoryAllocationResponse::Granted { .. }));

    // Check that memory pressure is now high/critical
    let status = manager_small.get_memory_status().await;
    assert!(matches!(status.memory_pressure, crate::MemoryPressure::Critical | crate::MemoryPressure::High));

    // Second allocation should be denied due to critical memory pressure
    let low_result = manager_small.request_allocation(request_low).await.unwrap();
    assert!(matches!(low_result, crate::memory::manager::MemoryAllocationResponse::Denied { .. }));
    }
}
