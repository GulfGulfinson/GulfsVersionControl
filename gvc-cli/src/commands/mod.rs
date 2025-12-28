use gvc_core::{Commit, Hash, Object, Repository, ModuleManager, ModuleManifest, RemoteManager, RemoteClient, ObjectData};
use std::env;
use std::path::{Path, PathBuf};
use std::collections::HashSet;

/// Initialize a new repository
pub fn init(path: &Path) -> anyhow::Result<()> {
    Repository::init(path)?;
    println!("Initialized empty GVC repository in {}", path.display());
    Ok(())
}

/// Show repository status
pub fn status() -> anyhow::Result<()> {
    let repo = Repository::open(&env::current_dir()?)?;
    
    // Get current branch
    if let Some(branch) = repo.current_branch()? {
        println!("On branch {}", branch);
    } else {
        println!("HEAD detached");
    }
    
    // Get detailed status
    let status = repo.status_detailed()?;
    
    if status.is_clean() {
        println!("\nNothing to commit, working tree clean");
        return Ok(());
    }
    
    // Staged changes
    if status.has_staged_changes() {
        println!("\nChanges to be committed:");
        println!("  (use \"gvc reset <file>...\" to unstage)");
        println!();
        
        for path in &status.staged_new {
            println!("  \x1b[32mnew file:   {}\x1b[0m", path.display());
        }
        for path in &status.staged_modified {
            println!("  \x1b[32mmodified:   {}\x1b[0m", path.display());
        }
    }
    
    // Unstaged changes
    if status.has_unstaged_changes() {
        println!("\nChanges not staged for commit:");
        println!("  (use \"gvc add <file>...\" to update what will be committed)");
        println!();
        
        for path in &status.modified {
            println!("  \x1b[31mmodified:   {}\x1b[0m", path.display());
        }
        for path in &status.deleted {
            println!("  \x1b[31mdeleted:    {}\x1b[0m", path.display());
        }
    }
    
    // Untracked files
    if status.has_untracked_files() {
        println!("\nUntracked files:");
        println!("  (use \"gvc add <file>...\" to include in what will be committed)");
        println!();
        
        for path in &status.untracked {
            println!("  \x1b[31m{}\x1b[0m", path.display());
        }
    }
    
    Ok(())
}

/// Add files to staging area
pub fn add(paths: &[PathBuf]) -> anyhow::Result<()> {
    let repo = Repository::open(&env::current_dir()?)?;
    repo.add(paths)?;
    println!("Added {} file(s) to staging area", paths.len());
    Ok(())
}

/// Create a commit
pub fn commit(message: &str, author: Option<&str>) -> anyhow::Result<()> {
    let repo = Repository::open(&env::current_dir()?)?;
    
    // Get author from parameter or environment
    let author = author
        .map(String::from)
        .or_else(|| env::var("GVC_AUTHOR").ok())
        .or_else(|| env::var("USER").ok())
        .or_else(|| env::var("USERNAME").ok())
        .unwrap_or_else(|| "Unknown".to_string());
    
    let hash = repo.commit(message, &author)?;
    
    println!("[{}] {}", hash.short(7), message);
    Ok(())
}

/// Show commit history
pub fn log(max_count: Option<usize>, oneline: bool) -> anyhow::Result<()> {
    let repo = Repository::open(&env::current_dir()?)?;
    let commits = repo.log(max_count)?;
    
    if commits.is_empty() {
        println!("No commits yet");
        return Ok(());
    }
    
    for (hash, commit) in commits {
        if oneline {
            println!("{} {}", hash.short(7), commit.message.lines().next().unwrap_or(""));
        } else {
            print_commit(&hash, &commit);
            println!();
        }
    }
    
    Ok(())
}

/// Print commit details
fn print_commit(hash: &Hash, commit: &Commit) {
    println!("commit {}", hash.to_hex());
    println!("Author: {}", commit.author);
    println!("Date:   {}", format_timestamp(commit.timestamp));
    println!();
    for line in commit.message.lines() {
        println!("    {}", line);
    }
}

/// Format Unix timestamp
fn format_timestamp(timestamp: i64) -> String {
    use chrono::{DateTime, Utc};
    let dt = DateTime::<Utc>::from_timestamp(timestamp, 0)
        .unwrap_or_else(|| DateTime::<Utc>::from_timestamp(0, 0).unwrap());
    dt.format("%a %b %d %H:%M:%S %Y %z").to_string()
}

/// Show differences
pub fn diff(staged: bool) -> anyhow::Result<()> {
    let repo = Repository::open(&env::current_dir()?)?;
    
    let diffs = if staged {
        repo.diff_staged()?
    } else {
        repo.diff_unstaged()?
    };
    
    if diffs.is_empty() {
        println!("No changes");
        return Ok(());
    }
    
    for diff in diffs {
        print_diff(&diff);
    }
    
    Ok(())
}

/// Print a file diff
fn print_diff(diff: &gvc_core::FileDiff) {
    use gvc_core::Change;
    
    // File header
    if diff.is_new_file() {
        println!("\x1b[1mnew file: {}\x1b[0m", diff.path);
    } else if diff.is_deleted_file() {
        println!("\x1b[1mdeleted file: {}\x1b[0m", diff.path);
    } else {
        println!("\x1b[1mdiff --gvc a/{} b/{}\x1b[0m", diff.path, diff.path);
    }
    
    // Hunks
    for hunk in &diff.hunks {
        println!(
            "\x1b[36m@@ -{},{} +{},{} @@\x1b[0m",
            hunk.old_start, hunk.old_count, hunk.new_start, hunk.new_count
        );
        
        for change in &hunk.changes {
            match change {
                Change::Add(line) => println!("\x1b[32m+{}\x1b[0m", line),
                Change::Delete(line) => println!("\x1b[31m-{}\x1b[0m", line),
                Change::Context(line) => println!(" {}", line),
            }
        }
    }
    
    println!();
}

/// Show object content
pub fn show(hash_str: &str) -> anyhow::Result<()> {
    let repo = Repository::open(&env::current_dir()?)?;
    let hash = Hash::from_hex(hash_str)?;
    let obj = repo.get_object(&hash)?;
    
    match obj {
        Object::Blob(blob) => {
            println!("blob {}", hash.to_hex());
            println!("size: {} bytes", blob.size());
            println!();
            
            // Try to print as text
            if let Ok(text) = String::from_utf8(blob.data.clone()) {
                print!("{}", text);
            } else {
                println!("<binary data>");
            }
        }
        Object::Tree(tree) => {
            println!("tree {}", hash.to_hex());
            println!();
            for entry in tree.entries.values() {
                println!(
                    "{:06o} {:6} {}  {}",
                    entry.mode,
                    entry.obj_type.as_str(),
                    entry.hash.short(7),
                    entry.name
                );
            }
        }
        Object::Commit(commit) => {
            print_commit(&hash, &commit);
        }
    }
    
    Ok(())
}

/// Create a branch
pub fn branch_create(name: &str) -> anyhow::Result<()> {
    let repo = Repository::open(&env::current_dir()?)?;
    repo.create_branch(name)?;
    println!("Created branch '{}'", name);
    Ok(())
}

/// Delete a branch
pub fn branch_delete(name: &str) -> anyhow::Result<()> {
    let repo = Repository::open(&env::current_dir()?)?;
    repo.delete_branch(name)?;
    println!("Deleted branch '{}'", name);
    Ok(())
}

/// List branches
pub fn branch_list() -> anyhow::Result<()> {
    let repo = Repository::open(&env::current_dir()?)?;
    let branches = repo.list_branches()?;
    let current = repo.current_branch()?;
    
    if branches.is_empty() {
        println!("No branches yet");
        return Ok(());
    }
    
    for branch in branches {
        if Some(&branch) == current.as_ref() {
            println!("* {}", branch);
        } else {
            println!("  {}", branch);
        }
    }
    
    Ok(())
}

/// Checkout a branch
pub fn checkout(branch: &str) -> anyhow::Result<()> {
    let repo = Repository::open(&env::current_dir()?)?;
    repo.checkout(branch)?;
    println!("Switched to branch '{}'", branch);
    Ok(())
}

/// Create a tag
pub fn tag_create(name: &str) -> anyhow::Result<()> {
    let repo = Repository::open(&env::current_dir()?)?;
    repo.create_tag(name)?;
    println!("Created tag '{}'", name);
    Ok(())
}

/// List tags
pub fn tag_list() -> anyhow::Result<()> {
    let repo = Repository::open(&env::current_dir()?)?;
    let tags = repo.list_tags()?;
    
    if tags.is_empty() {
        println!("No tags yet");
        return Ok(());
    }
    
    for tag in tags {
        println!("{}", tag);
    }
    
    Ok(())
}

/// Reset (unstage) files
pub fn reset(paths: &[PathBuf]) -> anyhow::Result<()> {
    let repo = Repository::open(&env::current_dir()?)?;
    
    if paths.is_empty() {
        // Reset all
        repo.reset_all()?;
        println!("Unstaged all changes");
    } else {
        repo.reset(paths)?;
        println!("Unstaged {} file(s)", paths.len());
    }
    
    Ok(())
}

/// Create a new module scaffold
pub fn module_create(
    name: &str,
    path: Option<&Path>,
    author: Option<&str>,
) -> anyhow::Result<()> {
    let target_dir = path
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(name));

    let authors = if let Some(a) = author {
        vec![a.to_string()]
    } else {
        vec![env::var("GVC_AUTHOR")
            .or_else(|_| env::var("USER"))
            .or_else(|_| env::var("USERNAME"))
            .unwrap_or_else(|_| "Unknown".to_string())]
    };

    ModuleManager::create_scaffold(&target_dir, name, authors)?;

    println!("Created module scaffold at: {}", target_dir.display());
    println!();
    println!("Next steps:");
    println!("  1. cd {}", target_dir.display());
    println!("  2. Edit module.toml");
    println!("  3. Add hooks in hooks/");
    println!("  4. gvc module install .");

    Ok(())
}

/// Install a module
pub fn module_install(path: &Path) -> anyhow::Result<()> {
    let repo = Repository::open(&env::current_dir()?)?;
    let manager = ModuleManager::new(repo.gvc_dir())?;

    let identifier = manager.install_local(path)?;

    println!("Installed module: {}", identifier);
    println!("To activate: gvc module add {}", identifier);

    Ok(())
}

/// Activate a module
pub fn module_add(identifier: &str) -> anyhow::Result<()> {
    let repo = Repository::open(&env::current_dir()?)?;
    let manager = ModuleManager::new(repo.gvc_dir())?;

    manager.activate(identifier)?;

    println!("Activated module: {}", identifier);

    Ok(())
}

/// Deactivate a module
pub fn module_remove(identifier: &str) -> anyhow::Result<()> {
    let repo = Repository::open(&env::current_dir()?)?;
    let manager = ModuleManager::new(repo.gvc_dir())?;

    manager.deactivate(identifier)?;

    println!("Deactivated module: {}", identifier);

    Ok(())
}

/// List modules
pub fn module_list(active_only: bool, installed_only: bool) -> anyhow::Result<()> {
    let repo = Repository::open(&env::current_dir()?)?;
    let manager = ModuleManager::new(repo.gvc_dir())?;

    if active_only {
        let active = manager.list_active()?;

        if active.is_empty() {
            println!("No active modules");
        } else {
            println!("Active modules:");
            for module in active {
                println!("  {}", module);
            }
        }
    } else if installed_only {
        let installed = manager.list_installed()?;

        if installed.is_empty() {
            println!("No installed modules");
        } else {
            println!("Installed modules:");
            for module in installed {
                println!("  {}", module);
            }
        }
    } else {
        // Show both
        let installed = manager.list_installed()?;
        let active = manager.list_active()?;

        println!("Installed modules:");
        if installed.is_empty() {
            println!("  (none)");
        } else {
            for module in &installed {
                let is_active = active.contains(module);
                if is_active {
                    println!("  \x1b[32m● {}\x1b[0m (active)", module);
                } else {
                    println!("  ○ {}", module);
                }
            }
        }

        if !active.is_empty() {
            println!();
            println!("Active in this repository:");
            for module in active {
                println!("  ● {}", module);
            }
        }
    }

    Ok(())
}

/// Show module information
pub fn module_info(identifier: &str) -> anyhow::Result<()> {
    let repo = Repository::open(&env::current_dir()?)?;
    let manager = ModuleManager::new(repo.gvc_dir())?;

    // Try to load manifest
    let manifest = manager.get_manifest(identifier, false)?;

    println!("Module: {}", manifest.module.name);
    println!("Version: {}", manifest.module.version);

    if let Some(desc) = &manifest.module.description {
        println!("Description: {}", desc);
    }

    println!("Authors: {}", manifest.module.authors.join(", "));

    if let Some(license) = &manifest.module.license {
        println!("License: {}", license);
    }

    if !manifest.module.keywords.is_empty() {
        println!("Keywords: {}", manifest.module.keywords.join(", "));
    }

    if !manifest.hooks.is_empty() {
        println!();
        println!("Hooks:");
        for (hook_type, hook_path) in &manifest.hooks {
            println!("  {} -> {}", hook_type, hook_path);
        }
    }

    if !manifest.templates.is_empty() {
        println!();
        println!("Templates:");
        for template in &manifest.templates {
            println!("  {}", template.name);
            if let Some(desc) = &template.description {
                println!("    {}", desc);
            }
        }
    }

    if !manifest.dependencies.is_empty() {
        println!();
        println!("Dependencies:");
        for (name, version) in &manifest.dependencies {
            println!("  {} = {}", name, version);
        }
    }

    Ok(())
}

// ============================================================================
// REMOTE COMMANDS (Phase 4)
// ============================================================================

/// Add a remote repository
pub fn remote_add(name: &str, url: &str) -> anyhow::Result<()> {
    let repo = Repository::open(&env::current_dir()?)?;
    let mut manager = RemoteManager::new(repo.gvc_dir())?;
    
    manager.add(name, url)?;
    println!("Added remote '{}' -> {}", name, url);
    
    Ok(())
}

/// Remove a remote repository
pub fn remote_remove(name: &str) -> anyhow::Result<()> {
    let repo = Repository::open(&env::current_dir()?)?;
    let mut manager = RemoteManager::new(repo.gvc_dir())?;
    
    manager.remove(name)?;
    println!("Removed remote '{}'", name);
    
    Ok(())
}

/// Rename a remote repository
pub fn remote_rename(old_name: &str, new_name: &str) -> anyhow::Result<()> {
    let repo = Repository::open(&env::current_dir()?)?;
    let mut manager = RemoteManager::new(repo.gvc_dir())?;
    
    manager.rename(old_name, new_name)?;
    println!("Renamed remote '{}' to '{}'", old_name, new_name);
    
    Ok(())
}

/// List remote repositories
pub fn remote_list(verbose: bool) -> anyhow::Result<()> {
    let repo = Repository::open(&env::current_dir()?)?;
    let manager = RemoteManager::new(repo.gvc_dir())?;
    
    let remotes = manager.list();
    
    if remotes.is_empty() {
        println!("No remotes configured");
    } else {
        for remote in remotes {
            if verbose {
                println!("{}\t{} (fetch)", remote.name, remote.url);
                println!("{}\t{} (push)", remote.name, remote.url);
            } else {
                println!("{}", remote.name);
            }
        }
    }
    
    Ok(())
}

/// Push to a remote repository
pub fn push(remote_name: &str, branch: Option<&str>, force: bool) -> anyhow::Result<()> {
    let repo = Repository::open(&env::current_dir()?)?;
    let manager = RemoteManager::new(repo.gvc_dir())?;
    
    // Get remote URL
    let remote = manager.get(remote_name)
        .ok_or_else(|| anyhow::anyhow!("Remote '{}' not found", remote_name))?;
    
    // Determine branch to push
    let branch = if let Some(b) = branch {
        b.to_string()
    } else {
        repo.current_branch()?
            .ok_or_else(|| anyhow::anyhow!("Not currently on a branch"))?
    };
    
    let ref_path = format!("refs/heads/{}", branch);
    let local_oid = repo.resolve_ref(&ref_path)?
        .ok_or_else(|| anyhow::anyhow!("Branch '{}' not found", branch))?;
    
    println!("Pushing {} to {}/{}...", branch, remote_name, branch);
    
    // Create client
    let client = RemoteClient::new(&remote.url)?;
    
    // Get remote refs
    let remote_refs = client.list_refs("default")?;
    let remote_oid = remote_refs.get(&ref_path).cloned();
    
    // Collect objects to push
    let objects_to_push = collect_objects_to_push(&repo, &local_oid, remote_oid.as_ref())?;
    
    println!("Uploading {} object(s)...", objects_to_push.len());
    
    // Create ref update
    let ref_update = gvc_core::protocol::RefUpdate {
        name: ref_path.clone(),
        old_oid: remote_oid,
        new_oid: local_oid,
        force,
    };
    
    // Push
    let result = client.push("default", objects_to_push, vec![ref_update])?;
    println!("{}", result);
    
    Ok(())
}

/// Fetch from a remote repository
pub fn fetch(remote_name: &str) -> anyhow::Result<()> {
    let repo = Repository::open(&env::current_dir()?)?;
    let manager = RemoteManager::new(repo.gvc_dir())?;
    
    // Get remote URL
    let remote = manager.get(remote_name)
        .ok_or_else(|| anyhow::anyhow!("Remote '{}' not found", remote_name))?;
    
    println!("Fetching from {}...", remote_name);
    
    // Create client
    let client = RemoteClient::new(&remote.url)?;
    
    // List remote refs
    let remote_refs = client.list_refs("default")?;
    
    println!("Found {} reference(s)", remote_refs.len());
    
    // Collect objects we need
    let mut objects_to_fetch = Vec::new();
    for (ref_name, oid) in &remote_refs {
        // Check if we have this object
        if repo.read_object(oid).is_err() {
            objects_to_fetch.push(oid.clone());
        }
    }
    
    if objects_to_fetch.is_empty() {
        println!("Already up to date");
        return Ok(());
    }
    
    println!("Downloading {} object(s)...", objects_to_fetch.len());
    
    // Fetch objects
    let objects = client.get_objects("default", &objects_to_fetch)?;
    
    // Write objects to local storage
    for obj_data in objects {
        repo.write_object_raw(&obj_data.oid, &obj_data.data)?;
    }
    
    // Update remote-tracking refs
    for (ref_name, oid) in remote_refs {
        if ref_name.starts_with("refs/heads/") {
            let tracking_ref = ref_name.replace("refs/heads/", &format!("refs/remotes/{}/", remote_name));
            repo.update_ref(&tracking_ref, &oid)?;
        }
    }
    
    println!("Fetch complete");
    
    Ok(())
}

/// Pull from a remote repository (fetch + merge)
pub fn pull(remote_name: &str, branch: Option<&str>) -> anyhow::Result<()> {
    // For now, just fetch - merging will be implemented in Phase 6
    fetch(remote_name)?;
    
    println!();
    println!("Note: Auto-merge not yet implemented (Phase 6)");
    println!("Use 'gvc checkout' to switch to the fetched branch manually");
    
    Ok(())
}

/// Clone a remote repository
pub fn clone(url: &str, directory: Option<&Path>) -> anyhow::Result<()> {
    // Determine target directory
    let target_dir = if let Some(dir) = directory {
        dir.to_path_buf()
    } else {
        // Extract repository name from URL
        let repo_name = url.split('/').last()
            .and_then(|s| s.strip_suffix(".git").or(Some(s)))
            .unwrap_or("repository");
        PathBuf::from(repo_name)
    };
    
    if target_dir.exists() {
        return Err(anyhow::anyhow!("Directory '{}' already exists", target_dir.display()));
    }
    
    println!("Cloning into '{}'...", target_dir.display());
    
    // Create and initialize repository
    std::fs::create_dir_all(&target_dir)?;
    let repo = Repository::init(&target_dir)?;
    
    // Add remote
    let mut manager = RemoteManager::new(repo.gvc_dir())?;
    manager.add("origin", url)?;
    
    println!("Added remote 'origin' -> {}", url);
    
    // Fetch all refs
    let client = RemoteClient::new(url)?;
    let remote_refs = client.list_refs("default")?;
    
    if remote_refs.is_empty() {
        println!("Remote repository is empty");
        return Ok(());
    }
    
    println!("Fetching {} reference(s)...", remote_refs.len());
    
    // Collect all objects
    let all_oids: Vec<_> = remote_refs.values().cloned().collect();
    let objects = client.get_objects("default", &all_oids)?;
    
    println!("Downloading {} object(s)...", objects.len());
    
    // Write all objects
    for obj_data in objects {
        repo.write_object_raw(&obj_data.oid, &obj_data.data)?;
    }
    
    // Update refs
    for (ref_name, oid) in &remote_refs {
        if ref_name.starts_with("refs/heads/") {
            let tracking_ref = ref_name.replace("refs/heads/", "refs/remotes/origin/");
            repo.update_ref(&tracking_ref, oid)?;
        }
    }
    
    // Determine default branch (prefer 'main', then 'master', then first available)
    let default_branch = if remote_refs.contains_key("refs/heads/main") {
        "main"
    } else if remote_refs.contains_key("refs/heads/master") {
        "master"
    } else {
        remote_refs.keys()
            .find(|k| k.starts_with("refs/heads/"))
            .and_then(|k| k.strip_prefix("refs/heads/"))
            .unwrap_or("main")
    };
    
    // Checkout default branch if it exists
    if let Some(oid) = remote_refs.get(&format!("refs/heads/{}", default_branch)) {
        repo.update_ref(&format!("refs/heads/{}", default_branch), oid)?;
        env::set_current_dir(&target_dir)?;
        repo.checkout(default_branch)?;
        println!("Checked out branch '{}'", default_branch);
    }
    
    println!("Clone complete");
    
    Ok(())
}

/// Collect objects that need to be pushed
fn collect_objects_to_push(
    repo: &Repository,
    local_oid: &Hash,
    remote_oid: Option<&Hash>,
) -> anyhow::Result<Vec<ObjectData>> {
    let mut objects = Vec::new();
    let mut visited = HashSet::new();
    let mut to_visit = vec![local_oid.clone()];
    
    // Mark remote objects as visited (we don't need to send them)
    if let Some(remote) = remote_oid {
        mark_reachable_objects(repo, remote, &mut visited)?;
    }
    
    // Collect all reachable objects from local_oid
    while let Some(oid) = to_visit.pop() {
        if visited.contains(&oid) {
            continue;
        }
        visited.insert(oid.clone());
        
        let raw_data = repo.read_object_raw(&oid)?;
        let obj = repo.read_object(&oid)?;
        
        let obj_type = match obj {
            Object::Blob(_) => gvc_core::protocol::ObjectType::Blob,
            Object::Tree(_) => gvc_core::protocol::ObjectType::Tree,
            Object::Commit(_) => gvc_core::protocol::ObjectType::Commit,
        };
        
        objects.push(ObjectData {
            oid: oid.clone(),
            data: raw_data,
            object_type: obj_type,
        });
        
        // Add referenced objects to visit
        match obj {
            Object::Commit(commit) => {
                to_visit.push(commit.tree.clone());
                to_visit.extend(commit.parents.iter().cloned());
            }
            Object::Tree(tree) => {
                for entry in tree.entries.values() {
                    to_visit.push(entry.hash.clone());
                }
            }
            Object::Blob(_) => {}
        }
    }
    
    Ok(objects)
}

/// Mark all objects reachable from a given OID
fn mark_reachable_objects(
    repo: &Repository,
    oid: &Hash,
    visited: &mut HashSet<Hash>,
) -> anyhow::Result<()> {
    if visited.contains(oid) {
        return Ok(());
    }
    visited.insert(oid.clone());
    
    let obj = repo.read_object(oid)?;
    
    match obj {
        Object::Commit(commit) => {
            mark_reachable_objects(repo, &commit.tree, visited)?;
            for parent in &commit.parents {
                mark_reachable_objects(repo, parent, visited)?;
            }
        }
        Object::Tree(tree) => {
            for entry in tree.entries.values() {
                mark_reachable_objects(repo, &entry.hash, visited)?;
            }
        }
        Object::Blob(_) => {}
    }
    
    Ok(())
}

