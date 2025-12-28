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
        assert_eq!(blob.data, b"hello world");
    }

    #[test]
    fn test_blob_empty() {
        let blob = Blob::new(vec![]);
        assert_eq!(blob.size(), 0);
    }

    #[test]
    fn test_tree_creation() {
        let tree = Tree::new();
        assert!(tree.entries.is_empty());
    }

    #[test]
    fn test_tree_add_entry() {
        let mut tree = Tree::new();
        let hash = Hash::compute(b"test");
        let entry = TreeEntry::new_file("test.txt".to_string(), hash.clone());
        
        tree.add_entry(entry);
        assert_eq!(tree.entries.len(), 1);
        assert!(tree.get_entry("test.txt").is_some());
        assert_eq!(tree.get_entry("test.txt").unwrap().hash, hash);
    }

    #[test]
    fn test_tree_multiple_entries() {
        let mut tree = Tree::new();
        tree.add_entry(TreeEntry::new_file("file1.txt".to_string(), Hash::compute(b"1")));
        tree.add_entry(TreeEntry::new_file("file2.txt".to_string(), Hash::compute(b"2")));
        tree.add_entry(TreeEntry::new_dir("dir1".to_string(), Hash::compute(b"3")));
        
        assert_eq!(tree.entries.len(), 3);
        assert!(tree.get_entry("file1.txt").is_some());
        assert!(tree.get_entry("file2.txt").is_some());
        assert!(tree.get_entry("dir1").is_some());
    }

    #[test]
    fn test_tree_entry_types() {
        let file_entry = TreeEntry::new_file("file.txt".to_string(), Hash::compute(b"test"));
        let dir_entry = TreeEntry::new_dir("dir".to_string(), Hash::compute(b"test"));
        
        assert!(file_entry.is_file());
        assert!(!file_entry.is_dir());
        assert!(dir_entry.is_dir());
        assert!(!dir_entry.is_file());
    }

    #[test]
    fn test_commit_creation() {
        let tree_hash = Hash::compute(b"tree");
        let commit = Commit::new(
            tree_hash.clone(),
            vec![],
            "Test Author".to_string(),
            "Test message".to_string(),
        );
        
        assert_eq!(commit.tree, tree_hash);
        assert_eq!(commit.author, "Test Author");
        assert_eq!(commit.message, "Test message");
        assert!(commit.is_root());
    }

    #[test]
    fn test_commit_with_parents() {
        let tree_hash = Hash::compute(b"tree");
        let parent_hash = Hash::compute(b"parent");
        let commit = Commit::new(
            tree_hash,
            vec![parent_hash],
            "Author".to_string(),
            "Message".to_string(),
        );
        
        assert!(!commit.is_root());
        assert_eq!(commit.parents.len(), 1);
    }

    #[test]
    fn test_object_type_blob() {
        let blob = Blob::new(vec![1, 2, 3]);
        let obj = Object::Blob(blob);
        assert_eq!(obj.object_type(), ObjectType::Blob);
    }

    #[test]
    fn test_object_type_tree() {
        let tree = Tree::new();
        let obj = Object::Tree(tree);
        assert_eq!(obj.object_type(), ObjectType::Tree);
    }

    #[test]
    fn test_object_type_commit() {
        let commit = Commit::new(
            Hash::compute(b"tree"),
            vec![],
            "Author".to_string(),
            "Message".to_string(),
        );
        let obj = Object::Commit(commit);
        assert_eq!(obj.object_type(), ObjectType::Commit);
    }

    #[test]
    fn test_object_serialization_blob() {
        let blob = Blob::new(b"test data".to_vec());
        let obj = Object::Blob(blob);
        
        let bytes = obj.to_bytes().unwrap();
        let deserialized = Object::from_bytes(&bytes).unwrap();
        
        assert_eq!(obj.object_type(), deserialized.object_type());
        assert_eq!(obj.as_blob().unwrap().data, deserialized.as_blob().unwrap().data);
    }

    #[test]
    fn test_object_serialization_tree() {
        let mut tree = Tree::new();
        tree.add_entry(TreeEntry::new_file("file.txt".to_string(), Hash::compute(b"test")));
        let obj = Object::Tree(tree);
        
        let bytes = obj.to_bytes().unwrap();
        let deserialized = Object::from_bytes(&bytes).unwrap();
        
        assert_eq!(obj.object_type(), deserialized.object_type());
        assert_eq!(obj.as_tree().unwrap().entries.len(), 
                   deserialized.as_tree().unwrap().entries.len());
    }

    #[test]
    fn test_object_serialization_commit() {
        let commit = Commit::new(
            Hash::compute(b"tree"),
            vec![Hash::compute(b"parent")],
            "Author".to_string(),
            "Message".to_string(),
        );
        let obj = Object::Commit(commit);
        
        let bytes = obj.to_bytes().unwrap();
        let deserialized = Object::from_bytes(&bytes).unwrap();
        
        assert_eq!(obj.object_type(), deserialized.object_type());
        let commit_des = deserialized.as_commit().unwrap();
        assert_eq!(commit_des.message, "Message");
        assert_eq!(commit_des.author, "Author");
    }

    #[test]
    fn test_object_hash() {
        let blob = Blob::new(b"test".to_vec());
        let obj = Object::Blob(blob);
        
        let hash1 = obj.hash().unwrap();
        let hash2 = obj.hash().unwrap();
        
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_object_as_methods() {
        let blob = Blob::new(vec![1, 2, 3]);
        let tree = Tree::new();
        let commit = Commit::new(
            Hash::compute(b"tree"),
            vec![],
            "Author".to_string(),
            "Message".to_string(),
        );
        
        let blob_obj = Object::Blob(blob);
        let tree_obj = Object::Tree(tree);
        let commit_obj = Object::Commit(commit);
        
        assert!(blob_obj.as_blob().is_some());
        assert!(blob_obj.as_tree().is_none());
        assert!(blob_obj.as_commit().is_none());
        
        assert!(tree_obj.as_tree().is_some());
        assert!(tree_obj.as_blob().is_none());
        assert!(tree_obj.as_commit().is_none());
        
        assert!(commit_obj.as_commit().is_some());
        assert!(commit_obj.as_blob().is_none());
        assert!(commit_obj.as_tree().is_none());
    }

    #[test]
    fn test_object_type_from_str() {
        assert_eq!(ObjectType::from_str("blob").unwrap(), ObjectType::Blob);
        assert_eq!(ObjectType::from_str("tree").unwrap(), ObjectType::Tree);
        assert_eq!(ObjectType::from_str("commit").unwrap(), ObjectType::Commit);
        assert_eq!(ObjectType::from_str("tag").unwrap(), ObjectType::Tag);
        assert!(ObjectType::from_str("invalid").is_err());
    }

    #[test]
    fn test_object_type_as_str() {
        assert_eq!(ObjectType::Blob.as_str(), "blob");
        assert_eq!(ObjectType::Tree.as_str(), "tree");
        assert_eq!(ObjectType::Commit.as_str(), "commit");
        assert_eq!(ObjectType::Tag.as_str(), "tag");
    }
}

