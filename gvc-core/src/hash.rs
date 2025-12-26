use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

/// SHA-256 hash wrapper
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hash([u8; 32]);

impl Hash {
    /// Compute hash from bytes
    pub fn compute(data: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        Hash(hash)
    }

    /// Create from hex string
    pub fn from_hex(s: &str) -> Result<Self, crate::Error> {
        let bytes = hex::decode(s)
            .map_err(|e| crate::Error::InvalidHash(format!("Invalid hex: {}", e)))?;
        
        if bytes.len() != 32 {
            return Err(crate::Error::InvalidHash(
                "Hash must be 32 bytes".to_string()
            ));
        }

        let mut hash = [0u8; 32];
        hash.copy_from_slice(&bytes);
        Ok(Hash(hash))
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Get raw bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Get first n characters of hex representation
    pub fn short(&self, len: usize) -> String {
        let hex = self.to_hex();
        hex.chars().take(len).collect()
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl From<[u8; 32]> for Hash {
    fn from(bytes: [u8; 32]) -> Self {
        Hash(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_compute() {
        let data = b"hello world";
        let hash = Hash::compute(data);
        assert_eq!(hash.to_hex().len(), 64);
    }

    #[test]
    fn test_hash_roundtrip() {
        let data = b"test data";
        let hash = Hash::compute(data);
        let hex = hash.to_hex();
        let parsed = Hash::from_hex(&hex).unwrap();
        assert_eq!(hash, parsed);
    }
}

