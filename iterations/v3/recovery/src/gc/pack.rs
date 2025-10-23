use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

use crate::types::Digest;

/// Pack file format for storing multiple objects efficiently
pub struct PackFile {
    /// Path to the pack file
    path: PathBuf,
    /// Pack file header
    header: PackHeader,
    /// Object index (digest -> offset)
    object_index: HashMap<Digest, u64>,
    /// Pack file handle
    file: Option<File>,
}

/// Pack file header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackHeader {
    /// Magic number for pack files
    pub magic: [u8; 4],
    /// Version number
    pub version: u32,
    /// Number of objects in pack
    pub object_count: u32,
    /// Total size of pack file
    pub total_size: u64,
    /// Checksum of pack file
    pub checksum: Digest,
    /// Creation timestamp
    pub created_at: u64,
    /// Last modified timestamp
    pub modified_at: u64,
}

/// Pack index entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackIndexEntry {
    /// Object digest
    pub digest: Digest,
    /// Offset in pack file
    pub offset: u64,
    /// Object size
    pub size: u32,
    /// Object type
    pub object_type: ObjectType,
}

/// Object type in pack file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObjectType {
    /// Blob object
    Blob,
    /// Tree object
    Tree,
    /// Commit object
    Commit,
    /// Chunk object
    Chunk,
}

/// Pack index for fast object lookup
pub struct PackIndex {
    /// Path to the index file
    path: PathBuf,
    /// Index entries
    entries: Vec<PackIndexEntry>,
    /// Fan-out table for fast lookup
    fan_out: [u32; 256],
    /// Index file handle
    file: Option<File>,
}

/// Pack file manager
pub struct PackManager {
    /// Pack directory
    pack_dir: PathBuf,
    /// Index directory
    index_dir: PathBuf,
    /// Active pack files
    active_packs: HashMap<String, PackFile>,
    /// Pack index
    pack_index: PackIndex,
    /// Configuration
    config: PackConfig,
}

/// Pack configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackConfig {
    /// Maximum objects per pack file
    pub max_objects_per_pack: usize,
    /// Maximum pack file size (bytes)
    pub max_pack_size: u64,
    /// Enable compression
    pub enable_compression: bool,
    /// Compression level
    pub compression_level: u8,
    /// Enable fan-out index
    pub enable_fan_out: bool,
    /// Pack file prefix
    pub pack_prefix: String,
    /// Index file prefix
    pub index_prefix: String,
}

impl Default for PackConfig {
    fn default() -> Self {
        Self {
            max_objects_per_pack: 10000,
            max_pack_size: 1024 * 1024 * 1024, // 1GB
            enable_compression: true,
            compression_level: 4,
            enable_fan_out: true,
            pack_prefix: "pack-".to_string(),
            index_prefix: "pack-".to_string(),
        }
    }
}

impl PackFile {
    /// Create a new pack file
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            header: PackHeader {
                magic: *b"PACK",
                version: 1,
                object_count: 0,
                total_size: 0,
                checksum: Digest::from_bytes([0; 32]),
                created_at: 0,
                modified_at: 0,
            },
            object_index: HashMap::new(),
            file: None,
        }
    }

    /// Open an existing pack file
    pub fn open(path: PathBuf) -> Result<Self> {
        let mut file = File::open(&path)?;
        let header = Self::read_header(&mut file)?;
        
        Ok(Self {
            path,
            header,
            object_index: HashMap::new(),
            file: Some(file),
        })
    }

    /// Read pack file header
    fn read_header(file: &mut File) -> Result<PackHeader> {
        let mut magic = [0u8; 4];
        file.read_exact(&mut magic)?;
        
        if magic != *b"PACK" {
            return Err(anyhow!("Invalid pack file magic"));
        }

        let mut version = [0u8; 4];
        file.read_exact(&mut version)?;
        let version = u32::from_be_bytes(version);

        let mut object_count = [0u8; 4];
        file.read_exact(&mut object_count)?;
        let object_count = u32::from_be_bytes(object_count);

        let mut total_size = [0u8; 8];
        file.read_exact(&mut total_size)?;
        let total_size = u64::from_be_bytes(total_size);

        let mut checksum = [0u8; 32];
        file.read_exact(&mut checksum)?;
        let checksum = Digest::from_bytes(checksum);

        let mut created_at = [0u8; 8];
        file.read_exact(&mut created_at)?;
        let created_at = u64::from_be_bytes(created_at);

        let mut modified_at = [0u8; 8];
        file.read_exact(&mut modified_at)?;
        let modified_at = u64::from_be_bytes(modified_at);

        Ok(PackHeader {
            magic,
            version,
            object_count,
            total_size,
            checksum,
            created_at,
            modified_at,
        })
    }

    /// Write pack file header
    fn write_header(&self, file: &mut File) -> Result<()> {
        file.write_all(&self.header.magic)?;
        file.write_all(&self.header.version.to_be_bytes())?;
        file.write_all(&self.header.object_count.to_be_bytes())?;
        file.write_all(&self.header.total_size.to_be_bytes())?;
        file.write_all(self.header.checksum.as_bytes())?;
        file.write_all(&self.header.created_at.to_be_bytes())?;
        file.write_all(&self.header.modified_at.to_be_bytes())?;
        Ok(())
    }

    /// Write pack file header (internal method)
    fn write_header_internal(file: &mut File, header: &PackHeader) -> Result<()> {
        file.write_all(&header.magic)?;
        file.write_all(&header.version.to_be_bytes())?;
        file.write_all(&header.object_count.to_be_bytes())?;
        file.write_all(&header.total_size.to_be_bytes())?;
        file.write_all(header.checksum.as_bytes())?;
        file.write_all(&header.created_at.to_be_bytes())?;
        file.write_all(&header.modified_at.to_be_bytes())?;
        Ok(())
    }

    /// Add an object to the pack file
    pub fn add_object(&mut self, digest: Digest, data: &[u8], object_type: ObjectType) -> Result<()> {
        if let Some(file) = &mut self.file {
            let offset = file.seek(SeekFrom::End(0))?;
            
            // Write object header
            let object_header = ObjectHeader {
                digest,
                size: data.len() as u32,
                object_type,
                compressed: false, // TODO: Implement compression
            };
            
            Self::write_object_header_internal(file, &object_header)?;
            
            // Write object data
            file.write_all(data)?;
            
            // Update index
            self.object_index.insert(digest, offset);
            
            // Update header
            self.header.object_count += 1;
            self.header.total_size += data.len() as u64;
            self.header.modified_at = Self::current_timestamp();
            
            Ok(())
        } else {
            Err(anyhow!("Pack file not open"))
        }
    }

    /// Get an object from the pack file
    pub fn get_object(&mut self, digest: &Digest) -> Result<Option<Vec<u8>>> {
        if let Some(offset) = self.object_index.get(digest) {
            if let Some(file) = &mut self.file {
                file.seek(SeekFrom::Start(*offset))?;
                
                // Read object header
                let header = Self::read_object_header_internal(file)?;
                
                // Read object data
                let mut data = vec![0u8; header.size as usize];
                file.read_exact(&mut data)?;
                
                Ok(Some(data))
            } else {
                Err(anyhow!("Pack file not open"))
            }
        } else {
            Ok(None)
        }
    }

    /// Write object header (internal method)
    fn write_object_header_internal(file: &mut File, header: &ObjectHeader) -> Result<()> {
        // Write digest
        file.write_all(header.digest.as_bytes())?;
        
        // Write size (varint)
        Self::write_varint(file, header.size as u64)?;
        
        // Write object type
        file.write_all(&[header.object_type.clone() as u8])?;
        
        // Write compressed flag
        file.write_all(&[if header.compressed { 1 } else { 0 }])?;
        
        Ok(())
    }

    /// Read object header (internal method)
    fn read_object_header_internal(file: &mut File) -> Result<ObjectHeader> {
        // Read digest
        let mut digest_bytes = [0u8; 32];
        file.read_exact(&mut digest_bytes)?;
        let digest = Digest::from_bytes(digest_bytes);
        
        // Read size (varint)
        let size = Self::read_varint(file)?;
        
        // Read object type
        let mut object_type_byte = [0u8; 1];
        file.read_exact(&mut object_type_byte)?;
        let object_type = match object_type_byte[0] {
            0 => ObjectType::Blob,
            1 => ObjectType::Tree,
            2 => ObjectType::Commit,
            3 => ObjectType::Chunk,
            _ => return Err(anyhow!("Invalid object type")),
        };
        
        // Read compressed flag
        let mut compressed_byte = [0u8; 1];
        file.read_exact(&mut compressed_byte)?;
        let compressed = compressed_byte[0] != 0;
        
        Ok(ObjectHeader {
            digest,
            size: size as u32,
            object_type,
            compressed,
        })
    }

    /// Write varint
    fn write_varint(file: &mut File, mut value: u64) -> Result<()> {
        while value >= 0x80 {
            file.write_all(&[((value & 0x7f) | 0x80) as u8])?;
            value >>= 7;
        }
        file.write_all(&[value as u8])?;
        Ok(())
    }

    /// Read varint
    fn read_varint(file: &mut File) -> Result<u64> {
        let mut result = 0u64;
        let mut shift = 0;
        
        loop {
            let mut byte = [0u8; 1];
            file.read_exact(&mut byte)?;
            let b = byte[0];
            
            result |= ((b & 0x7f) as u64) << shift;
            
            if (b & 0x80) == 0 {
                break;
            }
            
            shift += 7;
            if shift >= 64 {
                return Err(anyhow!("Varint too long"));
            }
        }
        
        Ok(result)
    }

    /// Get current timestamp
    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// Close the pack file
    pub fn close(&mut self) -> Result<()> {
        if let Some(file) = &mut self.file {
            // Update header with final values
            let header = self.header.clone();
            Self::write_header_internal(file, &header)?;
            file.sync_all()?;
        }
        self.file = None;
        Ok(())
    }
}

/// Object header in pack file
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ObjectHeader {
    digest: Digest,
    size: u32,
    object_type: ObjectType,
    compressed: bool,
}

impl PackIndex {
    /// Create a new pack index
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            entries: Vec::new(),
            fan_out: [0; 256],
            file: None,
        }
    }

    /// Load pack index from file
    pub fn load(path: PathBuf) -> Result<Self> {
        let mut file = File::open(&path)?;
        let mut reader = BufReader::new(&mut file);
        
        // Read fan-out table
        let mut fan_out = [0u32; 256];
        for i in 0..256 {
            let mut bytes = [0u8; 4];
            reader.read_exact(&mut bytes)?;
            fan_out[i] = u32::from_be_bytes(bytes);
        }
        
        // Read entries
        let mut entries = Vec::new();
        let entry_count = fan_out[255];
        
        for _ in 0..entry_count {
            let mut digest_bytes = [0u8; 32];
            reader.read_exact(&mut digest_bytes)?;
            let digest = Digest::from_bytes(digest_bytes);
            
            let mut offset_bytes = [0u8; 8];
            reader.read_exact(&mut offset_bytes)?;
            let offset = u64::from_be_bytes(offset_bytes);
            
            let mut size_bytes = [0u8; 4];
            reader.read_exact(&mut size_bytes)?;
            let size = u32::from_be_bytes(size_bytes);
            
            let mut type_byte = [0u8; 1];
            reader.read_exact(&mut type_byte)?;
            let object_type = match type_byte[0] {
                0 => ObjectType::Blob,
                1 => ObjectType::Tree,
                2 => ObjectType::Commit,
                3 => ObjectType::Chunk,
                _ => return Err(anyhow!("Invalid object type in index")),
            };
            
            entries.push(PackIndexEntry {
                digest,
                offset,
                size,
                object_type,
            });
        }
        
        Ok(Self {
            path,
            entries,
            fan_out,
            file: Some(file),
        })
    }

    /// Save pack index to file
    pub fn save(&self) -> Result<()> {
        let mut file = File::create(&self.path)?;
        let mut writer = BufWriter::new(&mut file);
        
        // Write fan-out table
        for &count in &self.fan_out {
            writer.write_all(&count.to_be_bytes())?;
        }
        
        // Write entries
        for entry in &self.entries {
            writer.write_all(entry.digest.as_bytes())?;
            writer.write_all(&entry.offset.to_be_bytes())?;
            writer.write_all(&entry.size.to_be_bytes())?;
            writer.write_all(&[entry.object_type.clone() as u8])?;
        }
        
        writer.flush()?;
        drop(writer); // Ensure writer is dropped before sync
        file.sync_all()?;
        Ok(())
    }

    /// Find object in index
    pub fn find_object(&self, digest: &Digest) -> Option<&PackIndexEntry> {
        self.entries.iter().find(|entry| entry.digest == *digest)
    }

    /// Add entry to index
    pub fn add_entry(&mut self, entry: PackIndexEntry) {
        self.entries.push(entry);
        self.update_fan_out();
    }

    /// Update fan-out table
    fn update_fan_out(&mut self) {
        self.fan_out.fill(0);
        
        for entry in &self.entries {
            let first_byte = entry.digest.as_bytes()[0];
            self.fan_out[first_byte as usize] += 1;
        }
        
        // Convert to cumulative counts
        for i in 1..256 {
            self.fan_out[i] += self.fan_out[i - 1];
        }
    }
}

impl PackManager {
    /// Create a new pack manager
    pub fn new(pack_dir: PathBuf, index_dir: PathBuf) -> Self {
        Self {
            pack_dir,
            index_dir: index_dir.clone(),
            active_packs: HashMap::new(),
            pack_index: PackIndex::new(index_dir.clone().join("pack.idx")),
            config: PackConfig::default(),
        }
    }

    /// Create a new pack file
    pub fn create_pack(&mut self, pack_id: String) -> Result<&mut PackFile> {
        let pack_path = self.pack_dir.join(format!("{}{}.pack", self.config.pack_prefix, pack_id));
        let mut pack_file = PackFile::new(pack_path);
        
        // Open pack file for writing
        let file = File::create(&pack_file.path)?;
        pack_file.file = Some(file);
        
        // Initialize header
        pack_file.header.created_at = Self::current_timestamp();
        pack_file.header.modified_at = pack_file.header.created_at;
        
        self.active_packs.insert(pack_id.clone(), pack_file);
        Ok(self.active_packs.get_mut(&pack_id).unwrap())
    }

    /// Get current timestamp
    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// Close all active packs
    pub fn close_all_packs(&mut self) -> Result<()> {
        for pack in self.active_packs.values_mut() {
            pack.close()?;
        }
        self.active_packs.clear();
        Ok(())
    }

    /// Get pack statistics
    pub fn get_stats(&self) -> PackStats {
        PackStats {
            active_packs: self.active_packs.len(),
            total_objects: self.pack_index.entries.len(),
            total_size: self.pack_index.entries.iter().map(|e| e.size as u64).sum(),
        }
    }
}

/// Pack statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackStats {
    /// Number of active pack files
    pub active_packs: usize,
    /// Total number of objects
    pub total_objects: usize,
    /// Total size in bytes
    pub total_size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_pack_file_creation() {
        let temp_dir = TempDir::new().unwrap();
        let pack_path = temp_dir.path().join("test.pack");
        let mut pack_file = PackFile::new(pack_path);
        
        assert_eq!(pack_file.header.magic, *b"PACK");
        assert_eq!(pack_file.header.version, 1);
        assert_eq!(pack_file.header.object_count, 0);
    }

    #[test]
    fn test_pack_index() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().join("test.idx");
        let mut pack_index = PackIndex::new(index_path);
        
        let entry = PackIndexEntry {
            digest: Digest::from_bytes([12; 32]),
            offset: 0,
            size: 100,
            object_type: ObjectType::Blob,
        };
        
        pack_index.add_entry(entry);
        assert_eq!(pack_index.entries.len(), 1);
    }

    #[test]
    fn test_pack_manager() {
        let temp_dir = TempDir::new().unwrap();
        let pack_dir = temp_dir.path().join("packs");
        let index_dir = temp_dir.path().join("index");
        
        std::fs::create_dir_all(&pack_dir).unwrap();
        std::fs::create_dir_all(&index_dir).unwrap();
        
        let mut pack_manager = PackManager::new(pack_dir, index_dir);
        let stats = pack_manager.get_stats();
        
        assert_eq!(stats.active_packs, 0);
        assert_eq!(stats.total_objects, 0);
    }
}
