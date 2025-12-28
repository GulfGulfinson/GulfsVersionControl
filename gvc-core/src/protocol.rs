// Protocol definitions for GVC client-server communication
// Wire format using JSON for simplicity and debugging

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::hash::Oid;

// Protocol version for compatibility checking
pub const PROTOCOL_VERSION: u32 = 1;

/// Request from client to server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Request {
    /// List all references in a repository
    ListRefs {
        repository: String,
    },
    /// Get objects by their OIDs
    GetObjects {
        repository: String,
        oids: Vec<Oid>,
    },
    /// Push objects and update refs
    Push {
        repository: String,
        objects: Vec<ObjectData>,
        ref_updates: Vec<RefUpdate>,
    },
    /// Check repository existence and get metadata
    InfoRefs {
        repository: String,
    },
}

/// Response from server to client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Response {
    /// List of references
    Refs {
        refs: HashMap<String, Oid>,
        head: Option<String>,
    },
    /// Objects data
    Objects {
        objects: Vec<ObjectData>,
    },
    /// Push result
    PushResult {
        success: bool,
        updated_refs: Vec<String>,
        message: String,
    },
    /// Repository info
    RepoInfo {
        exists: bool,
        head: Option<String>,
        branches: Vec<String>,
    },
    /// Error response
    Error {
        code: String,
        message: String,
    },
}

/// Object data for transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectData {
    pub oid: Oid,
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
    pub object_type: ObjectType,
}

/// Type of object being transferred
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectType {
    Blob,
    Tree,
    Commit,
    Tag,
}

/// Reference update operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefUpdate {
    pub name: String,
    pub old_oid: Option<Oid>,
    pub new_oid: Oid,
    pub force: bool,
}

/// Remote configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Remote {
    pub name: String,
    pub url: String,
    pub fetch_refs: Vec<String>,
    pub push_refs: Vec<String>,
}

impl Remote {
    pub fn new(name: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            url: url.into(),
            fetch_refs: vec!["refs/heads/*:refs/remotes/origin/*".to_string()],
            push_refs: vec![],
        }
    }
}

// Custom serialization for bytes using base64 or hex
mod serde_bytes {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        hex::decode(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization() {
        let request = Request::ListRefs {
            repository: "test-repo".to_string(),
        };
        let json = serde_json::to_string(&request).unwrap();
        let deserialized: Request = serde_json::from_str(&json).unwrap();
        
        match deserialized {
            Request::ListRefs { repository } => {
                assert_eq!(repository, "test-repo");
            }
            _ => panic!("Wrong request type"),
        }
    }

    #[test]
    fn test_object_data_serialization() {
        let oid = Oid::hash(b"test");
        let data = ObjectData {
            oid: oid.clone(),
            data: b"hello world".to_vec(),
            object_type: ObjectType::Blob,
        };
        
        let json = serde_json::to_string(&data).unwrap();
        let deserialized: ObjectData = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.oid, oid);
        assert_eq!(deserialized.data, b"hello world");
        assert_eq!(deserialized.object_type, ObjectType::Blob);
    }
}

