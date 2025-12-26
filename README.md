# GulfsVersionControl (GVC)

<div align="center">

**A modern, Git-inspired version control system written in Rust from scratch**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Phase](https://img.shields.io/badge/Phase-3%20Complete-success.svg)](ROADMAP.md)
[![Status](https://img.shields.io/badge/Status-Production--Ready-brightgreen.svg)](PROJECT_COMPLETE.md)

**🌟 [New here? Start Here!](WELCOME.md) | [5-Min Quick Start](QUICKSTART.md) | [Complete Tutorial](TUTORIAL.md) 🌟**

[Features](#features) • [Installation](#installation) • [Quick Start](#quick-start) • [Documentation](#documentation) • [Roadmap](#roadmap)

</div>

---

## 🎯 Overview

GVC is a fully functional version control system built to understand and innovate on Git's core concepts. Written entirely in Rust, it provides:

- **Content-addressable storage** with SHA-256 hashing
- **Complete object model** (Blobs, Trees, Commits)
- **Branching and tagging** support
- **Staging area** for incremental commits
- **Cross-platform** support (Windows, Linux, macOS)
- **Future-ready** architecture for modules and distributed workflows

## ✨ Features

### ✅ Phase 1 (MVP) - Complete

| Feature | Command | Status |
|---------|---------|--------|
| Repository initialization | `gvc init` | ✅ |
| Stage files | `gvc add <file>` | ✅ |
| Create commits | `gvc commit -m "msg"` | ✅ |
| View history | `gvc log` | ✅ |
| Check status | `gvc status` | ✅ |
| Create branches | `gvc branch create <name>` | ✅ |
| Switch branches | `gvc checkout <branch>` | ✅ |
| Create tags | `gvc tag create <name>` | ✅ |
| Inspect objects | `gvc show <hash>` | ✅ |

### ✅ Phase 2 (Usability) - Complete

| Feature | Command | Status |
|---------|---------|--------|
| Show differences | `gvc diff` / `gvc diff --staged` | ✅ |
| Improved status | `gvc status` (colorized, detailed) | ✅ |
| Ignore files | `.gvcignore` support | ✅ |
| Unstage files | `gvc reset [<file>]` | ✅ |
| Working dir update | `gvc checkout` (updates files) | ✅ |

### ✅ Phase 3 (Modules) - Complete

| Feature | Command | Status |
|---------|---------|--------|
| Create modules | `gvc module create <name>` | ✅ |
| Install modules | `gvc module install <path>` | ✅ |
| Activate modules | `gvc module add <id>` | ✅ |
| List modules | `gvc module list` | ✅ |
| Hook system | pre-commit, post-commit, etc. | ✅ |
| TOML manifests | `module.toml` | ✅ |

### 🚧 Coming Soon

- **Phase 4**: Remote server, push/pull, clone
- **Phase 5**: Performance optimization, packfiles, GC

See [ROADMAP.md](ROADMAP.md) for details.

## 📦 Installation

### Prerequisites

Install Rust (if not already installed):

**Windows (PowerShell):**
```powershell
Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile "$env:TEMP\rustup-init.exe"
& "$env:TEMP\rustup-init.exe" -y
```

**Linux/macOS:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Build GVC

```bash
# Clone or navigate to the project
cd GulfsControlSystem

# Build release version
cargo build --release

# Install globally
cargo install --path gvc-cli
```

The `gvc` binary will be available in `~/.cargo/bin/` (should be in your PATH).

**Verify installation:**
```bash
gvc --version
gvc --help
```

See [INSTALLATION.md](INSTALLATION.md) for detailed instructions.

## 🚀 Quick Start

```bash
# Create a new project
mkdir my-project
cd my-project

# Initialize GVC repository
gvc init

# Create some files
echo "# My Project" > README.md
echo "fn main() {}" > main.rs

# Stage files
gvc add .

# Create first commit
gvc commit -m "Initial commit"

# View history
gvc log

# Create a feature branch
gvc branch create feature-auth
gvc checkout feature-auth

# Make changes and commit
echo "fn authenticate() {}" >> auth.rs
gvc add auth.rs
gvc commit -m "Add authentication"

# Switch back to main
gvc checkout main

# Create a tag
gvc tag create v1.0
```

See [USAGE.md](USAGE.md) for comprehensive usage guide.

## 📚 Documentation

| Document | Description |
|----------|-------------|
| [USAGE.md](USAGE.md) | Complete user guide with examples |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Technical architecture and design decisions |
| [INSTALLATION.md](INSTALLATION.md) | Detailed installation instructions |
| [BUILD_INSTRUCTIONS.md](BUILD_INSTRUCTIONS.md) | Build guide (German) |
| [ROADMAP.md](ROADMAP.md) | Development roadmap (Phase 1-6) |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Contribution guidelines |
| [PROJECT_SUMMARY.md](PROJECT_SUMMARY.md) | Project summary (German) |

## 🏗️ Project Structure

```
GulfsControlSystem/
├── gvc-core/          # Core VCS logic
│   ├── error.rs       # Error types
│   ├── hash.rs        # SHA-256 hashing
│   ├── object.rs      # Blob, Tree, Commit
│   ├── storage.rs     # Content-addressable storage
│   ├── index.rs       # Staging area
│   ├── refs.rs        # Branches and tags
│   └── repository.rs  # High-level operations
├── gvc-cli/           # Command-line interface
│   ├── main.rs        # Argument parsing (clap)
│   └── commands/      # Command implementations
└── gvc-server/        # Server (Phase 4)
```

## 🔧 Architecture Highlights

### Content-Addressable Storage

All objects are stored by their SHA-256 hash:

```
.gvc/objects/ab/cdef123456...
              ^^  ^^^^^^^^^^^
              |   rest of hash
              first 2 hex chars
```

**Benefits:**
- Automatic deduplication
- Integrity verification
- Immutable objects

### Object Model

```rust
Blob    → File content
Tree    → Directory structure (sorted entries)
Commit  → Snapshot + metadata + parent(s)
```

### Deterministic Trees

Uses `BTreeMap` for sorted entries → same content always produces same hash.

See [ARCHITECTURE.md](ARCHITECTURE.md) for deep dive.

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run test script (Windows)
.\test-gvc.ps1

# Run test script (Linux/macOS)
chmod +x test-gvc.sh
./test-gvc.sh
```

## 🗺️ Roadmap

| Phase | Status | Features |
|-------|--------|----------|
| **Phase 1** | ✅ Complete | Core objects, init, add, commit, log, branches, tags |
| **Phase 2** | 📋 Planned | Diff, improved status, checkout with working dir update |
| **Phase 3** | 📋 Planned | Module system, hooks, templates |
| **Phase 4** | 📋 Planned | Remote server, push, pull, clone |
| **Phase 5** | 📋 Planned | Optimization, packfiles, GC, comprehensive tests |
| **Phase 6** | 💡 Future | Merge, rebase, advanced features |

See [ROADMAP.md](ROADMAP.md) for detailed timeline.

## 🤝 Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

**Good first issues:**
- Improve error messages
- Add more tests
- Documentation improvements
- CLI usability enhancements

## 📊 Comparison with Git

| Feature | Git | GVC (Phase 1) |
|---------|-----|---------------|
| Hash Algorithm | SHA-1 → SHA-256 | SHA-256 |
| Language | C | Rust |
| Storage | Packfiles + Loose | Loose (MVP) |
| Branches | ✅ | ✅ |
| Tags | ✅ | ✅ |
| Merge | ✅ | ❌ (Phase 2+) |
| Remote | ✅ | ❌ (Phase 4) |
| Diff | ✅ | ❌ (Phase 2) |
| Modules | Submodules | Custom (Phase 3) |

**GVC is NOT a Git replacement** - it's a learning project and innovation platform.

## 🎓 Learning Resources

- [Git Internals](https://git-scm.com/book/en/v2/Git-Internals-Plumbing-and-Porcelain)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Content-Addressable Storage](https://en.wikipedia.org/wiki/Content-addressable_storage)

## 📝 License

MIT License - see [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

Inspired by Git's elegant design and Rust's safety guarantees.

---

<div align="center">

**Built with ❤️ in Rust**

[Report Bug](https://github.com/yourusername/GulfsControlSystem/issues) • [Request Feature](https://github.com/yourusername/GulfsControlSystem/issues)

</div>

