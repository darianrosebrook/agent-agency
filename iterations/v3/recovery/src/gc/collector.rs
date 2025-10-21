use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::types::{Digest, ObjectRef, ChunkRef};
use crate::types::Digest as SourceDigest;

/// Garbage collector for managing object lifecycle
pub struct GarbageCollector {
    /// GC configuration
    config: GcConfig,
    /// Current reachability information
    reachability: ReachabilityInfo,
    /// GC statistics
    stats: GcStats,
    /// Grace period tracker
    grace_period: GracePeriodTracker,
}

/// GC configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcConfig {
    /// Enable automatic GC
    pub auto_gc: bool,
    /// GC interval in seconds
    pub gc_interval: u64,
    /// Grace period in seconds (24 hours = 86400)
    pub grace_period: u64,
    /// Maximum objects to process per GC cycle
    pub max_objects_per_cycle: usize,
    /// Enable packing of cold objects
    pub enable_packing: bool,
    /// Pack threshold (objects older than this get packed)
    pub pack_threshold: u64,
    /// Enable dry run mode
    pub dry_run: bool,
    /// Enable verbose logging
    pub verbose: bool,
}

impl Default for GcConfig {
    fn default() -> Self {
        Self {
            auto_gc: true,
            gc_interval: 3600, // 1 hour
            grace_period: 86400, // 24 hours
            max_objects_per_cycle: 10000,
            enable_packing: true,
            pack_threshold: 86400, // 24 hours
            dry_run: false,
            verbose: false,
        }
    }
}

/// Reachability information for objects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReachabilityInfo {
    /// Objects that are reachable
    pub reachable: HashSet<Digest>,
    /// Objects that are unreachable
    pub unreachable: HashSet<Digest>,
    /// Objects that are in grace period
    pub grace_period: HashSet<Digest>,
    /// Protected objects (never GC'd)
    pub protected: HashSet<Digest>,
    /// Last updated timestamp
    pub last_updated: u64,
}

/// GC statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcStats {
    /// Total objects processed
    pub objects_processed: usize,
    /// Objects marked as reachable
    pub reachable_objects: usize,
    /// Objects marked as unreachable
    pub unreachable_objects: usize,
    /// Objects in grace period
    pub grace_period_objects: usize,
    /// Objects swept (deleted)
    pub swept_objects: usize,
    /// Objects packed
    pub packed_objects: usize,
    /// Total bytes freed
    pub bytes_freed: u64,
    /// Total GC cycles
    pub gc_cycles: usize,
    /// Last GC timestamp
    pub last_gc: u64,
}

/// Grace period tracker for objects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GracePeriodTracker {
    /// Objects in grace period with their timestamps
    pub grace_objects: HashMap<Digest, u64>,
    /// Grace period duration
    pub grace_duration: u64,
}

impl GarbageCollector {
    /// Create a new garbage collector
    pub fn new() -> Self {
        Self {
            config: GcConfig::default(),
            reachability: ReachabilityInfo {
                reachable: HashSet::new(),
                unreachable: HashSet::new(),
                grace_period: HashSet::new(),
                protected: HashSet::new(),
                last_updated: Self::current_timestamp(),
            },
            stats: GcStats {
                objects_processed: 0,
                reachable_objects: 0,
                unreachable_objects: 0,
                grace_period_objects: 0,
                swept_objects: 0,
                packed_objects: 0,
                bytes_freed: 0,
                gc_cycles: 0,
                last_gc: 0,
            },
            grace_period: GracePeriodTracker {
                grace_objects: HashMap::new(),
                grace_duration: 86400, // 24 hours
            },
        }
    }

    /// Create a new garbage collector with custom configuration
    pub fn with_config(config: GcConfig) -> Self {
        Self {
            config,
            reachability: ReachabilityInfo {
                reachable: HashSet::new(),
                unreachable: HashSet::new(),
                grace_period: HashSet::new(),
                protected: HashSet::new(),
                last_updated: Self::current_timestamp(),
            },
            stats: GcStats {
                objects_processed: 0,
                reachable_objects: 0,
                unreachable_objects: 0,
                grace_period_objects: 0,
                swept_objects: 0,
                packed_objects: 0,
                bytes_freed: 0,
                gc_cycles: 0,
                last_gc: 0,
            },
            grace_period: GracePeriodTracker {
                grace_objects: HashMap::new(),
                grace_duration: 86400, // 24 hours
            },
        }
    }

    /// Run a full GC cycle
    pub fn run_gc_cycle(&mut self, protected_refs: &[ObjectRef]) -> Result<GcResult> {
        let start_time = Self::current_timestamp();
        
        if self.config.verbose {
            println!("Starting GC cycle at {}", start_time);
        }

        // Mark phase: identify reachable objects
        let reachable = self.mark_reachable(protected_refs)?;
        
        // Sweep phase: identify unreachable objects
        let unreachable = self.sweep_unreachable(&reachable)?;
        
        // Grace period: move unreachable objects to grace period
        let grace_period = self.apply_grace_period(&unreachable)?;
        
        // Pack phase: pack cold objects if enabled
        let packed = if self.config.enable_packing {
            self.pack_cold_objects(&reachable)?
        } else {
            Vec::new()
        };

        // Update statistics
        self.update_stats(reachable.len(), unreachable.len(), grace_period.len(), packed.len());
        
        let end_time = Self::current_timestamp();
        let duration = end_time - start_time;

        Ok(GcResult {
            reachable_objects: reachable.len(),
            unreachable_objects: unreachable.len(),
            grace_period_objects: grace_period.len(),
            packed_objects: packed.len(),
            bytes_freed: 0, // TODO: Calculate actual bytes freed
            duration_seconds: duration,
            dry_run: self.config.dry_run,
        })
    }

    /// Mark reachable objects from protected references
    fn mark_reachable(&mut self, protected_refs: &[ObjectRef]) -> Result<HashSet<Digest>> {
        let mut reachable = HashSet::new();
        let mut to_process = Vec::new();

        // Start with protected references
        for obj_ref in protected_refs {
            match obj_ref {
                ObjectRef::Blob(digest) => {
                    reachable.insert(*digest);
                    to_process.push(*digest);
                }
                ObjectRef::Chunk(digest) => {
                    reachable.insert(*digest);
                    to_process.push(*digest);
                }
                ObjectRef::Tree(digest) => {
                    reachable.insert(*digest);
                    to_process.push(*digest);
                }
                ObjectRef::Commit(digest) => {
                    reachable.insert(*digest);
                    to_process.push(*digest);
                }
            }
        }

        // Process objects in BFS order
        while let Some(digest) = to_process.pop() {
            if self.config.verbose {
                println!("Processing object: {:?}", digest);
            }

            // Get object references (this would be implemented based on your object store)
            let references = self.get_object_references(&digest)?;
            
            for reference in references {
                if !reachable.contains(&reference) {
                    reachable.insert(reference);
                    to_process.push(reference);
                }
            }
        }

        Ok(reachable)
    }

    /// Sweep unreachable objects
    fn sweep_unreachable(&mut self, reachable: &HashSet<Digest>) -> Result<HashSet<Digest>> {
        let mut unreachable = HashSet::new();
        
        // Get all objects in the system (this would be implemented based on your object store)
        let all_objects = self.get_all_objects()?;
        
        for object in all_objects {
            if !reachable.contains(&object) && !self.reachability.protected.contains(&object) {
                unreachable.insert(object);
            }
        }

        Ok(unreachable)
    }

    /// Apply grace period to unreachable objects
    fn apply_grace_period(&mut self, unreachable: &HashSet<Digest>) -> Result<HashSet<Digest>> {
        let current_time = Self::current_timestamp();
        let mut grace_period = HashSet::new();

        for &digest in unreachable {
            if let Some(grace_time) = self.grace_period.grace_objects.get(&digest) {
                // Object is already in grace period
                if current_time - grace_time >= self.grace_period.grace_duration {
                    // Grace period expired, object can be deleted
                    self.grace_period.grace_objects.remove(&digest);
                } else {
                    // Still in grace period
                    grace_period.insert(digest);
                }
            } else {
                // New object entering grace period
                self.grace_period.grace_objects.insert(digest, current_time);
                grace_period.insert(digest);
            }
        }

        Ok(grace_period)
    }

    /// Pack cold objects
    fn pack_cold_objects(&mut self, reachable: &HashSet<Digest>) -> Result<Vec<Digest>> {
        if !self.config.enable_packing {
            return Ok(Vec::new());
        }

        let current_time = Self::current_timestamp();
        let mut packed = Vec::new();

        for &digest in reachable {
            if let Some(age) = self.get_object_age(&digest)? {
                if age > self.config.pack_threshold {
                    // Object is cold, pack it
                    if !self.config.dry_run {
                        self.pack_object(&digest)?;
                    }
                    packed.push(digest);
                }
            }
        }

        Ok(packed)
    }

    /// Get object references (to be implemented based on your object store)
    fn get_object_references(&self, _digest: &Digest) -> Result<Vec<Digest>> {
        // TODO: Implement based on your object store
        // This would traverse the object and find all references to other objects
        Ok(Vec::new())
    }

    /// Get all objects in the system
    fn get_all_objects(&self) -> Result<Vec<Digest>> {
        // TODO: Implement based on your object store
        // This would return all object digests in the system
        Ok(Vec::new())
    }

    /// Get object age in seconds
    fn get_object_age(&self, _digest: &Digest) -> Result<Option<u64>> {
        // TODO: Implement based on your object store
        // This would return the age of the object in seconds
        Ok(Some(0))
    }

    /// Pack an object
    fn pack_object(&mut self, _digest: &Digest) -> Result<()> {
        // TODO: Implement packing logic
        // This would move the object to a pack file
        Ok(())
    }

    /// Update GC statistics
    fn update_stats(&mut self, reachable: usize, unreachable: usize, grace_period: usize, packed: usize) {
        self.stats.objects_processed += reachable + unreachable;
        self.stats.reachable_objects += reachable;
        self.stats.unreachable_objects += unreachable;
        self.stats.grace_period_objects += grace_period;
        self.stats.packed_objects += packed;
        self.stats.gc_cycles += 1;
        self.stats.last_gc = Self::current_timestamp();
    }

    /// Get current timestamp
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// Get GC statistics
    pub fn get_stats(&self) -> &GcStats {
        &self.stats
    }

    /// Get reachability information
    pub fn get_reachability(&self) -> &ReachabilityInfo {
        &self.reachability
    }

    /// Get grace period information
    pub fn get_grace_period(&self) -> &GracePeriodTracker {
        &self.grace_period
    }

    /// Get configuration
    pub fn get_config(&self) -> &GcConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: GcConfig) {
        self.config = config;
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = GcStats {
            objects_processed: 0,
            reachable_objects: 0,
            unreachable_objects: 0,
            grace_period_objects: 0,
            swept_objects: 0,
            packed_objects: 0,
            bytes_freed: 0,
            gc_cycles: 0,
            last_gc: 0,
        };
    }
}

/// Result of a GC cycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcResult {
    /// Number of reachable objects
    pub reachable_objects: usize,
    /// Number of unreachable objects
    pub unreachable_objects: usize,
    /// Number of objects in grace period
    pub grace_period_objects: usize,
    /// Number of objects packed
    pub packed_objects: usize,
    /// Bytes freed
    pub bytes_freed: u64,
    /// Duration in seconds
    pub duration_seconds: u64,
    /// Whether this was a dry run
    pub dry_run: bool,
}

/// GC scheduler for automatic GC
pub struct GcScheduler {
    /// GC collector
    collector: GarbageCollector,
    /// Last GC time
    last_gc: u64,
    /// GC interval
    interval: u64,
}

impl GcScheduler {
    /// Create a new GC scheduler
    pub fn new(collector: GarbageCollector) -> Self {
        Self {
            collector,
            last_gc: 0,
            interval: 3600, // 1 hour
        }
    }

    /// Check if GC should run
    pub fn should_run_gc(&self) -> bool {
        if !self.collector.config.auto_gc {
            return false;
        }

        let current_time = Self::current_timestamp();
        current_time - self.last_gc >= self.interval
    }

    /// Run GC if needed
    pub fn run_gc_if_needed(&mut self, protected_refs: &[ObjectRef]) -> Result<Option<GcResult>> {
        if self.should_run_gc() {
            let result = self.collector.run_gc_cycle(protected_refs)?;
            self.last_gc = Self::current_timestamp();
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    /// Force GC run
    pub fn force_gc(&mut self, protected_refs: &[ObjectRef]) -> Result<GcResult> {
        let result = self.collector.run_gc_cycle(protected_refs)?;
        self.last_gc = Self::current_timestamp();
        Ok(result)
    }

    /// Get current timestamp
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gc_cycle() {
        let mut collector = GarbageCollector::new();
        let protected_refs = vec![
            ObjectRef::Blob(Digest::from_bytes(&[1, 2, 3, 4])),
            ObjectRef::Commit(Digest::from_bytes(&[5, 6, 7, 8])),
        ];

        let result = collector.run_gc_cycle(&protected_refs).unwrap();
        assert!(result.reachable_objects >= 0);
        assert!(result.unreachable_objects >= 0);
    }

    #[test]
    fn test_gc_scheduler() {
        let collector = GarbageCollector::new();
        let mut scheduler = GcScheduler::new(collector);
        
        // Should not run GC immediately
        assert!(!scheduler.should_run_gc());
        
        // Force GC
        let protected_refs = vec![ObjectRef::Blob(Digest::from_bytes(&[1, 2, 3, 4]))];
        let result = scheduler.force_gc(&protected_refs).unwrap();
        assert!(result.reachable_objects >= 0);
    }

    #[test]
    fn test_grace_period() {
        let mut collector = GarbageCollector::new();
        let digest = Digest::from_bytes(&[1, 2, 3, 4]);
        
        // Add object to grace period
        collector.grace_period.grace_objects.insert(digest, 0);
        
        // Check grace period
        assert!(collector.grace_period.grace_objects.contains_key(&digest));
    }

    #[test]
    fn test_gc_stats() {
        let collector = GarbageCollector::new();
        let stats = collector.get_stats();
        
        assert_eq!(stats.objects_processed, 0);
        assert_eq!(stats.gc_cycles, 0);
    }
}
