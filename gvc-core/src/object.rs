use crate::{Error, Hash, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Object types in GVC
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectType {
    Blob,
    Tree,
    Commit,
    Tag,
}

impl ObjectType {
    pub fn as_str(&self) -> &str {
        match self {
            ObjectType::Blob => "blob",
            ObjectType::Tree => "tree",
            ObjectType::Commit => "commit",
            ObjectType::Tag => "tag",
        }
    }

    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "blob" => Ok(ObjectType::Blob),
            "tree" => Ok(ObjectType::Tree),
            "commit" => Ok(ObjectType::Commit),
            "tag" => Ok(ObjectType::Tag),
            _ => Err(Error::InvalidObjectType(s.to_string())),
        }
    }
}

/// Blob - stores file content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blob {
    pub data: Vec<u8>,
}

impl Blob {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }
}

/// Tree entry - represents a file or directory in a tree
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TreeEntry {
    pub mode: u32,      // File permissions (e.g., 100644 for regular file, 040000 for directory)
    pub name: String,   // File/directory name
    pub hash: Hash,     // Hash of the blob or tree
    pub obj_type: ObjectType,
}

impl TreeEntry {
    pub fn new_file(name: String, hash: Hash) -> Self {
        Self {
            mode: 0o100644,
            name,
            hash,
            obj_type: ObjectType::Blob,
        }
    }

    pub fn new_dir(name: String, hash: Hash) -> Self {
        Self {
            mode: 0o040000,
            name,
            hash,
            obj_type: ObjectType::Tree,
        }
    }

    pub fn is_file(&self) -> bool {
        self.obj_type == ObjectType::Blob
    }

    pub fn is_dir(&self) -> bool {
        self.obj_type == ObjectType::Tree
    }
}

/// Tree - represents directory structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tree {
    /// Entries sorted by name for deterministic hashing
    pub entries: BTreeMap<String, TreeEntry>,
}

impl Tree {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    pub fn add_entry(&mut self, entry: TreeEntry) {
        self.entries.insert(entry.name.clone(), entry);
    }

    pub fn get_entry(&self, name: &str) -> Option<&TreeEntry> {
        self.entries.get(name)
    }

    pub fn remove_entry(&mut self, name: &str) -> Option<TreeEntry> {
        self.entries.remove(name)
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for Tree {
    fn default() -> Self {
        Self::new()
    }
}

/// Commit - represents a snapshot with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub tree: Hash,                 // Root tree hash
    pub parents: Vec<Hash>,         // Parent commit(s)
    pub author: String,             // Author name
    pub committer: String,          // Committer name
    pub timestamp: i64,             // Unix timestamp
    pub message: String,            // Commit message
}

impl Commit {
    pub fn new(
        tree: Hash,
        parents: Vec<Hash>,
        author: String,
        message: String,
    ) -> Self {
        let timestamp = chrono::Utc::now().timestamp();
        Self {
            tree,
            parents,
            author: author.clone(),
            committer: author,
            timestamp,
            message,
        }
    }

    pub fn is_root(&self) -> bool {
        self.parents.is_empty()
    }
}

/// Generic object wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Object {
    Blob(Blob),
    Tree(Tree),
    Commit(Commit),
}

impl Object {
    pub fn object_type(&self) -> ObjectType {
        match self {
            Object::Blob(_) => ObjectType::Blob,
            Object::Tree(_) => ObjectType::Tree,
            Object::Commit(_) => ObjectType::Commit,
        }
    }

    /// Serialize object to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        bincode::serialize(self)
            .map_err(|e| Error::Serialization(format!("Failed to serialize: {}", e)))
    }

    /// Deserialize object from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        bincode::deserialize(data)
            .map_err(|e| Error::Serialization(format!("Failed to deserialize: {}", e)))
    }

    /// Compute hash of this object
    pub fn hash(&self) -> Result<Hash> {
        let bytes = self.to_bytes()?;
        Ok(Hash::compute(&bytes))
    }

    pub fn as_blob(&self) -> Option<&Blob> {
        match self {
            Object::Blob(b) => Some(b),
            _ => None,
        }
    }

    pub fn as_tree(&self) -> Option<&Tree> {
        match self {
            Object::Tree(t) => Some(t),
            _ => None,
        }
    }

    pub fn as_commit(&self) -> Option<&Commit> {
        match self {
            Object::Commit(c) => Some(c),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blob_creation() {
        let blob = Blob::new(b"hello world".to_vec());
        assert_eq!(blob.size(), 11);
    }

    #[test]
    fn test_tree_operations() {
        let mut tree = Tree::new();
        let hash = Hash::compute(b"test");
        let entry = TreeEntry::new_file("test.txt".to_string(), hash);
        
        tree.add_entry(entry.clone());
        assert_eq!(tree.entries.len(), 1);
        assert!(tree.get_entry("test.txt").is_some());
    }

    #[test]
    fn test_object_serialization() {
        let blob = Blob::new(b"test data".to_vec());
        let obj = Object::Blob(blob);
        
        let bytes = obj.to_bytes().unwrap();
        let deserialized = Object::from_bytes(&bytes).unwrap();
        
        assert_eq!(obj.object_type(), deserialized.object_type());
    }
}

