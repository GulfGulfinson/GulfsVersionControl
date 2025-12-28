// Integration tests for GVC CLI commands

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

// Test helper to get the gvc binary path
fn gvc_bin() -> PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // Remove test binary name
    path.pop(); // Remove 'deps'
    path.push("gvc");
    
    if cfg!(windows) {
        path.set_extension("exe");
    }
    
    path
}

#[test]
fn test_init_creates_repository() {
    let temp = TempDir::new().unwrap();
    let repo_path = temp.path();
    
    let output = Command::new(gvc_bin())
        .arg("init")
        .arg(repo_path)
        .output()
        .expect("Failed to execute gvc init");
    
    assert!(output.status.success());
    assert!(repo_path.join(".gvc").exists());
    assert!(repo_path.join(".gvc/objects").exists());
    assert!(repo_path.join(".gvc/refs/heads").exists());
    assert!(repo_path.join(".gvc/config").exists());
}

#[test]
fn test_init_already_exists() {
    let temp = TempDir::new().unwrap();
    let repo_path = temp.path();
    
    // First init should succeed
    Command::new(gvc_bin())
        .arg("init")
        .arg(repo_path)
        .output()
        .expect("Failed to execute gvc init");
    
    // Second init should fail
    let output = Command::new(gvc_bin())
        .arg("init")
        .arg(repo_path)
        .output()
        .expect("Failed to execute gvc init");
    
    assert!(!output.status.success());
}

#[test]
fn test_add_and_status() {
    let temp = TempDir::new().unwrap();
    let repo_path = temp.path();
    
    // Init repository
    Command::new(gvc_bin())
        .arg("init")
        .arg(repo_path)
        .output()
        .unwrap();
    
    // Create a test file
    let test_file = repo_path.join("test.txt");
    fs::write(&test_file, b"hello world").unwrap();
    
    // Add the file
    let output = Command::new(gvc_bin())
        .current_dir(repo_path)
        .arg("add")
        .arg("test.txt")
        .output()
        .unwrap();
    
    assert!(output.status.success());
    
    // Check status
    let output = Command::new(gvc_bin())
        .current_dir(repo_path)
        .arg("status")
        .output()
        .unwrap();
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("test.txt") || stdout.contains("new file"));
}

#[test]
fn test_commit() {
    let temp = TempDir::new().unwrap();
    let repo_path = temp.path();
    
    // Init repository
    Command::new(gvc_bin())
        .arg("init")
        .arg(repo_path)
        .output()
        .unwrap();
    
    // Create and add a file
    let test_file = repo_path.join("test.txt");
    fs::write(&test_file, b"hello world").unwrap();
    
    Command::new(gvc_bin())
        .current_dir(repo_path)
        .arg("add")
        .arg("test.txt")
        .output()
        .unwrap();
    
    // Commit
    let output = Command::new(gvc_bin())
        .current_dir(repo_path)
        .arg("commit")
        .arg("-m")
        .arg("Test commit")
        .output()
        .unwrap();
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Test commit"));
}

#[test]
fn test_log() {
    let temp = TempDir::new().unwrap();
    let repo_path = temp.path();
    
    // Setup: init, add, commit
    Command::new(gvc_bin()).arg("init").arg(repo_path).output().unwrap();
    
    let test_file = repo_path.join("test.txt");
    fs::write(&test_file, b"content").unwrap();
    
    Command::new(gvc_bin())
        .current_dir(repo_path)
        .arg("add")
        .arg("test.txt")
        .output()
        .unwrap();
    
    Command::new(gvc_bin())
        .current_dir(repo_path)
        .arg("commit")
        .arg("-m")
        .arg("First commit")
        .output()
        .unwrap();
    
    // Check log
    let output = Command::new(gvc_bin())
        .current_dir(repo_path)
        .arg("log")
        .output()
        .unwrap();
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("First commit"));
}

#[test]
fn test_branch_create_and_list() {
    let temp = TempDir::new().unwrap();
    let repo_path = temp.path();
    
    // Setup: init, add, commit (need at least one commit for branches)
    Command::new(gvc_bin()).arg("init").arg(repo_path).output().unwrap();
    
    let test_file = repo_path.join("test.txt");
    fs::write(&test_file, b"content").unwrap();
    
    Command::new(gvc_bin())
        .current_dir(repo_path)
        .arg("add")
        .arg("test.txt")
        .output()
        .unwrap();
    
    Command::new(gvc_bin())
        .current_dir(repo_path)
        .arg("commit")
        .arg("-m")
        .arg("Initial commit")
        .output()
        .unwrap();
    
    // Create a new branch
    let output = Command::new(gvc_bin())
        .current_dir(repo_path)
        .arg("branch")
        .arg("create")
        .arg("feature")
        .output()
        .unwrap();
    
    assert!(output.status.success());
    
    // List branches
    let output = Command::new(gvc_bin())
        .current_dir(repo_path)
        .arg("branch")
        .arg("list")
        .output()
        .unwrap();
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("feature"));
    assert!(stdout.contains("main"));
}

#[test]
fn test_diff() {
    let temp = TempDir::new().unwrap();
    let repo_path = temp.path();
    
    // Setup
    Command::new(gvc_bin()).arg("init").arg(repo_path).output().unwrap();
    
    let test_file = repo_path.join("test.txt");
    fs::write(&test_file, b"line1\n").unwrap();
    
    Command::new(gvc_bin())
        .current_dir(repo_path)
        .arg("add")
        .arg("test.txt")
        .output()
        .unwrap();
    
    Command::new(gvc_bin())
        .current_dir(repo_path)
        .arg("commit")
        .arg("-m")
        .arg("Initial")
        .output()
        .unwrap();
    
    // Modify file
    fs::write(&test_file, b"line1\nline2\n").unwrap();
    
    // Check diff
    let output = Command::new(gvc_bin())
        .current_dir(repo_path)
        .arg("diff")
        .output()
        .unwrap();
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("test.txt") || stdout.contains("line2"));
}

#[test]
fn test_checkout() {
    let temp = TempDir::new().unwrap();
    let repo_path = temp.path();
    
    // Setup with initial commit
    Command::new(gvc_bin()).arg("init").arg(repo_path).output().unwrap();
    
    let test_file = repo_path.join("test.txt");
    fs::write(&test_file, b"content").unwrap();
    
    Command::new(gvc_bin())
        .current_dir(repo_path)
        .arg("add")
        .arg("test.txt")
        .output()
        .unwrap();
    
    Command::new(gvc_bin())
        .current_dir(repo_path)
        .arg("commit")
        .arg("-m")
        .arg("Initial")
        .output()
        .unwrap();
    
    // Create and checkout branch
    Command::new(gvc_bin())
        .current_dir(repo_path)
        .arg("branch")
        .arg("create")
        .arg("feature")
        .output()
        .unwrap();
    
    let output = Command::new(gvc_bin())
        .current_dir(repo_path)
        .arg("checkout")
        .arg("feature")
        .output()
        .unwrap();
    
    assert!(output.status.success());
}

#[test]
fn test_tag() {
    let temp = TempDir::new().unwrap();
    let repo_path = temp.path();
    
    // Setup
    Command::new(gvc_bin()).arg("init").arg(repo_path).output().unwrap();
    
    let test_file = repo_path.join("test.txt");
    fs::write(&test_file, b"content").unwrap();
    
    Command::new(gvc_bin())
        .current_dir(repo_path)
        .arg("add")
        .arg("test.txt")
        .output()
        .unwrap();
    
    Command::new(gvc_bin())
        .current_dir(repo_path)
        .arg("commit")
        .arg("-m")
        .arg("v1.0")
        .output()
        .unwrap();
    
    // Create tag
    let output = Command::new(gvc_bin())
        .current_dir(repo_path)
        .arg("tag")
        .arg("create")
        .arg("v1.0.0")
        .output()
        .unwrap();
    
    assert!(output.status.success());
    
    // List tags
    let output = Command::new(gvc_bin())
        .current_dir(repo_path)
        .arg("tag")
        .arg("list")
        .output()
        .unwrap();
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("v1.0.0"));
}

#[test]
fn test_reset() {
    let temp = TempDir::new().unwrap();
    let repo_path = temp.path();
    
    // Setup
    Command::new(gvc_bin()).arg("init").arg(repo_path).output().unwrap();
    
    let test_file = repo_path.join("test.txt");
    fs::write(&test_file, b"content").unwrap();
    
    // Add file
    Command::new(gvc_bin())
        .current_dir(repo_path)
        .arg("add")
        .arg("test.txt")
        .output()
        .unwrap();
    
    // Reset (unstage)
    let output = Command::new(gvc_bin())
        .current_dir(repo_path)
        .arg("reset")
        .output()
        .unwrap();
    
    assert!(output.status.success());
}

