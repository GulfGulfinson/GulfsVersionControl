use gvc_core::{Commit, Hash, Object, Repository, ModuleManager, ModuleManifest};
use std::env;
use std::path::{Path, PathBuf};

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

