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
    use crate::{Blob, Tree, TreeEntry, Commit};
    use tempfile::TempDir;

    #[test]
    fn test_storage_init() {
        let temp = TempDir::new().unwrap();
        let storage = ObjectStorage::new(temp.path());
        storage.init().unwrap();
        
        assert!(temp.path().join("objects").exists());
        assert!(temp.path().join("objects/00").exists());
        assert!(temp.path().join("objects/ff").exists());
        assert!(temp.path().join("objects/ab").exists());
    }

    #[test]
    fn test_store_and_load_blob() {
        let temp = TempDir::new().unwrap();
        let storage = ObjectStorage::new(temp.path());
        storage.init().unwrap();
        
        let blob = Blob::new(b"test content".to_vec());
        let obj = Object::Blob(blob.clone());
        
        let hash = storage.store(&obj).unwrap();
        assert!(storage.exists(&hash));
        
        let loaded = storage.load(&hash).unwrap();
        assert_eq!(obj.object_type(), loaded.object_type());
        assert_eq!(loaded.as_blob().unwrap().data, blob.data);
    }

    #[test]
    fn test_store_and_load_tree() {
        let temp = TempDir::new().unwrap();
        let storage = ObjectStorage::new(temp.path());
        storage.init().unwrap();
        
        let mut tree = Tree::new();
        tree.add_entry(TreeEntry::new_file("file.txt".to_string(), Hash::compute(b"test")));
        let obj = Object::Tree(tree);
        
        let hash = storage.store(&obj).unwrap();
        let loaded = storage.load(&hash).unwrap();
        
        assert_eq!(obj.object_type(), loaded.object_type());
        assert_eq!(loaded.as_tree().unwrap().entries.len(), 1);
    }

    #[test]
    fn test_store_and_load_commit() {
        let temp = TempDir::new().unwrap();
        let storage = ObjectStorage::new(temp.path());
        storage.init().unwrap();
        
        let commit = Commit::new(
            Hash::compute(b"tree"),
            vec![],
            "Author".to_string(),
            "Message".to_string(),
        );
        let obj = Object::Commit(commit);
        
        let hash = storage.store(&obj).unwrap();
        let loaded = storage.load(&hash).unwrap();
        
        assert_eq!(obj.object_type(), loaded.object_type());
        assert_eq!(loaded.as_commit().unwrap().message, "Message");
    }

    #[test]
    fn test_store_idempotent() {
        let temp = TempDir::new().unwrap();
        let storage = ObjectStorage::new(temp.path());
        storage.init().unwrap();
        
        let blob = Blob::new(b"test".to_vec());
        let obj = Object::Blob(blob);
        
        let hash1 = storage.store(&obj).unwrap();
        let hash2 = storage.store(&obj).unwrap();
        
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_exists() {
        let temp = TempDir::new().unwrap();
        let storage = ObjectStorage::new(temp.path());
        storage.init().unwrap();
        
        let fake_hash = Hash::compute(b"nonexistent");
        assert!(!storage.exists(&fake_hash));
        
        let blob = Blob::new(b"test".to_vec());
        let obj = Object::Blob(blob);
        let hash = storage.store(&obj).unwrap();
        
        assert!(storage.exists(&hash));
    }

    #[test]
    fn test_load_nonexistent() {
        let temp = TempDir::new().unwrap();
        let storage = ObjectStorage::new(temp.path());
        storage.init().unwrap();
        
        let fake_hash = Hash::compute(b"nonexistent");
        let result = storage.load(&fake_hash);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_list_objects() {
        let temp = TempDir::new().unwrap();
        let storage = ObjectStorage::new(temp.path());
        storage.init().unwrap();
        
        // Initially empty
        let objects = storage.list_objects().unwrap();
        assert_eq!(objects.len(), 0);
        
        // Add some objects
        let blob1 = Object::Blob(Blob::new(b"test1".to_vec()));
        let blob2 = Object::Blob(Blob::new(b"test2".to_vec()));
        let blob3 = Object::Blob(Blob::new(b"test3".to_vec()));
        
        let hash1 = storage.store(&blob1).unwrap();
        let hash2 = storage.store(&blob2).unwrap();
        let hash3 = storage.store(&blob3).unwrap();
        
        let objects = storage.list_objects().unwrap();
        assert_eq!(objects.len(), 3);
        assert!(objects.contains(&hash1));
        assert!(objects.contains(&hash2));
        assert!(objects.contains(&hash3));
    }

    #[test]
    fn test_object_path() {
        let temp = TempDir::new().unwrap();
        let storage = ObjectStorage::new(temp.path());
        
        let hash = Hash::compute(b"test");
        let hex = hash.to_hex();
        let expected_prefix = &hex[0..2];
        let expected_suffix = &hex[2..];
        
        let path = storage.object_path(&hash);
        
        assert!(path.to_string_lossy().contains(expected_prefix));
        assert!(path.to_string_lossy().contains(expected_suffix));
    }
}

