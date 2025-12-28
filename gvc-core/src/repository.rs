use crate::ignore::IgnoreFile;
use crate::index::{Index, IndexEntry};
use crate::refs::RefManager;
use crate::storage::ObjectStorage;
use crate::{
    error::Result, Blob, Commit, DiffEngine, Error, FileDiff, Hash, HookManager, HookType, Object,
    Tree, TreeEntry,
};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Main repository structure
pub struct Repository {
    /// Working directory (project root)
    work_dir: PathBuf,
    /// .gvc directory
    gvc_dir: PathBuf,
    /// Object storage
    storage: ObjectStorage,
    /// Reference manager
    refs: RefManager,
}

impl Repository {
    /// Initialize a new repository
    pub fn init(path: &Path) -> Result<Self> {
        let work_dir = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        let gvc_dir = work_dir.join(".gvc");

        if gvc_dir.exists() {
            return Err(Error::RepositoryExists(work_dir));
        }

        // Create directory structure
        fs::create_dir_all(&gvc_dir)?;
        fs::create_dir_all(gvc_dir.join("objects"))?;
        fs::create_dir_all(gvc_dir.join("refs/heads"))?;
        fs::create_dir_all(gvc_dir.join("refs/tags"))?;
        fs::create_dir_all(gvc_dir.join("modules"))?;
        fs::create_dir_all(gvc_dir.join("hooks"))?;

        // Write config file
        let config = "# GVC Repository Configuration\n[core]\n\trepositoryformatversion = 0\n";
        fs::write(gvc_dir.join("config"), config)?;

        let storage = ObjectStorage::new(&gvc_dir);
        storage.init()?;

        let refs = RefManager::new(&gvc_dir);
        refs.init()?;

        // Initialize empty index
        let index = Index::new();
        index.save(&gvc_dir)?;

        Ok(Self {
            work_dir,
            gvc_dir,
            storage,
            refs,
        })
    }

    /// Open an existing repository
    pub fn open(path: &Path) -> Result<Self> {
        let work_dir = Self::find_repo_root(path)?;
        let gvc_dir = work_dir.join(".gvc");

        let storage = ObjectStorage::new(&gvc_dir);
        let refs = RefManager::new(&gvc_dir);

        Ok(Self {
            work_dir,
            gvc_dir,
            storage,
            refs,
        })
    }

    /// Find repository root by searching for .gvc directory
    fn find_repo_root(start: &Path) -> Result<PathBuf> {
        let mut current = start.canonicalize().unwrap_or_else(|_| start.to_path_buf());

        loop {
            let gvc_dir = current.join(".gvc");
            if gvc_dir.exists() && gvc_dir.is_dir() {
                return Ok(current);
            }

            match current.parent() {
                Some(parent) => current = parent.to_path_buf(),
                None => return Err(Error::RepositoryNotFound(start.to_path_buf())),
            }
        }
    }

    /// Add file(s) to staging area
    pub fn add(&self, paths: &[PathBuf]) -> Result<()> {
        let mut index = Index::load(&self.gvc_dir)?;

        for path in paths {
            let full_path = if path.is_absolute() {
                path.clone()
            } else {
                self.work_dir.join(path)
            };

            if !full_path.exists() {
                return Err(Error::FileNotFound(path.clone()));
            }

            if full_path.is_file() {
                self.add_file(&mut index, &full_path)?;
            } else if full_path.is_dir() {
                self.add_directory(&mut index, &full_path)?;
            }
        }

        index.save(&self.gvc_dir)?;
        Ok(())
    }

    /// Add single file to index
    fn add_file(&self, index: &mut Index, file_path: &Path) -> Result<()> {
        let content = fs::read(file_path)?;
        let metadata = fs::metadata(file_path)?;

        // Create blob and store it
        let blob = Blob::new(content);
        let obj = Object::Blob(blob);
        let hash = self.storage.store(&obj)?;

        // Get relative path from work_dir
        let rel_path = file_path
            .strip_prefix(&self.work_dir)
            .unwrap_or(file_path)
            .to_path_buf();

        let entry = IndexEntry {
            path: rel_path,
            hash,
            size: metadata.len(),
            mtime: metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
        };

        index.add_entry(entry);
        Ok(())
    }

    /// Add directory recursively to index
    fn add_directory(&self, index: &mut Index, dir_path: &Path) -> Result<()> {
        for entry in WalkDir::new(dir_path).into_iter().filter_entry(|e| {
            // Skip .gvc directory
            !e.path().components().any(|c| c.as_os_str() == ".gvc")
        }) {
            let entry = entry?;
            if entry.file_type().is_file() {
                self.add_file(index, entry.path())?;
            }
        }
        Ok(())
    }

    /// Create a commit from staged changes
    pub fn commit(&self, message: &str, author: &str) -> Result<Hash> {
        let index = Index::load(&self.gvc_dir)?;

        if index.is_empty() {
            return Err(Error::CommitError("Nothing to commit".to_string()));
        }

        // Execute pre-commit hooks
        let hook_manager = HookManager::new(&self.gvc_dir);
        let mut env_vars = HashMap::new();
        env_vars.insert("GVC_AUTHOR".to_string(), author.to_string());
        env_vars.insert("GVC_MESSAGE".to_string(), message.to_string());

        let hook_results = hook_manager.execute(HookType::PreCommit, env_vars.clone())?;

        // Check if any hook failed
        for result in &hook_results {
            if result.is_failure() {
                return Err(Error::CommitError(format!(
                    "Pre-commit hook failed with exit code: {:?}\n{}",
                    result.exit_code, result.stderr
                )));
            }
        }

        // Build tree from index
        let tree_hash = self.build_tree_from_index(&index)?;

        // Get parent commit(s)
        let mut parents = Vec::new();
        if let Some(head_hash) = self.refs.get_head()? {
            parents.push(head_hash);
        }

        // Create commit object
        let commit = Commit::new(tree_hash, parents, author.to_string(), message.to_string());
        let commit_obj = Object::Commit(commit);
        let commit_hash = self.storage.store(&commit_obj)?;

        // Update HEAD
        if let Some(branch_ref) = self.refs.get_head_symbolic()? {
            self.refs.update_ref(&branch_ref, &commit_hash)?;
        } else {
            self.refs.set_head_direct(&commit_hash)?;
        }

        // Execute post-commit hooks
        env_vars.insert("GVC_COMMIT_HASH".to_string(), commit_hash.to_hex());
        let _ = hook_manager.execute(HookType::PostCommit, env_vars);
        // Ignore post-commit hook failures (they don't prevent commit)

        Ok(commit_hash)
    }

    /// Build tree structure from index entries
    fn build_tree_from_index(&self, index: &Index) -> Result<Hash> {
        // Group entries by directory
        let mut tree_map: BTreeMap<PathBuf, BTreeMap<String, TreeEntry>> = BTreeMap::new();

        for entry in index.entries().values() {
            let mut current_path = PathBuf::new();
            let components: Vec<std::path::Component> = entry.path.components().collect();

            for (i, component) in components.iter().enumerate() {
                let name: String = component.as_os_str().to_string_lossy().to_string();

                if i == components.len() - 1 {
                    // Leaf file
                    let tree_entry = TreeEntry::new_file(name, entry.hash.clone());
                    tree_map
                        .entry(current_path.clone())
                        .or_default()
                        .insert(tree_entry.name.clone(), tree_entry);
                } else {
                    current_path.push(component);
                }
            }
        }

        // Build trees bottom-up
        let mut dir_hashes: BTreeMap<PathBuf, Hash> = BTreeMap::new();

        // Sort paths by depth (deepest first)
        let mut sorted_paths: Vec<_> = tree_map.keys().cloned().collect();
        sorted_paths.sort_by_key(|b| std::cmp::Reverse(b.components().count()));

        for path in sorted_paths {
            let mut tree = Tree::new();

            if let Some(entries) = tree_map.get(&path) {
                for entry in entries.values() {
                    tree.add_entry(entry.clone());
                }
            }

            // Add subdirectories
            for (subdir_path, subdir_hash) in &dir_hashes {
                if let Ok(rel) = subdir_path.strip_prefix(&path) {
                    if rel.components().count() == 1 {
                        let name = rel.to_string_lossy().to_string();
                        let entry = TreeEntry::new_dir(name, subdir_hash.clone());
                        tree.add_entry(entry);
                    }
                }
            }

            let tree_obj = Object::Tree(tree);
            let tree_hash = self.storage.store(&tree_obj)?;
            dir_hashes.insert(path, tree_hash);
        }

        // Return root tree hash
        dir_hashes
            .get(&PathBuf::new())
            .cloned()
            .ok_or_else(|| Error::CommitError("Failed to build tree".to_string()))
    }

    /// Get commit history starting from HEAD
    pub fn log(&self, max_count: Option<usize>) -> Result<Vec<(Hash, Commit)>> {
        let mut commits = Vec::new();

        let head_hash = match self.refs.get_head()? {
            Some(h) => h,
            None => return Ok(commits),
        };

        let mut current = Some(head_hash);
        let mut count = 0;

        while let Some(hash) = current {
            if let Some(max) = max_count {
                if count >= max {
                    break;
                }
            }

            let obj = self.storage.load(&hash)?;
            if let Some(commit) = obj.as_commit() {
                commits.push((hash.clone(), commit.clone()));
                current = commit.parents.first().cloned();
                count += 1;
            } else {
                break;
            }
        }

        Ok(commits)
    }

    /// Get object from storage
    pub fn get_object(&self, hash: &Hash) -> Result<Object> {
        self.storage.load(hash)
    }

    /// Get current branch name
    pub fn current_branch(&self) -> Result<Option<String>> {
        self.refs.current_branch()
    }

    /// Create a new branch
    pub fn create_branch(&self, name: &str) -> Result<()> {
        let head_hash = self
            .refs
            .get_head()?
            .ok_or_else(|| Error::CommitError("No commits yet".to_string()))?;

        self.refs.create_branch(name, &head_hash)
    }

    /// Delete a branch
    pub fn delete_branch(&self, name: &str) -> Result<()> {
        self.refs.delete_branch(name)
    }

    /// List all branches
    pub fn list_branches(&self) -> Result<Vec<String>> {
        self.refs.list_branches()
    }

    /// Checkout a branch
    pub fn checkout(&self, branch: &str) -> Result<()> {
        let ref_path = format!("refs/heads/{}", branch);

        // Verify branch exists and get commit hash
        let commit_hash = self
            .refs
            .resolve_ref(&ref_path)?
            .ok_or_else(|| Error::BranchNotFound(branch.to_string()))?;

        // Check for uncommitted changes
        let status = self.status_detailed()?;
        if status.has_unstaged_changes() {
            return Err(Error::CommitError(
                "You have unstaged changes. Commit or discard them before checkout.".to_string(),
            ));
        }

        // Execute pre-checkout hooks
        let hook_manager = HookManager::new(&self.gvc_dir);
        let mut env_vars = HashMap::new();
        env_vars.insert("GVC_BRANCH".to_string(), branch.to_string());
        env_vars.insert("GVC_COMMIT_HASH".to_string(), commit_hash.to_hex());

        let hook_results = hook_manager.execute(HookType::PreCheckout, env_vars.clone())?;

        // Check if any hook failed
        for result in &hook_results {
            if result.is_failure() {
                return Err(Error::CommitError(format!(
                    "Pre-checkout hook failed: {}",
                    result.stderr
                )));
            }
        }

        // Get target commit tree
        let commit_obj = self.storage.load(&commit_hash)?;
        let commit = commit_obj
            .as_commit()
            .ok_or_else(|| Error::InvalidObjectType("Expected commit".to_string()))?;

        let tree_obj = self.storage.load(&commit.tree)?;

        // Update working directory
        self.update_working_directory(&tree_obj, &PathBuf::new())?;

        // Update HEAD
        self.refs.set_head_symbolic(&ref_path)?;

        // Update index to match checked out tree
        self.update_index_from_tree(&tree_obj, &PathBuf::new())?;

        // Execute post-checkout hooks
        let _ = hook_manager.execute(HookType::PostCheckout, env_vars);
        // Ignore post-checkout hook failures

        Ok(())
    }

    /// Update working directory to match tree
    fn update_working_directory(&self, tree_obj: &Object, base_path: &Path) -> Result<()> {
        let tree = tree_obj
            .as_tree()
            .ok_or_else(|| Error::InvalidObjectType("Expected tree".to_string()))?;

        for entry in tree.entries.values() {
            let entry_path = base_path.join(&entry.name);
            let full_path = self.work_dir.join(&entry_path);

            if entry.is_file() {
                // Write file
                let blob_obj = self.storage.load(&entry.hash)?;
                let blob = blob_obj
                    .as_blob()
                    .ok_or_else(|| Error::InvalidObjectType("Expected blob".to_string()))?;

                // Create parent directories
                if let Some(parent) = full_path.parent() {
                    fs::create_dir_all(parent)?;
                }

                fs::write(&full_path, &blob.data)?;
            } else if entry.is_dir() {
                // Recursively process subdirectory
                let subtree_obj = self.storage.load(&entry.hash)?;
                self.update_working_directory(&subtree_obj, &entry_path)?;
            }
        }

        Ok(())
    }

    /// Update index to match tree
    fn update_index_from_tree(&self, tree_obj: &Object, base_path: &Path) -> Result<()> {
        let mut index = Index::new();
        self.add_tree_to_index(&mut index, tree_obj, base_path)?;
        index.save(&self.gvc_dir)?;
        Ok(())
    }

    /// Recursively add tree entries to index
    fn add_tree_to_index(
        &self,
        index: &mut Index,
        tree_obj: &Object,
        base_path: &Path,
    ) -> Result<()> {
        let tree = tree_obj
            .as_tree()
            .ok_or_else(|| Error::InvalidObjectType("Expected tree".to_string()))?;

        for entry in tree.entries.values() {
            let entry_path = base_path.join(&entry.name);

            if entry.is_file() {
                let full_path = self.work_dir.join(&entry_path);
                let metadata = fs::metadata(&full_path)?;

                let index_entry = IndexEntry {
                    path: entry_path,
                    hash: entry.hash.clone(),
                    size: metadata.len(),
                    mtime: metadata
                        .modified()
                        .ok()
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs() as i64)
                        .unwrap_or(0),
                };

                index.add_entry(index_entry);
            } else if entry.is_dir() {
                let subtree_obj = self.storage.load(&entry.hash)?;
                self.add_tree_to_index(index, &subtree_obj, &entry_path)?;
            }
        }

        Ok(())
    }

    /// Create a tag
    pub fn create_tag(&self, name: &str) -> Result<()> {
        let head_hash = self
            .refs
            .get_head()?
            .ok_or_else(|| Error::CommitError("No commits yet".to_string()))?;

        self.refs.create_tag(name, &head_hash)
    }

    /// List all tags
    pub fn list_tags(&self) -> Result<Vec<String>> {
        self.refs.list_tags()
    }

    /// Get repository paths
    pub fn work_dir(&self) -> &Path {
        &self.work_dir
    }

    pub fn gvc_dir(&self) -> &Path {
        &self.gvc_dir
    }

    /// Get diff between staged changes and HEAD
    pub fn diff_staged(&self) -> Result<Vec<FileDiff>> {
        let index = Index::load(&self.gvc_dir)?;
        let mut diffs = Vec::new();

        // Get HEAD tree
        let head_tree = if let Some(head_hash) = self.refs.get_head()? {
            let commit_obj = self.storage.load(&head_hash)?;
            if let Some(commit) = commit_obj.as_commit() {
                Some(self.storage.load(&commit.tree)?)
            } else {
                None
            }
        } else {
            None
        };

        // Get files in HEAD tree
        let head_files = if let Some(tree_obj) = head_tree {
            self.get_tree_files(&tree_obj)?
        } else {
            BTreeMap::new()
        };

        // Compare index with HEAD
        for (path, entry) in index.entries() {
            let old_hash = head_files.get(path as &PathBuf).cloned();
            let new_hash = Some(entry.hash.clone());

            if old_hash != new_hash {
                let old_blob = if let Some(ref h) = old_hash {
                    Some(self.storage.load(h)?)
                } else {
                    None
                };

                let new_blob = if let Some(ref h) = new_hash {
                    Some(self.storage.load(h)?)
                } else {
                    None
                };

                let hunks = DiffEngine::diff_blobs(
                    old_blob.as_ref().and_then(|o| o.as_blob()),
                    new_blob.as_ref().and_then(|o| o.as_blob()),
                )?;

                diffs.push(FileDiff {
                    path: (path as &PathBuf).to_string_lossy().to_string(),
                    old_hash,
                    new_hash,
                    hunks,
                });
            }
        }

        // Check for deleted files
        for (path, hash) in head_files {
            if !index.entries().contains_key(&path) {
                let old_blob = self.storage.load(&hash)?;
                let hunks = DiffEngine::diff_blobs(old_blob.as_blob(), None)?;

                diffs.push(FileDiff {
                    path: path.to_string_lossy().to_string(),
                    old_hash: Some(hash),
                    new_hash: None,
                    hunks,
                });
            }
        }

        Ok(diffs)
    }

    /// Get diff between working directory and index (unstaged changes)
    pub fn diff_unstaged(&self) -> Result<Vec<FileDiff>> {
        let index = Index::load(&self.gvc_dir)?;
        let mut diffs = Vec::new();

        for (path, entry) in index.entries() {
            let full_path = self.work_dir.join(path as &PathBuf);

            if !full_path.exists() {
                // File deleted
                let old_blob = self.storage.load(&entry.hash)?;
                let hunks = DiffEngine::diff_blobs(old_blob.as_blob(), None)?;

                diffs.push(FileDiff {
                    path: (path as &PathBuf).to_string_lossy().to_string(),
                    old_hash: Some(entry.hash.clone()),
                    new_hash: None,
                    hunks,
                });
            } else {
                // Check if modified
                let content = fs::read(&full_path)?;
                let new_blob = Blob::new(content);
                let new_hash = Object::Blob(new_blob.clone()).hash()?;

                if new_hash != entry.hash {
                    let old_blob = self.storage.load(&entry.hash)?;
                    let hunks = DiffEngine::diff_blobs(old_blob.as_blob(), Some(&new_blob))?;

                    diffs.push(FileDiff {
                        path: path.to_string_lossy().to_string(),
                        old_hash: Some(entry.hash.clone()),
                        new_hash: Some(new_hash),
                        hunks,
                    });
                }
            }
        }

        Ok(diffs)
    }

    /// Get all files from a tree recursively
    fn get_tree_files(&self, tree_obj: &Object) -> Result<BTreeMap<PathBuf, Hash>> {
        let mut files = BTreeMap::new();

        if let Some(tree) = tree_obj.as_tree() {
            for entry in tree.entries.values() {
                if entry.is_file() {
                    files.insert(PathBuf::from(&entry.name), entry.hash.clone());
                } else if entry.is_dir() {
                    let subtree_obj = self.storage.load(&entry.hash)?;
                    let subfiles = self.get_tree_files(&subtree_obj)?;
                    for (subpath, hash) in subfiles {
                        files.insert(PathBuf::from(&entry.name).join(subpath), hash);
                    }
                }
            }
        }

        Ok(files)
    }

    /// Get repository status
    pub fn status_detailed(&self) -> Result<RepositoryStatus> {
        let index = Index::load(&self.gvc_dir)?;
        let ignore = IgnoreFile::load(&self.work_dir)?;

        // Get HEAD tree files
        let head_files = if let Some(head_hash) = self.refs.get_head()? {
            let commit_obj = self.storage.load(&head_hash)?;
            if let Some(commit) = commit_obj.as_commit() {
                let tree_obj = self.storage.load(&commit.tree)?;
                self.get_tree_files(&tree_obj)?
            } else {
                BTreeMap::new()
            }
        } else {
            BTreeMap::new()
        };

        let mut status = RepositoryStatus::new();

        // Check staged files
        for (path, entry) in index.entries() {
            let head_hash = head_files.get(path as &PathBuf);

            if head_hash.is_none() {
                status.staged_new.push((path as &PathBuf).clone());
            } else if Some(&entry.hash) != head_hash {
                status.staged_modified.push(path.clone());
            }
        }

        // Check working directory
        for entry in WalkDir::new(&self.work_dir)
            .into_iter()
            .filter_entry(|e| !e.path().components().any(|c| c.as_os_str() == ".gvc"))
        {
            let entry = entry?;
            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path().strip_prefix(&self.work_dir).unwrap();

            // Check if ignored
            if ignore.should_ignore(path) {
                continue;
            }

            if let Some(index_entry) = index.get_entry(path) {
                // File is tracked - check if modified
                let content = fs::read(entry.path())?;
                let current_hash = Object::Blob(Blob::new(content)).hash()?;

                if current_hash != index_entry.hash {
                    status.modified.push(path.to_path_buf());
                }
            } else if !head_files.contains_key(path) {
                // File is untracked
                status.untracked.push(path.to_path_buf());
            }
        }

        // Check for deleted files
        for path in index.entries().keys() {
            let full_path = self.work_dir.join(path as &PathBuf);
            if !full_path.exists() {
                status.deleted.push((path as &PathBuf).clone());
            }
        }

        Ok(status)
    }

    /// Reset (unstage) files
    pub fn reset(&self, paths: &[PathBuf]) -> Result<()> {
        let mut index = Index::load(&self.gvc_dir)?;

        for path in paths {
            index.remove_entry(path);
        }

        index.save(&self.gvc_dir)?;
        Ok(())
    }

    /// Reset all staged files
    pub fn reset_all(&self) -> Result<()> {
        let mut index = Index::load(&self.gvc_dir)?;
        index.clear();
        index.save(&self.gvc_dir)?;
        Ok(())
    }

    /// Read an object by OID
    pub fn read_object(&self, oid: &Hash) -> Result<Object> {
        self.storage.load(oid)
    }

    /// Read raw object data by OID
    pub fn read_object_raw(&self, oid: &Hash) -> Result<Vec<u8>> {
        let path = self
            .gvc_dir
            .join("objects")
            .join(&oid.to_hex()[0..2])
            .join(&oid.to_hex()[2..]);

        if !path.exists() {
            return Err(Error::ObjectNotFound(oid.to_hex()));
        }

        Ok(fs::read(&path)?)
    }

    /// Write raw object data
    pub fn write_object_raw(&self, oid: &Hash, data: &[u8]) -> Result<()> {
        let hex = oid.to_hex();
        let (prefix, suffix) = hex.split_at(2);
        let dir = self.gvc_dir.join("objects").join(prefix);
        fs::create_dir_all(&dir)?;

        let path = dir.join(suffix);
        if !path.exists() {
            fs::write(&path, data)?;
        }

        Ok(())
    }

    /// Resolve a reference to an OID
    pub fn resolve_ref(&self, ref_name: &str) -> Result<Option<Hash>> {
        self.refs.resolve_ref(ref_name)
    }

    /// Update a reference to point to a new OID
    pub fn update_ref(&self, ref_name: &str, oid: &Hash) -> Result<()> {
        self.refs.update_ref(ref_name, oid)
    }

    /// Get the HEAD reference name
    pub fn get_head(&self) -> Result<Option<String>> {
        self.refs.get_head_symbolic()
    }
}

/// Repository status information
#[derive(Debug, Default)]
pub struct RepositoryStatus {
    pub staged_new: Vec<PathBuf>,
    pub staged_modified: Vec<PathBuf>,
    pub modified: Vec<PathBuf>,
    pub deleted: Vec<PathBuf>,
    pub untracked: Vec<PathBuf>,
}

impl RepositoryStatus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has_staged_changes(&self) -> bool {
        !self.staged_new.is_empty() || !self.staged_modified.is_empty()
    }

    pub fn has_unstaged_changes(&self) -> bool {
        !self.modified.is_empty() || !self.deleted.is_empty()
    }

    pub fn has_untracked_files(&self) -> bool {
        !self.untracked.is_empty()
    }

    pub fn is_clean(&self) -> bool {
        !self.has_staged_changes() && !self.has_unstaged_changes() && !self.has_untracked_files()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_repository_init() {
        let temp = TempDir::new().unwrap();
        let _repo = Repository::init(temp.path()).unwrap();

        assert!(temp.path().join(".gvc").exists());
        assert!(temp.path().join(".gvc/objects").exists());
        assert!(temp.path().join(".gvc/refs/heads").exists());
        assert!(temp.path().join(".gvc/config").exists());
    }

    #[test]
    fn test_add_and_commit() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path()).unwrap();

        // Create test file
        let test_file = temp.path().join("test.txt");
        fs::write(&test_file, b"hello world").unwrap();

        // Add and commit
        repo.add(&[PathBuf::from("test.txt")]).unwrap();
        let commit_hash = repo.commit("Initial commit", "Test Author").unwrap();

        assert!(!commit_hash.to_hex().is_empty());

        // Verify log
        let log = repo.log(None).unwrap();
        assert_eq!(log.len(), 1);
        assert_eq!(log[0].1.message, "Initial commit");
    }
}
