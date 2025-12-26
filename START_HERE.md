# 🚀 Start Here - GulfsVersionControl

**Welcome to GVC!** This guide will get you started in 5 minutes.

---

## What is GVC?

GVC (GulfsVersionControl) is a **complete version control system** written in Rust, similar to Git but with unique features like a powerful module system.

**Current Status:** Phase 1, 2 & 3 Complete ✅✅✅

---

## Quick Start

### 1. Do you have Rust?

**No?** Install it:

```bash
# Windows (PowerShell)
Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile "$env:TEMP\rustup-init.exe"
& "$env:TEMP\rustup-init.exe" -y

# Linux/macOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Yes?** Great! Continue to step 2.

### 2. Build GVC

```bash
cd GulfsControlSystem
cargo build --release
cargo install --path gvc-cli
```

### 3. Verify Installation

```bash
gvc --version
```

You should see: `gvc 0.3.0`

### 4. First Repository

```bash
# Create project
mkdir my-project
cd my-project

# Initialize GVC
gvc init

# Create file
echo "Hello, GVC!" > hello.txt

# Add and commit
gvc add hello.txt
gvc commit -m "Initial commit"

# View history
gvc log
```

**🎉 Congratulations! You're using GVC!**

---

## What's Next?

### Learn the Basics (15 min)
👉 Read [QUICKSTART.md](QUICKSTART.md)

### Complete Tutorial (1 hour)
👉 Read [TUTORIAL.md](TUTORIAL.md)

### Quick Command Reference
👉 See [CHEATSHEET.md](CHEATSHEET.md)

### Explore Modules
👉 Check [examples/](examples/)

---

## Common Commands

```bash
# Status
gvc status

# Stage files
gvc add <file>
gvc add .                    # All files

# Commit
gvc commit -m "message"

# History
gvc log
gvc log --oneline            # Compact

# Diff
gvc diff                     # Unstaged changes
gvc diff --staged            # Staged changes

# Branches
gvc branch create <name>
gvc checkout <name>
gvc branch list

# Modules
gvc module list
gvc module install <path>
gvc module add <name@version>
```

---

## Key Features

### ✅ Phase 1: Core VCS
- Repository management
- Commits, branches, tags
- Object inspection
- SHA-256 hashing

### ✅ Phase 2: Usability
- **Diff** (Myers' algorithm)
- **Improved status** (colorized)
- **.gvcignore** (pattern matching)
- **Reset** (unstage files)

### ✅ Phase 3: Modules
- **Module system** (create, install, activate)
- **Hook system** (6 types)
- **Example modules** (2 ready-to-use)

### 🚧 Phase 4: Coming Soon
- Remote repositories
- Push/pull
- Clone

---

## Documentation

| Document | Purpose |
|----------|---------|
| [README.md](README.md) | Project overview |
| [QUICKSTART.md](QUICKSTART.md) | 5-minute guide |
| [TUTORIAL.md](TUTORIAL.md) | Complete tutorial |
| [CHEATSHEET.md](CHEATSHEET.md) | Command reference |
| [FAQ.md](FAQ.md) | Frequently asked questions |
| [USAGE.md](USAGE.md) | Full command documentation |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Technical details |
| [INDEX.md](INDEX.md) | Documentation index |

**Total:** 27 documents, ~16000 lines!

---

## Example: Full Workflow

```bash
# Setup
gvc init
export GVC_AUTHOR="Your Name"

# Work
echo "# My Project" > README.md
gvc add README.md
gvc commit -m "feat: initial commit"

# Branch
gvc branch create feature-x
gvc checkout feature-x

# Changes
echo "More content" >> README.md
gvc add README.md
gvc commit -m "feat: add content"

# Check history
gvc log --oneline

# Switch back
gvc checkout main

# Tag
gvc tag create v1.0
```

---

## Install Example Module

```bash
# Install rust linter module
gvc module install ./examples/rust-linter-module

# Activate in your project
cd your-rust-project
gvc module add rust-linter@1.0.0

# Now clippy runs automatically on commit!
gvc commit -m "test"
```

---

## Need Help?

### Quick Answers
- **"How do I...?"** → [TUTORIAL.md](TUTORIAL.md)
- **"What command for...?"** → [CHEATSHEET.md](CHEATSHEET.md)
- **"Common question?"** → [FAQ.md](FAQ.md)

### Get Support
```bash
gvc --help
gvc <command> --help
```

---

## Project Stats

- **15 Rust files** (~4000 lines of code)
- **20 commands** (fully functional)
- **27 documentation files** (~16000 lines)
- **2 example modules** (ready to use)
- **25+ unit tests** (all passing)
- **0 linter errors** ✅

---

## Comparison with Git

| Feature | Git | GVC |
|---------|-----|-----|
| Hash | SHA-1 | SHA-256 ✅ |
| Language | C | Rust ✅ |
| Modules | Submodules | Custom system ✅ |
| Local use | ✅ | ✅ |
| Remote | ✅ | Phase 4 |

---

## Quick Tips

### Set Author Once
```bash
# Linux/macOS
export GVC_AUTHOR="Your Name"

# Windows
$env:GVC_AUTHOR = "Your Name"
```

### Ignore Files
Create `.gvcignore`:
```
*.log
target/
node_modules/
```

### Aliases
```bash
alias gs='gvc status'
alias ga='gvc add'
alias gc='gvc commit'
alias gl='gvc log --oneline'
```

---

## What Makes GVC Special?

1. **SHA-256** - More secure than SHA-1
2. **Rust** - Memory-safe, modern
3. **Module System** - Unique extensibility
4. **Hook System** - Integrated, powerful
5. **Great Docs** - 16000 lines!

---

## Ready to Dive Deeper?

### Next Steps:

1. ✅ You've completed the quick start!
2. 📖 Read [TUTORIAL.md](TUTORIAL.md) for in-depth guide
3. 🔧 Try example modules in [examples/](examples/)
4. 📚 Explore [INDEX.md](INDEX.md) for all documentation
5. 🚀 Start using GVC in your projects!

---

## Support

- **Documentation:** See [INDEX.md](INDEX.md)
- **Examples:** See [examples/](examples/)
- **FAQ:** See [FAQ.md](FAQ.md)
- **Status:** See [PROJECT_STATUS.md](PROJECT_STATUS.md)

---

**🎉 Welcome to GVC! Happy version controlling!** 🚀

*Start reading: [README.md](README.md) → [QUICKSTART.md](QUICKSTART.md) → [TUTORIAL.md](TUTORIAL.md)*

