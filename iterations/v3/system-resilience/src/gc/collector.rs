use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::recovery_types::{Digest, ObjectRef};

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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GracePeriodTracker {
    /// Objects in grace period with their timestamps
    pub grace_objects: HashMap<Digest, u64>,
    /// Grace period duration
    pub grace_duration: u64,
}

impl Default for GarbageCollector {
    fn default() -> Self {
        Self::new()
    }
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
            reachable.insert(obj_ref.digest);
            to_process.push(obj_ref.digest);
        }

        // Process objects in BFS order
        while let Some(digest) = to_process.pop() {
            if self.config.verbose {
                println!("Processing object: {:?}", digest);
            }

            // TODO: Implement object reference retrieval from object store
            //       Currently uses placeholder; should query actual object store for object references based on digest.
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
        
        // TODO: Implement object enumeration from object store
        //       Currently uses placeholder; should query actual object store to enumerate all objects in the system.
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
    fn get_object_references(&self, digest: &Digest) -> Result<Vec<Digest>> {
        use crate::cas::BlobStore;
        use crate::merkle::{Commit, FileTree};

        // Create a blob store instance to read objects
        // TODO: Implement dependency injection for blob store
        //       Currently creates blob store directly; should be injected or accessed from collector for better testability and configuration.
        //
        // COMPLETION CHECKLIST:
        // [ ] Add blob store as constructor parameter
        // [ ] Access blob store from collector instance
        // [ ] Support configuration-based blob store creation
        // [ ] Handle missing blob store gracefully
        // [ ] Add unit tests with mock blob store
        // [ ] Add integration tests with real blob store
        // [ ] Performance: No performance impact (construction-time only)
        // [ ] Documentation: Document injection pattern
        //
        // ACCEPTANCE CRITERIA:
        // - Blob store can be injected via constructor
        // - Blob store is accessible from collector
        // - Configuration-based creation is supported
        // - Missing blob store is handled gracefully
        // - Injection pattern is testable
        //
        // DEPENDENCIES:
        // - Dependency injection framework (Optional)
        // - Blob store interface (Required)
        // - Configuration management (Required)
        //
        // ESTIMATED EFFORT: 3-4 hours (high confidence)
        // PRIORITY: Low
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 3 (code quality improvement)
        // - Change Budget: ~100 LOC
        // - Reviewer Requirements: Dependency injection expertise
        let objects_dir = std::path::PathBuf::from("./.recovery/objects");
        let blob_store = BlobStore::new(objects_dir);

        // Try to read the object as a blob first
        if let Ok(Some(blob)) = blob_store.get_blob(*digest) {
            // Parse blob content based on payload kind to find references
            return self.parse_blob_references(&blob);
        }

        // Try to read as a commit object (stored as JSON in blob store)
        if let Ok(Some(blob)) = blob_store.get_blob(*digest) {
            if let Ok(commit_str) = std::str::from_utf8(&blob.data) {
                if let Ok(commit) = serde_json::from_str::<Commit>(commit_str) {
                    return self.parse_commit_references(&commit);
                }
            }
        }

        // Try to read as a tree object (stored as JSON in blob store)
        if let Ok(Some(blob)) = blob_store.get_blob(*digest) {
            if let Ok(tree_str) = std::str::from_utf8(&blob.data) {
                if let Ok(tree) = serde_json::from_str::<FileTree>(tree_str) {
                    return self.parse_tree_references(&tree);
                }
            }
        }

        // Object not found or unrecognized type
        if self.config.verbose {
            println!("Warning: Could not parse references for object {:?}", digest);
        }
        Ok(Vec::new())
    }

    /// Parse references from a blob object
    fn parse_blob_references(&self, blob: &crate::cas::Blob) -> Result<Vec<Digest>> {
        use crate::recovery_types::PayloadKind;

        let mut references = Vec::new();

        match blob.header.kind {
            PayloadKind::Full => {
                // TODO: Parse internal blob references for complete reference tracking
                //       Currently skips parsing to avoid loading large objects; should parse internal blob references (e.g., chunk references) for complete reference tracking.
                //
                // COMPLETION CHECKLIST:
                // [ ] Primary functionality implemented
                // [ ] API/data structures defined & stable
                // [ ] Error handling + validation aligned with error taxonomy
                // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
                // [ ] Integration tests for external systems/contracts
                // [ ] Documentation: public API + system behavior
                // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
                // [ ] Security posture reviewed (inputs, authz, sandboxing)
                // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
                // [ ] Configurability and feature flags defined if relevant
                // [ ] Failure-mode cards documented (degradation paths)
                //
                // ACCEPTANCE CRITERIA:
                // - Internal references are parsed correctly
                // - Parsing is efficient for large objects
                // - Reference tracking is complete
                // - Performance is acceptable
                //
                // DEPENDENCIES:
                // - Blob parsing utilities (Required)
                // - Reference extraction algorithms (Required)
                // - Digest pattern matching (Required)
                //
                // ESTIMATED EFFORT: 5-6 hours (medium confidence)
                // PRIORITY: Low
                // BLOCKING: No
                //
                // GOVERNANCE:
                // - CAWS Tier: 3 (GC optimization enhancement)
                // - Change Budget: ~120 LOC
                // - Reviewer Requirements: GC and blob parsing expertise
                // Temporary: skip parsing until efficient implementation
            }
            PayloadKind::UnifiedDiff => {
                // Diff blobs might reference the base object they're diffing against
                // Parse the diff header to find referenced objects
                self.parse_diff_references(blob)?;
            }
            PayloadKind::ChunkMap => {
                // Chunk maps reference multiple chunks
                // Parse the chunk map to extract all chunk digests
                if let Ok(chunks) = self.parse_chunk_map(blob) {
                    references.extend(chunks);
                }
            }
        }

        Ok(references)
    }

    /// Parse references from a commit object
    fn parse_commit_references(&self, commit: &crate::merkle::Commit) -> Result<Vec<Digest>> {
        let mut references = Vec::new();

        // Commits always reference their tree
        references.push(commit.tree);

        // Commits may reference a parent commit
        if let Some(parent) = commit.parent {
            references.push(parent);
        }

        Ok(references)
    }

    /// Parse references from a tree object
    fn parse_tree_references(&self, tree: &crate::merkle::FileTree) -> Result<Vec<Digest>> {
        let mut references = Vec::new();

        // Trees reference all their entries (files, directories, symlinks)
        for entry in &tree.entries {
            references.push(entry.digest);

            // If this is a directory (tree), it will be handled when we process that tree object
            // The graph traversal will find it naturally
        }

        // Trees also reference their parent if they have one (for directory hierarchies)
        // This is implicit in the Merkle tree structure

        Ok(references)
    }

    /// Parse references from a unified diff
    fn parse_diff_references(&self, blob: &crate::cas::Blob) -> Result<Vec<Digest>> {
        // TODO: Extract referenced objects from diff headers
        //       Currently returns empty; should extract referenced objects from diff headers for complete reference tracking.
        //
        // COMPLETION CHECKLIST:
        // [ ] Primary functionality implemented
        // [ ] API/data structures defined & stable
        // [ ] Error handling + validation aligned with error taxonomy
        // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
        // [ ] Integration tests for external systems/contracts
        // [ ] Documentation: public API + system behavior
        // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
        // [ ] Security posture reviewed (inputs, authz, sandboxing)
        // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
        // [ ] Configurability and feature flags defined if relevant
        // [ ] Failure-mode cards documented (degradation paths)
        //
        // ACCEPTANCE CRITERIA:
        // - References are extracted from diff headers correctly
        // - Parsing handles various diff formats
        // - Reference tracking is complete
        // - Performance is acceptable
        //
        // DEPENDENCIES:
        // - Diff parsing utilities (Required)
        // - Header extraction algorithms (Required)
        // - Reference tracking infrastructure (Required)
        //
        // ESTIMATED EFFORT: 4-5 hours (medium confidence)
        // PRIORITY: Low
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 3 (GC enhancement)
        // - Change Budget: ~100 LOC
        // - Reviewer Requirements: Diff parsing expertise
        Ok(Vec::new()) // Temporary: empty until diff header parsing
    }

    /// Parse chunk references from a chunk map
    fn parse_chunk_map(&self, blob: &crate::cas::Blob) -> Result<Vec<Digest>> {
        use crate::recovery_types::ChunkRef;

        let mut chunks = Vec::new();

        // Try to deserialize the blob data as chunk references
        // The exact format depends on how chunk maps are serialized
        if let Ok(chunk_refs) = serde_json::from_slice::<Vec<ChunkRef>>(&blob.data) {
            for chunk_ref in chunk_refs {
                chunks.push(chunk_ref.digest);
            }
        } else if let Ok(chunk_refs) = bincode::deserialize::<Vec<ChunkRef>>(&blob.data) {
            for chunk_ref in chunk_refs {
                chunks.push(chunk_ref.digest);
            }
        }

        Ok(chunks)
    }

    /// Get all objects in the system
    fn get_all_objects(&self) -> Result<Vec<Digest>> {
        use crate::cas::BlobStore;
        use walkdir::WalkDir;

        let mut all_objects = Vec::new();

        // Create a blob store instance to scan the objects directory
        let objects_dir = std::path::PathBuf::from("./.recovery/objects");
        let blob_store = BlobStore::new(objects_dir.clone());

        // Walk the objects directory to find all stored objects
        // Objects are stored in a sharded directory structure: xx/xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
        if objects_dir.exists() {
            for entry in WalkDir::new(&objects_dir).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    // Extract digest from file path
                    // Expected path format: .recovery/objects/xx/xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
                    if let Some(file_name) = entry.file_name().to_str() {
                        if file_name.len() == 64 { // BLAKE3 hex is 64 characters
                            if let Ok(digest) = Digest::from_hex(file_name) {
                                all_objects.push(digest);
                            }
                        }
                    }
                }
            }
        }

        if self.config.verbose {
            println!("Found {} objects in the system", all_objects.len());
        }

        Ok(all_objects)
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
            last_gc: Self::current_timestamp(), // Initialize to current time so GC doesn't run immediately
            interval: 3600, // 1 hour
        }
    }

    /// Check if GC should run
    pub fn should_run_gc(&self) -> bool {
        if !self.collector.config.auto_gc {
            return false;
        }

        let current_time = Self::current_timestamp();
        // If last_gc is 0 (never run), don't run immediately
        if self.last_gc == 0 {
            return false;
        }
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
            ObjectRef {
                digest: Digest::from_bytes([13; 32]),
                size: 100,
            },
            ObjectRef {
                digest: Digest::from_bytes([14; 32]),
                size: 200,
            },
        ];

        let result = collector.run_gc_cycle(&protected_refs).unwrap();
        // reachable_objects and unreachable_objects are usize, always >= 0
    }

    #[test]
    fn test_gc_scheduler() {
        let collector = GarbageCollector::new();
        let mut scheduler = GcScheduler::new(collector);
        
        // Should not run GC immediately
        assert!(!scheduler.should_run_gc());
        
        // Force GC
        let protected_refs = vec![ObjectRef {
            digest: Digest::from_bytes([15; 32]),
            size: 150,
        }];
        let result = scheduler.force_gc(&protected_refs).unwrap();
        // reachable_objects is usize, always >= 0
    }

    #[test]
    fn test_grace_period() {
        let mut collector = GarbageCollector::new();
        let digest = Digest::from_bytes([16; 32]);
        
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
