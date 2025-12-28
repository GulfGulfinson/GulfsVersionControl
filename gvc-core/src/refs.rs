use crate::{Error, Hash, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Reference types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefType {
    Head,
    Tag,
}

/// Reference manager - handles branches, tags, and HEAD
pub struct RefManager {
    refs_dir: PathBuf,
    heads_dir: PathBuf,
    tags_dir: PathBuf,
    head_file: PathBuf,
}

impl RefManager {
    pub fn new(gvc_dir: &Path) -> Self {
        let refs_dir = gvc_dir.join("refs");
        Self {
            heads_dir: refs_dir.join("heads"),
            tags_dir: refs_dir.join("tags"),
            refs_dir,
            head_file: gvc_dir.join("HEAD"),
        }
    }

    /// Initialize reference directories
    pub fn init(&self) -> Result<()> {
        fs::create_dir_all(&self.heads_dir)?;
        fs::create_dir_all(&self.tags_dir)?;

        // Create default branch (main)
        self.set_head_symbolic("refs/heads/main")?;

        Ok(())
    }

    /// Get current HEAD reference
    pub fn get_head(&self) -> Result<Option<Hash>> {
        if !self.head_file.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&self.head_file)?;
        let content = content.trim();

        // HEAD can be symbolic (ref: refs/heads/main) or direct hash
        if let Some(ref_path) = content.strip_prefix("ref: ") {
            self.resolve_ref(ref_path)
        } else {
            Hash::from_hex(content).map(Some)
        }
    }

    /// Get symbolic HEAD reference (e.g., "refs/heads/main")
    pub fn get_head_symbolic(&self) -> Result<Option<String>> {
        if !self.head_file.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&self.head_file)?;
        let content = content.trim();

        if let Some(ref_path) = content.strip_prefix("ref: ") {
            Ok(Some(ref_path.to_string()))
        } else {
            Ok(None)
        }
    }

    /// Set HEAD to point to a symbolic reference
    pub fn set_head_symbolic(&self, ref_path: &str) -> Result<()> {
        let content = format!("ref: {}", ref_path);
        fs::write(&self.head_file, content)?;
        Ok(())
    }

    /// Set HEAD to point directly to a hash (detached HEAD)
    pub fn set_head_direct(&self, hash: &Hash) -> Result<()> {
        fs::write(&self.head_file, hash.to_hex())?;
        Ok(())
    }

    /// Resolve a reference to a hash
    pub fn resolve_ref(&self, ref_path: &str) -> Result<Option<Hash>> {
        let path = self.ref_path(ref_path);

        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&path)?;
        Hash::from_hex(content.trim()).map(Some)
    }

    /// Update a reference to point to a hash
    pub fn update_ref(&self, ref_path: &str, hash: &Hash) -> Result<()> {
        let path = self.ref_path(ref_path);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&path, hash.to_hex())?;
        Ok(())
    }

    /// Create a new branch
    pub fn create_branch(&self, name: &str, hash: &Hash) -> Result<()> {
        let ref_path = format!("refs/heads/{}", name);
        let path = self.ref_path(&ref_path);

        if path.exists() {
            return Err(Error::BranchExists(name.to_string()));
        }

        self.update_ref(&ref_path, hash)
    }

    /// Delete a branch
    pub fn delete_branch(&self, name: &str) -> Result<()> {
        let ref_path = format!("refs/heads/{}", name);
        let path = self.ref_path(&ref_path);

        if !path.exists() {
            return Err(Error::BranchNotFound(name.to_string()));
        }

        fs::remove_file(&path)?;
        Ok(())
    }

    /// List all branches
    pub fn list_branches(&self) -> Result<Vec<String>> {
        let mut branches = Vec::new();

        if !self.heads_dir.exists() {
            return Ok(branches);
        }

        for entry in fs::read_dir(&self.heads_dir)? {
            let entry = entry?;
            if let Some(name) = entry.file_name().to_str() {
                branches.push(name.to_string());
            }
        }

        branches.sort();
        Ok(branches)
    }

    /// Get current branch name
    pub fn current_branch(&self) -> Result<Option<String>> {
        if let Some(symbolic) = self.get_head_symbolic()? {
            if let Some(branch) = symbolic.strip_prefix("refs/heads/") {
                return Ok(Some(branch.to_string()));
            }
        }
        Ok(None)
    }

    /// Create a tag
    pub fn create_tag(&self, name: &str, hash: &Hash) -> Result<()> {
        let ref_path = format!("refs/tags/{}", name);
        self.update_ref(&ref_path, hash)
    }

    /// List all tags
    pub fn list_tags(&self) -> Result<Vec<String>> {
        let mut tags = Vec::new();

        if !self.tags_dir.exists() {
            return Ok(tags);
        }

        for entry in fs::read_dir(&self.tags_dir)? {
            let entry = entry?;
            if let Some(name) = entry.file_name().to_str() {
                tags.push(name.to_string());
            }
        }

        tags.sort();
        Ok(tags)
    }

    /// Get full path for a reference
    fn ref_path(&self, ref_path: &str) -> PathBuf {
        self.refs_dir.parent().unwrap().join(ref_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_ref_manager_init() {
        let temp = TempDir::new().unwrap();
        let refs = RefManager::new(temp.path());
        refs.init().unwrap();

        assert!(temp.path().join("refs/heads").exists());
        assert!(temp.path().join("refs/tags").exists());
        assert!(temp.path().join("HEAD").exists());
    }

    #[test]
    fn test_branch_operations() {
        let temp = TempDir::new().unwrap();
        let refs = RefManager::new(temp.path());
        refs.init().unwrap();

        let hash = Hash::compute(b"test");
        refs.create_branch("feature", &hash).unwrap();

        let branches = refs.list_branches().unwrap();
        assert!(branches.contains(&"feature".to_string()));

        let resolved = refs.resolve_ref("refs/heads/feature").unwrap();
        assert_eq!(resolved, Some(hash));
    }
}
