# GVC Quick Start Guide

Get up and running with GulfsVersionControl in 5 minutes.

## Step 1: Install Rust

**Already have Rust?** Skip to Step 2.

### Windows
```powershell
# Download and install rustup
Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile "$env:TEMP\rustup-init.exe"
& "$env:TEMP\rustup-init.exe" -y

# Restart your terminal
```

### Linux/macOS
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

**Verify:**
```bash
rustc --version
cargo --version
```

## Step 2: Build GVC

```bash
cd GulfsControlSystem
cargo build --release
cargo install --path gvc-cli
```

**Verify:**
```bash
gvc --version
```

## Step 3: Your First Repository

```bash
# Create a project
mkdir my-project
cd my-project

# Initialize GVC
gvc init
```

**Output:**
```
Initialized empty GVC repository in my-project
```

## Step 4: Create and Commit Files

```bash
# Create files
echo "# My Project" > README.md
echo "Hello, GVC!" > hello.txt

# Stage files
gvc add .

# Check status
gvc status

# Commit
gvc commit -m "Initial commit"
```

**Output:**
```
[a1b2c3d] Initial commit
```

## Step 5: View History

```bash
gvc log
```

**Output:**
```
commit a1b2c3d4e5f6...
Author: YourName
Date:   Fri Dec 26 10:30:00 2025 +0000

    Initial commit
```

## Step 6: Work with Branches

```bash
# Create a branch
gvc branch create feature-x

# List branches
gvc branch list

# Switch to branch
gvc checkout feature-x

# Make changes
echo "New feature" >> feature.txt
gvc add feature.txt
gvc commit -m "Add feature X"

# Switch back
gvc checkout main
```

## Step 7: Create a Tag

```bash
gvc tag create v1.0
gvc tag list
```

## Common Commands

```bash
# Initialize
gvc init

# Stage files
gvc add <file>
gvc add .                    # Add all files

# Commit
gvc commit -m "message"
gvc commit -m "msg" --author "Name"

# View history
gvc log
gvc log -n 5                 # Last 5 commits
gvc log --oneline            # Compact format

# Status
gvc status

# Branches
gvc branch create <name>
gvc branch delete <name>
gvc branch list
gvc checkout <branch>

# Tags
gvc tag create <name>
gvc tag list

# Inspect objects
gvc show <hash>
```

## Example Workflow

```bash
# Start a new project
mkdir my-app && cd my-app
gvc init

# Create initial files
echo "# My App" > README.md
echo "fn main() {}" > main.rs

# First commit
gvc add .
gvc commit -m "Initial commit"

# Create feature branch
gvc branch create feature-auth
gvc checkout feature-auth

# Develop feature
echo "fn login() {}" >> auth.rs
gvc add auth.rs
gvc commit -m "Add authentication"

# More commits
echo "fn logout() {}" >> auth.rs
gvc add auth.rs
gvc commit -m "Add logout"

# View history
gvc log --oneline

# Tag release
gvc checkout main
gvc tag create v1.0

# Continue development
gvc branch create feature-api
gvc checkout feature-api
```

## Tips

1. **Set author globally:**
   ```bash
   # Windows
   $env:GVC_AUTHOR = "Your Name"
   
   # Linux/macOS
   export GVC_AUTHOR="Your Name"
   ```

2. **View short hashes:**
   ```bash
   gvc log --oneline
   ```

3. **GVC finds repository automatically:**
   ```bash
   cd my-project/src/subdir
   gvc status  # Works from anywhere in project
   ```

4. **Inspect any object:**
   ```bash
   gvc show a1b2c3d
   ```

## Troubleshooting

### "cargo: command not found"
- Restart your terminal after installing Rust
- Check PATH includes `~/.cargo/bin`

### "Repository not found"
- Make sure you're inside a GVC repository
- Run `gvc init` to create one

### "Nothing to commit"
- Stage files first: `gvc add <file>`

## Next Steps

- Read [USAGE.md](USAGE.md) for detailed usage
- Check [ARCHITECTURE.md](ARCHITECTURE.md) to understand internals
- See [ROADMAP.md](ROADMAP.md) for upcoming features
- Try the test script: `.\test-gvc.ps1` or `./test-gvc.sh`

## Getting Help

```bash
gvc --help
gvc <command> --help
```

---

**Ready to explore?** Start building with GVC! 🚀

