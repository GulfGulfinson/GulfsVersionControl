# GulfsVersionControl - Usage Guide

## Basic Workflow

### 1. Initialize a Repository

```bash
# Create a new directory
mkdir my-project
cd my-project

# Initialize GVC repository
gvc init
```

This creates a `.gvc/` directory with the repository structure.

### 2. Create and Add Files

```bash
# Create some files
echo "# My Project" > README.md
echo "fn main() {}" > main.rs

# Stage files for commit
gvc add README.md main.rs

# Or add all files in current directory
gvc add .
```

### 3. Commit Changes

```bash
# Commit with a message
gvc commit -m "Initial commit"

# Commit with custom author
gvc commit -m "Add feature" --author "John Doe"
```

**Note:** If no author is specified, GVC uses:
1. `--author` flag
2. `GVC_AUTHOR` environment variable
3. `USER` or `USERNAME` environment variable
4. "Unknown" as fallback

### 4. View History

```bash
# Show all commits
gvc log

# Show last 5 commits
gvc log -n 5

# Show in compact format
gvc log --oneline
```

**Output:**
```
commit a1b2c3d4e5f6...
Author: John Doe
Date:   Fri Dec 26 10:30:00 2025 +0000

    Initial commit
```

### 5. Check Status

```bash
gvc status
```

**Output:**
```
On branch main

Changes to be committed:
  modified:   README.md
  modified:   main.rs
```

## Working with Branches

### Create a Branch

```bash
# Create a new branch from current commit
gvc branch create feature-x
```

### List Branches

```bash
gvc branch list
```

**Output:**
```
* main
  feature-x
```

The `*` indicates the current branch.

### Switch Branches

```bash
# Using checkout
gvc checkout feature-x

# Or using switch (alias)
gvc switch feature-x
```

### Delete a Branch

```bash
gvc branch delete feature-x
```

**Note:** You cannot delete the current branch.

## Working with Tags

### Create a Tag

```bash
# Tag current commit
gvc tag create v1.0
```

### List Tags

```bash
gvc tag list
```

**Output:**
```
v1.0
v1.1
v2.0
```

## Inspecting Objects

### Show Object Content

```bash
# Show commit
gvc show a1b2c3d

# Show tree
gvc show e4f5a6b

# Show blob (file content)
gvc show 7c8d9e0
```

**Output for blob:**
```
blob 7c8d9e0a1b2c3d4e5f6...
size: 42 bytes

Hello, GVC!
```

**Output for tree:**
```
tree e4f5a6b7c8d9e0a1b2c3...

100644 blob   a1b2c3d  README.md
100644 blob   e4f5a6b  main.rs
040000 tree   7c8d9e0  src
```

**Output for commit:**
```
commit a1b2c3d4e5f6...
Author: John Doe
Date:   Fri Dec 26 10:30:00 2025 +0000

    Initial commit
```

## Advanced Usage

### Setting Author Globally

**Windows (PowerShell):**
```powershell
$env:GVC_AUTHOR = "John Doe"
```

**Linux/macOS:**
```bash
export GVC_AUTHOR="John Doe"
```

Add to your shell profile (`.bashrc`, `.zshrc`, etc.) to make permanent.

### Working with Multiple Repositories

```bash
# Initialize repo in specific directory
gvc init /path/to/project

# GVC automatically finds the repository root
cd /path/to/project/src/subdir
gvc status  # Works from any subdirectory
```

### Viewing Specific Number of Commits

```bash
# Last 10 commits
gvc log -n 10

# All commits (default)
gvc log
```

## Repository Structure

After initialization, your repository looks like this:

```
my-project/
├── .gvc/
│   ├── objects/        # Content-addressable storage
│   │   ├── 00/
│   │   ├── 01/
│   │   └── ...
│   ├── refs/
│   │   ├── heads/
│   │   │   └── main    # Branch pointer
│   │   └── tags/
│   │       └── v1.0    # Tag pointer
│   ├── index           # Staging area
│   ├── config          # Repository config
│   └── HEAD            # Current branch/commit
├── README.md
└── main.rs
```

## Understanding Hashes

GVC uses SHA-256 hashes to identify objects:

```
Full hash:  a1b2c3d4e5f6789012345678901234567890123456789012345678901234
Short hash: a1b2c3d (first 7 characters)
```

Most commands accept short hashes for convenience.

## Common Workflows

### Starting a New Project

```bash
mkdir my-app
cd my-app
gvc init
echo "# My App" > README.md
gvc add README.md
gvc commit -m "Initial commit"
```

### Making Changes

```bash
# Edit files
vim main.rs

# Stage changes
gvc add main.rs

# Commit
gvc commit -m "Add main function"
```

### Creating a Feature Branch

```bash
# Create and switch to feature branch
gvc branch create feature-auth
gvc checkout feature-auth

# Make changes
echo "fn authenticate() {}" >> auth.rs
gvc add auth.rs
gvc commit -m "Add authentication"

# Switch back to main
gvc checkout main
```

### Tagging Releases

```bash
# After completing version 1.0
gvc tag create v1.0

# Continue development
gvc commit -m "Start v1.1 development"

# Later, create v1.1 tag
gvc tag create v1.1
```

## Troubleshooting

### "Repository not found"

Make sure you're inside a GVC repository:

```bash
# Check if .gvc directory exists
ls -la .gvc

# Or initialize if needed
gvc init
```

### "Nothing to commit"

You need to stage files first:

```bash
gvc add <files>
gvc commit -m "message"
```

### "Branch already exists"

Choose a different name:

```bash
gvc branch create feature-auth-v2
```

### "Branch not found"

List available branches:

```bash
gvc branch list
```

## Differences from Git

| Operation | Git | GVC |
|-----------|-----|-----|
| Initialize | `git init` | `gvc init` |
| Stage | `git add` | `gvc add` |
| Commit | `git commit` | `gvc commit` |
| History | `git log` | `gvc log` |
| Branch | `git branch` | `gvc branch create/delete/list` |
| Switch | `git checkout` / `git switch` | `gvc checkout` / `gvc switch` |
| Tag | `git tag` | `gvc tag create/list` |

## Coming Soon

- **Diff:** Compare changes between commits
- **Merge:** Combine branches
- **Remote:** Push/pull to remote repositories
- **Modules:** Extensibility system
- **Ignore:** `.gvcignore` file support

## Getting Help

```bash
# General help
gvc --help

# Command-specific help
gvc commit --help
gvc branch --help
```

## Examples

### Complete Example Session

```bash
# Setup
mkdir my-project
cd my-project
gvc init

# Create files
echo "# My Project" > README.md
echo "fn main() { println!(\"Hello\"); }" > main.rs

# Initial commit
gvc add .
gvc commit -m "Initial commit"

# Create feature branch
gvc branch create feature-logging
gvc checkout feature-logging

# Add logging
echo "fn log(msg: &str) { println!(\"{}\", msg); }" >> main.rs
gvc add main.rs
gvc commit -m "Add logging function"

# Tag release
gvc checkout main
gvc tag create v1.0

# View history
gvc log --oneline
```

**Output:**
```
a1b2c3d Add logging function
e4f5a6b Initial commit
```

## Best Practices

1. **Commit often:** Small, focused commits are easier to understand
2. **Write good messages:** Explain *why*, not just *what*
3. **Use branches:** Isolate features and experiments
4. **Tag releases:** Mark important milestones
5. **Check status:** Before committing, review what's staged

## Next Steps

- Read [ARCHITECTURE.md](ARCHITECTURE.md) to understand internals
- See [INSTALLATION.md](INSTALLATION.md) for setup instructions
- Check [README.md](README.md) for project overview

