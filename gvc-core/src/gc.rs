// Garbage collection for unreachable objects

use crate::{Hash, Object, Repository, Result};
use std::collections::HashSet;

/// Garbage collector for GVC repository
pub struct GarbageCollector<'a> {
    repo: &'a Repository,
}

impl<'a> GarbageCollector<'a> {
    pub fn new(repo: &'a Repository) -> Self {
        Self { repo }
    }

    /// Perform garbage collection
    /// Returns (objects_removed, bytes_freed)
    pub fn collect(&self, dry_run: bool) -> Result<(usize, u64)> {
        // Collect all reachable objects
        let reachable = self.collect_reachable_objects()?;

        // Get all objects in storage
        let storage = crate::storage::ObjectStorage::new(&self.repo.gvc_dir().join(""));
        let all_objects = storage.list_objects()?;

        // Find unreachable objects
        let mut unreachable = Vec::new();
        let mut bytes_freed = 0u64;

        for oid in &all_objects {
            if !reachable.contains(oid) {
                // Calculate object size
                if let Ok(data) = self.repo.read_object_raw(oid) {
                    bytes_freed += data.len() as u64;
                }
                unreachable.push(oid.clone());
            }
        }

        // Remove unreachable objects
        if !dry_run {
            for oid in &unreachable {
                self.remove_object(oid)?;
            }
        }

        Ok((unreachable.len(), bytes_freed))
    }

    /// Collect all objects reachable from refs
    fn collect_reachable_objects(&self) -> Result<HashSet<Hash>> {
        let mut reachable = HashSet::new();
        let mut to_visit = Vec::new();

        // Start with all refs (branches and tags)
        let branches = self.repo.list_branches()?;
        for branch in branches {
            if let Some(oid) = self.repo.resolve_ref(&format!("refs/heads/{}", branch))? {
                to_visit.push(oid);
            }
        }

        let tags = self.repo.list_tags()?;
        for tag in tags {
            if let Some(oid) = self.repo.resolve_ref(&format!("refs/tags/{}", tag))? {
                to_visit.push(oid);
            }
        }

        // Traverse object graph
        while let Some(oid) = to_visit.pop() {
            if reachable.contains(&oid) {
                continue;
            }
            reachable.insert(oid.clone());

            // Load object and add its references
            if let Ok(obj) = self.repo.read_object(&oid) {
                match obj {
                    Object::Commit(commit) => {
                        to_visit.push(commit.tree);
                        to_visit.extend(commit.parents);
                    }
                    Object::Tree(tree) => {
                        for entry in tree.entries.values() {
                            to_visit.push(entry.hash.clone());
                        }
                    }
                    Object::Blob(_) => {
                        // Blobs have no references
                    }
                }
            }
        }

        Ok(reachable)
    }

    /// Remove an object from storage
    fn remove_object(&self, oid: &Hash) -> Result<()> {
        let hex = oid.to_hex();
        let (prefix, suffix) = hex.split_at(2);
        let path = self
            .repo
            .gvc_dir()
            .join("objects")
            .join(prefix)
            .join(suffix);

        if path.exists() {
            std::fs::remove_file(&path)?;
        }

        Ok(())
    }

    /// Get repository statistics
    pub fn stats(&self) -> Result<GcStats> {
        let storage = crate::storage::ObjectStorage::new(&self.repo.gvc_dir().join(""));
        let all_objects = storage.list_objects()?;
        let reachable = self.collect_reachable_objects()?;

        let mut total_size = 0u64;
        let mut reachable_size = 0u64;

        for oid in &all_objects {
            if let Ok(data) = self.repo.read_object_raw(oid) {
                let size = data.len() as u64;
                total_size += size;

                if reachable.contains(oid) {
                    reachable_size += size;
                }
            }
        }

        Ok(GcStats {
            total_objects: all_objects.len(),
            reachable_objects: reachable.len(),
            unreachable_objects: all_objects.len() - reachable.len(),
            total_size,
            reachable_size,
            unreachable_size: total_size - reachable_size,
        })
    }
}

/// Garbage collection statistics
#[derive(Debug, Clone)]
pub struct GcStats {
    pub total_objects: usize,
    pub reachable_objects: usize,
    pub unreachable_objects: usize,
    pub total_size: u64,
    pub reachable_size: u64,
    pub unreachable_size: u64,
}

impl GcStats {
    /// Format size in human-readable format
    pub fn format_size(bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.2} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.2} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} bytes", bytes)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_gc_no_unreachable() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path()).unwrap();

        // Create a commit
        let test_file = temp.path().join("test.txt");
        fs::write(&test_file, b"content").unwrap();

        repo.add(&[test_file.strip_prefix(temp.path()).unwrap().to_path_buf()])
            .unwrap();
        repo.commit("Test", "Author").unwrap();

        // Run GC
        let gc = GarbageCollector::new(&repo);
        let (removed, _) = gc.collect(false).unwrap();

        // Nothing should be removed
        assert_eq!(removed, 0);
    }

    #[test]
    fn test_gc_stats() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path()).unwrap();

        // Create a commit
        let test_file = temp.path().join("test.txt");
        fs::write(&test_file, b"content").unwrap();

        repo.add(&[test_file.strip_prefix(temp.path()).unwrap().to_path_buf()])
            .unwrap();
        repo.commit("Test", "Author").unwrap();

        // Get stats
        let gc = GarbageCollector::new(&repo);
        let stats = gc.stats().unwrap();

        assert!(stats.total_objects > 0);
        assert_eq!(stats.unreachable_objects, 0);
    }

    #[test]
    fn test_format_size() {
        assert_eq!(GcStats::format_size(500), "500 bytes");
        assert_eq!(GcStats::format_size(1024), "1.00 KB");
        assert_eq!(GcStats::format_size(1024 * 1024), "1.00 MB");
        assert_eq!(GcStats::format_size(1024 * 1024 * 1024), "1.00 GB");
    }
}
