use crate::{Error, Hash, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Index entry - represents a staged file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexEntry {
    pub path: PathBuf,
    pub hash: Hash,
    pub size: u64,
    pub mtime: i64,
}

/// Index (staging area) - tracks files to be committed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Index {
    /// Map from file path to index entry
    entries: BTreeMap<PathBuf, IndexEntry>,
}

impl Index {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    /// Load index from file
    pub fn load(gvc_dir: &Path) -> Result<Self> {
        let index_path = gvc_dir.join("index");

        if !index_path.exists() {
            return Ok(Self::new());
        }

        let data = fs::read(&index_path)?;
        bincode::deserialize(&data)
            .map_err(|e| Error::IndexError(format!("Failed to deserialize index: {}", e)))
    }

    /// Save index to file
    pub fn save(&self, gvc_dir: &Path) -> Result<()> {
        let index_path = gvc_dir.join("index");
        let data = bincode::serialize(self)
            .map_err(|e| Error::IndexError(format!("Failed to serialize index: {}", e)))?;

        fs::write(&index_path, data)?;
        Ok(())
    }

    /// Add or update entry
    pub fn add_entry(&mut self, entry: IndexEntry) {
        self.entries.insert(entry.path.clone(), entry);
    }

    /// Remove entry
    pub fn remove_entry(&mut self, path: &Path) -> Option<IndexEntry> {
        self.entries.remove(path)
    }

    /// Get entry
    pub fn get_entry(&self, path: &Path) -> Option<&IndexEntry> {
        self.entries.get(path)
    }

    /// Get all entries
    pub fn entries(&self) -> &BTreeMap<PathBuf, IndexEntry> {
        &self.entries
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Check if index is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

impl Default for Index {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_index_operations() {
        let mut index = Index::new();
        let hash = Hash::compute(b"test");

        let entry = IndexEntry {
            path: PathBuf::from("test.txt"),
            hash,
            size: 100,
            mtime: 12345,
        };

        index.add_entry(entry.clone());
        assert_eq!(index.len(), 1);
        assert!(index.get_entry(&PathBuf::from("test.txt")).is_some());
    }

    #[test]
    fn test_index_persistence() {
        let temp = TempDir::new().unwrap();
        let mut index = Index::new();

        let entry = IndexEntry {
            path: PathBuf::from("test.txt"),
            hash: Hash::compute(b"test"),
            size: 100,
            mtime: 12345,
        };

        index.add_entry(entry);
        index.save(temp.path()).unwrap();

        let loaded = Index::load(temp.path()).unwrap();
        assert_eq!(loaded.len(), 1);
    }
}
