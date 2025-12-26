# GulfsVersionControl - Complete Tutorial

Learn GVC from scratch to advanced usage.

---

## Table of Contents

1. [Installation](#installation)
2. [Basic Workflow](#basic-workflow)
3. [Branching](#branching)
4. [Diff & Status](#diff--status)
5. [Ignoring Files](#ignoring-files)
6. [Modules](#modules)
7. [Hooks](#hooks)
8. [Advanced Topics](#advanced-topics)

---

## Installation

### 1. Install Rust

**Windows:**
```powershell
Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile "$env:TEMP\rustup-init.exe"
& "$env:TEMP\rustup-init.exe" -y
# Restart terminal
```

**Linux/macOS:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 2. Build GVC

```bash
cd GulfsControlSystem
cargo build --release
cargo test  # Optional: run tests
```

### 3. Install GVC

```bash
cargo install --path gvc-cli
```

Verify:
```bash
gvc --version
gvc --help
```

---

## Basic Workflow

### Initialize a Repository

```bash
mkdir my-project
cd my-project
gvc init
```

Output:
```
Initialized empty GVC repository in my-project
```

This creates `.gvc/` directory:
```
.gvc/
├── objects/
├── refs/
│   ├── heads/
│   └── tags/
├── index
├── config
└── HEAD
```

### Create Files

```bash
echo "# My Project" > README.md
echo "fn main() { println!(\"Hello, GVC!\"); }" > main.rs
```

### Check Status

```bash
gvc status
```

Output:
```
On branch main

Untracked files:
  (use "gvc add <file>..." to include in what will be committed)

  README.md
  main.rs
```

### Stage Files

```bash
# Stage specific files
gvc add README.md main.rs

# Or stage all
gvc add .
```

### Check Status Again

```bash
gvc status
```

Output:
```
On branch main

Changes to be committed:
  (use "gvc reset <file>..." to unstage)

  new file:   README.md
  new file:   main.rs
```

### Commit

```bash
gvc commit -m "Initial commit"
```

Output:
```
[a1b2c3d] Initial commit
```

### View History

```bash
gvc log
```

Output:
```
commit a1b2c3d4e5f6789012345678901234567890123456789012345678901234
Author: Your Name
Date:   Fri Dec 26 10:30:00 2025 +0000

    Initial commit
```

Compact format:
```bash
gvc log --oneline
```

Output:
```
a1b2c3d Initial commit
```

---

## Branching

### Create a Branch

```bash
gvc branch create feature-auth
```

### List Branches

```bash
gvc branch list
```

Output:
```
* main
  feature-auth
```

### Switch Branches

```bash
gvc checkout feature-auth
```

Or use alias:
```bash
gvc switch feature-auth
```

### Make Changes on Branch

```bash
echo "fn authenticate() { /* TODO */ }" >> auth.rs
gvc add auth.rs
gvc commit -m "Add authentication stub"
```

### Switch Back to Main

```bash
gvc checkout main
```

Notice: `auth.rs` is gone (it only exists on feature-auth branch).

### Delete a Branch

```bash
gvc branch delete feature-auth
```

**Note:** Can't delete current branch.

---

## Diff & Status

### Make Changes

```bash
echo "More content" >> README.md
```

### See Unstaged Changes

```bash
gvc diff
```

Output (colorized):
```diff
diff --gvc a/README.md b/README.md
@@ -1,1 +1,2 @@
 # My Project
+More content
```

### Stage Changes

```bash
gvc add README.md
```

### See Staged Changes

```bash
gvc diff --staged
```

### Detailed Status

```bash
gvc status
```

Output:
```
On branch main

Changes to be committed:
  (use "gvc reset <file>..." to unstage)

  modified:   README.md
```

### Unstage Files

```bash
gvc reset README.md
```

Now the change is unstaged:
```bash
gvc status
```

Output:
```
On branch main

Changes not staged for commit:
  (use "gvc add <file>..." to update what will be committed)

  modified:   README.md
```

---

## Ignoring Files

### Create .gvcignore

```bash
cat > .gvcignore << 'EOF'
# Build artifacts
target/
*.o
*.exe

# Temporary files
*.tmp
*.log

# But keep important.log
!important.log

# IDE files
.vscode/
.idea/

# Dependencies
node_modules/
EOF
```

### Test Ignore

```bash
# Create files
touch debug.log important.log target/build.o

gvc status
```

Output:
```
On branch main

Untracked files:
  .gvcignore
  important.log
```

Notice: `debug.log` and `target/build.o` are not shown (ignored).

### Pattern Examples

```
*.log           # All .log files
*.tmp           # All .tmp files
test?.txt       # test1.txt, test2.txt, etc.
**/*.pyc        # All .pyc files recursively
!important.log  # Exception: don't ignore
dir/            # Specific directory
```

---

## Modules

### Create a Module

```bash
gvc module create my-linter --author "Your Name"
```

Output:
```
Created module scaffold at: my-linter

Next steps:
  1. cd my-linter
  2. Edit module.toml
  3. Add hooks in hooks/
  4. gvc module install .
```

### Module Structure

```
my-linter/
├── module.toml
├── hooks/
│   └── pre-commit.sh.sample
├── templates/
└── README.md
```

### Edit Module

```bash
cd my-linter
```

Edit `module.toml`:
```toml
[module]
name = "my-linter"
version = "0.1.0"
description = "Lints code before commit"
authors = ["Your Name"]

[hooks]
pre-commit = "hooks/lint.sh"
```

Create hook `hooks/lint.sh`:
```bash
#!/bin/bash
echo "Running linter..."
# Your linting logic here
exit 0
```

Make executable:
```bash
chmod +x hooks/lint.sh
```

### Install Module

```bash
gvc module install .
```

Output:
```
Installed module: my-linter@0.1.0
To activate: gvc module add my-linter@0.1.0
```

Module is now in `~/.gvc/modules/my-linter@0.1.0`

### Activate Module

```bash
cd ../my-project
gvc module add my-linter@0.1.0
```

Output:
```
Activated module: my-linter@0.1.0
```

### List Modules

```bash
gvc module list
```

Output:
```
Installed modules:
  ● my-linter@0.1.0 (active)

Active in this repository:
  ● my-linter@0.1.0
```

### Test Hook

```bash
echo "test" > file.txt
gvc add file.txt
gvc commit -m "test"
```

Output:
```
Running linter...
✅ Commit successful!
```

### Module Info

```bash
gvc module info my-linter@0.1.0
```

Output:
```
Module: my-linter
Version: 0.1.0
Description: Lints code before commit
Authors: Your Name

Hooks:
  pre-commit -> hooks/lint.sh
```

### Deactivate Module

```bash
gvc module remove my-linter@0.1.0
```

---

## Hooks

### Hook Types

| Hook | When | Can Block |
|------|------|-----------|
| `pre-commit` | Before commit | ✅ Yes |
| `post-commit` | After commit | ❌ No |
| `pre-checkout` | Before checkout | ✅ Yes |
| `post-checkout` | After checkout | ❌ No |
| `pre-push` | Before push (Phase 4) | ✅ Yes |
| `post-pull` | After pull (Phase 4) | ❌ No |

### Repository Hooks

Hooks in `.gvc/hooks/`:

**Create pre-commit hook:**

```bash
# Windows (PowerShell)
cat > .gvc/hooks/pre-commit.ps1 << 'EOF'
Write-Host "Running pre-commit hook"
# Your validation
exit 0
EOF

# Linux/macOS (Bash)
cat > .gvc/hooks/pre-commit.sh << 'EOF'
#!/bin/bash
echo "Running pre-commit hook"
# Your validation
exit 0
EOF
chmod +x .gvc/hooks/pre-commit.sh
```

### Hook Environment Variables

Available in hooks:

```bash
GVC_AUTHOR="..."        # Commit author
GVC_MESSAGE="..."       # Commit message
GVC_COMMIT_HASH="..."   # Commit hash (post-commit)
GVC_BRANCH="..."        # Branch name (checkout)
MODULE_NAME="..."       # Module name (module hooks only)
MODULE_PATH="..."       # Module path (module hooks only)
```

Example:
```bash
#!/bin/bash
echo "Committing as: $GVC_AUTHOR"
echo "Message: $GVC_MESSAGE"
```

### Block Commit Example

```bash
cat > .gvc/hooks/pre-commit.sh << 'EOF'
#!/bin/bash
if echo "$GVC_MESSAGE" | grep -qi "TODO"; then
    echo "Error: Commit message contains 'TODO'"
    exit 1
fi
exit 0
EOF
chmod +x .gvc/hooks/pre-commit.sh
```

Test:
```bash
gvc commit -m "TODO: fix this"
# Error: Commit message contains 'TODO'
# Commit blocked!
```

---

## Advanced Topics

### Working with Tags

```bash
# Create tag
gvc tag create v1.0

# List tags
gvc tag list

# Show tagged commit
gvc show v1.0  # Coming in Phase 4
```

### Inspecting Objects

```bash
# Get commit hash
HASH=$(gvc log --oneline | head -1 | cut -d' ' -f1)

# Show object
gvc show $HASH
```

### Multiple Files at Once

```bash
# Stage multiple files
gvc add file1.txt file2.rs file3.md

# Stage by pattern (shell expansion)
gvc add *.rs

# Stage everything
gvc add .
```

### Author Configuration

```bash
# Set author for session
export GVC_AUTHOR="Your Name"

# Or per-commit
gvc commit -m "message" --author "Your Name"
```

### Repository Structure

```
.gvc/
├── objects/           # All objects (blobs, trees, commits)
│   ├── 00/
│   ├── 01/
│   └── ...
├── refs/
│   ├── heads/         # Branches
│   │   └── main
│   └── tags/          # Tags
│       └── v1.0
├── modules/
│   └── active/        # Active modules (symlinks)
├── hooks/             # Repository hooks
├── index              # Staging area
├── config             # Repository config
└── HEAD               # Current branch/commit
```

---

## Real-World Example

### Complete Project Setup

```bash
# 1. Create Rust project
cargo new my-app
cd my-app

# 2. Initialize GVC
gvc init

# 3. Create .gvcignore
cat > .gvcignore << 'EOF'
target/
Cargo.lock
*.swp
EOF

# 4. Initial commit
gvc add .
gvc commit -m "feat: initial project structure"

# 5. Install linter module
gvc module install ../examples/rust-linter-module
gvc module add rust-linter@1.0.0

# 6. Install commit convention module
gvc module install ../examples/commit-convention-module
gvc module add commit-convention@1.0.0

# 7. Create feature branch
gvc branch create feature-add-cli
gvc checkout feature-add-cli

# 8. Make changes
# ... edit src/main.rs ...
gvc add src/main.rs
gvc commit -m "feat(cli): add command-line argument parsing"

# 9. Switch back and tag
gvc checkout main
gvc tag create v0.1.0

# 10. View history
gvc log --oneline
```

---

## Tips & Tricks

### 1. Aliases

**Bash:**
```bash
alias gs='gvc status'
alias ga='gvc add'
alias gc='gvc commit'
alias gl='gvc log --oneline'
alias gd='gvc diff'
```

### 2. Check Before Commit

```bash
gvc status && gvc diff --staged && gvc commit -m "message"
```

### 3. Quick Status Check

```bash
gvc status | grep -E "(Changes|Untracked)"
```

### 4. Find Large Files

```bash
find . -type f -size +1M ! -path "./.gvc/*"
```

### 5. Module Development Workflow

```bash
# Edit module
cd my-module
# ... make changes ...

# Reinstall
gvc module install .

# Test in another repo
cd ../test-repo
gvc commit -m "test"  # Runs updated hooks
```

---

## Troubleshooting

### "Repository not found"

```bash
# Check if initialized
ls -la .gvc

# Initialize if needed
gvc init
```

### "Nothing to commit"

```bash
# Stage files first
gvc add <files>
```

### Hook Fails

```bash
# Check hook script
cat .gvc/hooks/pre-commit.sh

# Make executable (Linux/macOS)
chmod +x .gvc/hooks/pre-commit.sh

# Test manually
bash .gvc/hooks/pre-commit.sh
```

### Module Not Working

```bash
# List active modules
gvc module list

# Check module manifest
cat ~/.gvc/modules/module-name@version/module.toml

# Reinstall
gvc module remove module-name@version
gvc module install /path/to/module
gvc module add module-name@version
```

---

## Next Steps

- Read [USAGE.md](USAGE.md) for complete command reference
- Read [ARCHITECTURE.md](ARCHITECTURE.md) for technical details
- Check [examples/](examples/) for more module examples
- See [ROADMAP.md](ROADMAP.md) for upcoming features

---

**Happy version controlling with GVC!** 🚀

