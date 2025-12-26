use crate::{Error, Hash, Object, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Object storage - content-addressable storage for all objects
pub struct ObjectStorage {
    objects_dir: PathBuf,
}

impl ObjectStorage {
    pub fn new(gvc_dir: &Path) -> Self {
        Self {
            objects_dir: gvc_dir.join("objects"),
        }
    }

    /// Initialize storage directory structure
    pub fn init(&self) -> Result<()> {
        fs::create_dir_all(&self.objects_dir)?;
        
        // Create subdirectories for first two hex chars (00-ff)
        // Improves filesystem performance with many objects
        for i in 0..256 {
            let subdir = self.objects_dir.join(format!("{:02x}", i));
            fs::create_dir_all(subdir)?;
        }
        
        Ok(())
    }

    /// Store an object and return its hash
    pub fn store(&self, object: &Object) -> Result<Hash> {
        let bytes = object.to_bytes()?;
        let hash = Hash::compute(&bytes);
        
        let path = self.object_path(&hash);
        
        // Only write if doesn't exist (content-addressable = idempotent)
        if !path.exists() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&path, bytes)?;
        }
        
        Ok(hash)
    }

    /// Load an object by hash
    pub fn load(&self, hash: &Hash) -> Result<Object> {
        let path = self.object_path(hash);
        
        if !path.exists() {
            return Err(Error::ObjectNotFound(hash.to_hex()));
        }
        
        let bytes = fs::read(&path)?;
        Object::from_bytes(&bytes)
    }

    /// Check if object exists
    pub fn exists(&self, hash: &Hash) -> bool {
        self.object_path(hash).exists()
    }

    /// Get path for object file
    /// Format: objects/ab/cdef123456...
    fn object_path(&self, hash: &Hash) -> PathBuf {
        let hex = hash.to_hex();
        let (prefix, suffix) = hex.split_at(2);
        self.objects_dir.join(prefix).join(suffix)
    }

    /// List all object hashes (useful for debugging/GC)
    pub fn list_objects(&self) -> Result<Vec<Hash>> {
        let mut objects = Vec::new();
        
        for entry in fs::read_dir(&self.objects_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                for obj_entry in fs::read_dir(&path)? {
                    let obj_entry = obj_entry?;
                    let obj_path = obj_entry.path();
                    
                    if obj_path.is_file() {
                        let prefix = path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("");
                        let suffix = obj_path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("");
                        
                        let hash_str = format!("{}{}", prefix, suffix);
                        if let Ok(hash) = Hash::from_hex(&hash_str) {
                            objects.push(hash);
                        }
                    }
                }
            }
        }
        
        Ok(objects)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Blob;
    use tempfile::TempDir;

    #[test]
    fn test_storage_init() {
        let temp = TempDir::new().unwrap();
        let storage = ObjectStorage::new(temp.path());
        storage.init().unwrap();
        
        assert!(temp.path().join("objects").exists());
        assert!(temp.path().join("objects/00").exists());
        assert!(temp.path().join("objects/ff").exists());
    }

    #[test]
    fn test_store_and_load() {
        let temp = TempDir::new().unwrap();
        let storage = ObjectStorage::new(temp.path());
        storage.init().unwrap();
        
        let blob = Blob::new(b"test content".to_vec());
        let obj = Object::Blob(blob);
        
        let hash = storage.store(&obj).unwrap();
        assert!(storage.exists(&hash));
        
        let loaded = storage.load(&hash).unwrap();
        assert_eq!(obj.object_type(), loaded.object_type());
    }
}

