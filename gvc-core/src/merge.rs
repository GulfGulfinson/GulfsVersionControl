// Merge functionality for GVC

use crate::{Commit, Error, Hash, Object, Repository, Result, Tree, TreeEntry};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Merge result
#[derive(Debug, Clone)]
pub enum MergeResult {
    /// Fast-forward merge (no actual merge needed)
    FastForward { from: Hash, to: Hash },
    /// Successful merge without conflicts
    Success { merge_commit: Hash },
    /// Merge with conflicts that need resolution
    Conflicts { conflicts: Vec<ConflictedFile> },
    /// Already up to date
    UpToDate,
}

/// A file with merge conflicts
#[derive(Debug, Clone)]
pub struct ConflictedFile {
    pub path: PathBuf,
    pub ours: Option<Vec<u8>>,
    pub theirs: Option<Vec<u8>>,
    pub base: Option<Vec<u8>>,
}

/// Merge strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeStrategy {
    /// Recursive three-way merge (default)
    Recursive,
    /// Always prefer our changes
    Ours,
    /// Always prefer their changes
    Theirs,
    /// Fast-forward only (fail if not possible)
    FastForwardOnly,
}

/// Merge manager
pub struct MergeManager<'a> {
    repo: &'a Repository,
}

impl<'a> MergeManager<'a> {
    pub fn new(repo: &'a Repository) -> Self {
        Self { repo }
    }

    /// Merge a branch into current HEAD
    pub fn merge(
        &self,
        their_branch: &str,
        strategy: MergeStrategy,
        message: Option<&str>,
    ) -> Result<MergeResult> {
        // Get current HEAD
        let our_commit_oid = self
            .repo
            .resolve_ref("HEAD")?
            .ok_or_else(|| Error::CommitError("No HEAD commit".to_string()))?;

        // Get their commit
        let their_ref = format!("refs/heads/{}", their_branch);
        let their_commit_oid = self
            .repo
            .resolve_ref(&their_ref)?
            .ok_or_else(|| Error::BranchNotFound(their_branch.to_string()))?;

        // Check if already up to date
        if our_commit_oid == their_commit_oid {
            return Ok(MergeResult::UpToDate);
        }

        // Check for fast-forward
        if self.can_fast_forward(&our_commit_oid, &their_commit_oid)? {
            if strategy == MergeStrategy::FastForwardOnly {
                return self.do_fast_forward(&their_commit_oid);
            }
            // Ask if fast-forward preferred (for now, always do it)
            return self.do_fast_forward(&their_commit_oid);
        }

        if strategy == MergeStrategy::FastForwardOnly {
            return Err(Error::CommitError(
                "Cannot fast-forward, and --ff-only was specified".to_string(),
            ));
        }

        // Find merge base (common ancestor)
        let base_oid = self
            .find_merge_base(&our_commit_oid, &their_commit_oid)?
            .ok_or_else(|| Error::CommitError("No common ancestor found".to_string()))?;

        // Perform three-way merge
        self.three_way_merge(
            &base_oid,
            &our_commit_oid,
            &their_commit_oid,
            their_branch,
            strategy,
            message,
        )
    }

    /// Check if we can fast-forward from our to their
    fn can_fast_forward(&self, our: &Hash, their: &Hash) -> Result<bool> {
        // Can fast-forward if their is an ancestor of our
        let mut current = Some(their.clone());

        while let Some(oid) = current {
            if oid == *our {
                return Ok(true);
            }

            let obj = self.repo.read_object(&oid)?;
            if let Some(commit) = obj.as_commit() {
                current = commit.parents.first().cloned();
            } else {
                break;
            }
        }

        Ok(false)
    }

    /// Perform fast-forward merge
    fn do_fast_forward(&self, to: &Hash) -> Result<MergeResult> {
        let from = self
            .repo
            .resolve_ref("HEAD")?
            .ok_or_else(|| Error::CommitError("No HEAD".to_string()))?;

        // Update HEAD to point to their commit
        self.repo.update_ref("HEAD", to)?;

        // Update working directory
        let commit_obj = self.repo.read_object(to)?;
        let commit = commit_obj
            .as_commit()
            .ok_or_else(|| Error::InvalidObjectType("Expected commit".to_string()))?;
        let tree_obj = self.repo.read_object(&commit.tree)?;

        self.update_working_directory_from_tree(&tree_obj, &PathBuf::new())?;

        Ok(MergeResult::FastForward {
            from,
            to: to.clone(),
        })
    }

    /// Find common ancestor (merge base)
    fn find_merge_base(&self, oid1: &Hash, oid2: &Hash) -> Result<Option<Hash>> {
        // Simple algorithm: find first common ancestor
        let ancestors1 = self.get_ancestors(oid1)?;
        let mut current = Some(oid2.clone());

        while let Some(oid) = current {
            if ancestors1.contains(&oid) {
                return Ok(Some(oid));
            }

            let obj = self.repo.read_object(&oid)?;
            if let Some(commit) = obj.as_commit() {
                current = commit.parents.first().cloned();
            } else {
                break;
            }
        }

        Ok(None)
    }

    /// Get all ancestors of a commit
    fn get_ancestors(&self, oid: &Hash) -> Result<Vec<Hash>> {
        let mut ancestors = Vec::new();
        let mut to_visit = vec![oid.clone()];

        while let Some(current) = to_visit.pop() {
            if ancestors.contains(&current) {
                continue;
            }
            ancestors.push(current.clone());

            let obj = self.repo.read_object(&current)?;
            if let Some(commit) = obj.as_commit() {
                to_visit.extend(commit.parents.iter().cloned());
            }
        }

        Ok(ancestors)
    }

    /// Perform three-way merge
    fn three_way_merge(
        &self,
        base: &Hash,
        ours: &Hash,
        theirs: &Hash,
        their_branch: &str,
        strategy: MergeStrategy,
        message: Option<&str>,
    ) -> Result<MergeResult> {
        // Get trees
        let base_tree = self.get_commit_tree(base)?;
        let our_tree = self.get_commit_tree(ours)?;
        let their_tree = self.get_commit_tree(theirs)?;

        // Get all files
        let base_files = self.get_tree_files(&base_tree)?;
        let our_files = self.get_tree_files(&our_tree)?;
        let their_files = self.get_tree_files(&their_tree)?;

        // Find all unique paths
        let mut all_paths = std::collections::HashSet::new();
        all_paths.extend(base_files.keys().cloned());
        all_paths.extend(our_files.keys().cloned());
        all_paths.extend(their_files.keys().cloned());

        let mut conflicts = Vec::new();
        let mut merged_files = BTreeMap::new();

        for path in all_paths {
            let base_hash = base_files.get(&path).cloned();
            let our_hash = our_files.get(&path).cloned();
            let their_hash = their_files.get(&path).cloned();

            match self.merge_file(&path, base_hash, our_hash, their_hash, strategy)? {
                FileMergeResult::Success(hash) => {
                    merged_files.insert(path, hash);
                }
                FileMergeResult::Conflict(conflict) => {
                    conflicts.push(conflict);
                }
                FileMergeResult::Deleted => {
                    // File was deleted, don't include it
                }
            }
        }

        if !conflicts.is_empty() {
            // Write conflict markers to working directory
            for conflict in &conflicts {
                self.write_conflict_markers(conflict)?;
            }
            return Ok(MergeResult::Conflicts { conflicts });
        }

        // Create merged tree
        let merged_tree = self.create_tree_from_files(&merged_files)?;

        // Create merge commit
        let default_message = format!("Merge branch '{}'", their_branch);
        let merge_message = message.unwrap_or(&default_message);
        let merge_commit = Commit::new(
            merged_tree.clone(),
            vec![ours.clone(), theirs.clone()],
            "Author".to_string(), // TODO: Get from config
            merge_message.to_string(),
        );

        let merge_commit_obj = Object::Commit(merge_commit);
        let merge_commit_hash = self.repo.write_object(&merge_commit_obj)?;

        // Update HEAD
        self.repo.update_ref("HEAD", &merge_commit_hash)?;

        // Update working directory
        let tree_obj = self.repo.read_object(&merged_tree)?;
        self.update_working_directory_from_tree(&tree_obj, &PathBuf::new())?;

        Ok(MergeResult::Success {
            merge_commit: merge_commit_hash,
        })
    }

    /// Merge a single file
    fn merge_file(
        &self,
        path: &Path,
        base: Option<Hash>,
        ours: Option<Hash>,
        theirs: Option<Hash>,
        strategy: MergeStrategy,
    ) -> Result<FileMergeResult> {
        // Clone for later use
        let base_copy = base.clone();

        match (base, ours, theirs) {
            // File unchanged
            (Some(b), Some(o), Some(t)) if b == o && o == t => Ok(FileMergeResult::Success(o)),
            // We modified, they didn't
            (Some(b), Some(o), Some(t)) if b == t && b != o => Ok(FileMergeResult::Success(o)),
            // They modified, we didn't
            (Some(b), Some(o), Some(t)) if b == o && b != t => Ok(FileMergeResult::Success(t)),
            // Both modified (potential conflict)
            (Some(_), Some(o), Some(t)) if o != t => {
                match strategy {
                    MergeStrategy::Ours => Ok(FileMergeResult::Success(o)),
                    MergeStrategy::Theirs => Ok(FileMergeResult::Success(t)),
                    MergeStrategy::Recursive => {
                        // Try to merge content
                        self.try_content_merge(path, base_copy.clone(), Some(o), Some(t))
                    }
                    MergeStrategy::FastForwardOnly => unreachable!(),
                }
            }
            // File added by us
            (None, Some(o), None) => Ok(FileMergeResult::Success(o)),
            // File added by them
            (None, None, Some(t)) => Ok(FileMergeResult::Success(t)),
            // File added by both (conflict if different)
            (None, Some(o), Some(t)) => {
                if o == t {
                    Ok(FileMergeResult::Success(o))
                } else {
                    self.try_content_merge(path, base_copy, Some(o), Some(t))
                }
            }
            // File deleted by us
            (Some(_), None, Some(t)) => match strategy {
                MergeStrategy::Ours => Ok(FileMergeResult::Deleted),
                _ => Ok(FileMergeResult::Success(t)),
            },
            // File deleted by them
            (Some(_), Some(o), None) => match strategy {
                MergeStrategy::Theirs => Ok(FileMergeResult::Deleted),
                _ => Ok(FileMergeResult::Success(o)),
            },
            // File deleted by both
            (Some(_), None, None) => Ok(FileMergeResult::Deleted),
            // File doesn't exist anywhere
            (None, None, None) => Ok(FileMergeResult::Deleted),
            // Other cases
            _ => Ok(FileMergeResult::Deleted),
        }
    }

    /// Try to merge file content
    fn try_content_merge(
        &self,
        path: &Path,
        base: Option<Hash>,
        ours: Option<Hash>,
        theirs: Option<Hash>,
    ) -> Result<FileMergeResult> {
        let base_content = base.and_then(|h| self.read_blob_content(&h).ok());
        let our_content = ours.and_then(|h| self.read_blob_content(&h).ok());
        let their_content = theirs.and_then(|h| self.read_blob_content(&h).ok());

        // For now, just report conflict
        // TODO: Implement actual line-by-line merge
        Ok(FileMergeResult::Conflict(ConflictedFile {
            path: path.to_path_buf(),
            ours: our_content,
            theirs: their_content,
            base: base_content,
        }))
    }

    /// Read blob content
    fn read_blob_content(&self, hash: &Hash) -> Result<Vec<u8>> {
        let obj = self.repo.read_object(hash)?;
        let blob = obj
            .as_blob()
            .ok_or_else(|| Error::InvalidObjectType("Expected blob".to_string()))?;
        Ok(blob.data.clone())
    }

    /// Write conflict markers to file
    fn write_conflict_markers(&self, conflict: &ConflictedFile) -> Result<()> {
        let mut content = Vec::new();

        content.extend_from_slice(b"<<<<<<< HEAD\n");
        if let Some(ours) = &conflict.ours {
            content.extend_from_slice(ours);
        }
        content.extend_from_slice(b"=======\n");
        if let Some(theirs) = &conflict.theirs {
            content.extend_from_slice(theirs);
        }
        content.extend_from_slice(b">>>>>>> MERGE\n");

        let full_path = self.repo.work_dir().join(&conflict.path);
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&full_path, content)?;

        Ok(())
    }

    // Helper methods
    fn get_commit_tree(&self, commit_oid: &Hash) -> Result<Object> {
        let commit_obj = self.repo.read_object(commit_oid)?;
        let commit = commit_obj
            .as_commit()
            .ok_or_else(|| Error::InvalidObjectType("Expected commit".to_string()))?;
        self.repo.read_object(&commit.tree)
    }

    fn get_tree_files(&self, tree_obj: &Object) -> Result<BTreeMap<PathBuf, Hash>> {
        let mut files = BTreeMap::new();
        self.collect_tree_files(tree_obj, &PathBuf::new(), &mut files)?;
        Ok(files)
    }

    fn collect_tree_files(
        &self,
        tree_obj: &Object,
        base_path: &Path,
        files: &mut BTreeMap<PathBuf, Hash>,
    ) -> Result<()> {
        let tree = tree_obj
            .as_tree()
            .ok_or_else(|| Error::InvalidObjectType("Expected tree".to_string()))?;

        for entry in tree.entries.values() {
            let entry_path = base_path.join(&entry.name);

            if entry.is_file() {
                files.insert(entry_path, entry.hash.clone());
            } else if entry.is_dir() {
                let subtree = self.repo.read_object(&entry.hash)?;
                self.collect_tree_files(&subtree, &entry_path, files)?;
            }
        }

        Ok(())
    }

    fn create_tree_from_files(&self, files: &BTreeMap<PathBuf, Hash>) -> Result<Hash> {
        // Create tree structure
        let mut root_tree = Tree::new();

        for (path, hash) in files {
            let components: Vec<_> = path.components().collect();
            self.add_file_to_tree(&mut root_tree, &components, hash)?;
        }

        let tree_obj = Object::Tree(root_tree);
        self.repo.write_object(&tree_obj)
    }

    fn add_file_to_tree(
        &self,
        tree: &mut Tree,
        components: &[std::path::Component],
        hash: &Hash,
    ) -> Result<()> {
        if components.is_empty() {
            return Ok(());
        }

        let name = components[0].as_os_str().to_string_lossy().to_string();

        if components.len() == 1 {
            // Leaf node - add file
            tree.add_entry(TreeEntry::new_file(name, hash.clone()));
        } else {
            // Directory - recurse
            // TODO: Implement proper tree building for directories
            tree.add_entry(TreeEntry::new_file(name, hash.clone()));
        }

        Ok(())
    }

    fn update_working_directory_from_tree(
        &self,
        tree_obj: &Object,
        base_path: &Path,
    ) -> Result<()> {
        let tree = tree_obj
            .as_tree()
            .ok_or_else(|| Error::InvalidObjectType("Expected tree".to_string()))?;

        for entry in tree.entries.values() {
            let entry_path = base_path.join(&entry.name);
            let full_path = self.repo.work_dir().join(&entry_path);

            if entry.is_file() {
                let blob_obj = self.repo.read_object(&entry.hash)?;
                let blob = blob_obj
                    .as_blob()
                    .ok_or_else(|| Error::InvalidObjectType("Expected blob".to_string()))?;

                if let Some(parent) = full_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&full_path, &blob.data)?;
            } else if entry.is_dir() {
                let subtree = self.repo.read_object(&entry.hash)?;
                self.update_working_directory_from_tree(&subtree, &entry_path)?;
            }
        }

        Ok(())
    }
}

/// Result of merging a single file
enum FileMergeResult {
    Success(Hash),
    Conflict(ConflictedFile),
    Deleted,
}

// Expose write_object method on Repository
impl Repository {
    pub fn write_object(&self, object: &Object) -> Result<Hash> {
        let storage = crate::storage::ObjectStorage::new(&self.gvc_dir().join(""));
        storage.store(object)
    }
}
