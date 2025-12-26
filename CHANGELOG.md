# Changelog

All notable changes to GulfsVersionControl will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Planned for Phase 4
- Remote repository support
- Push/pull functionality
- Clone command
- Server implementation
- Authentication system

---

## [0.3.0] - 2025-12-26

### Phase 3: Module System - Complete ✅

### Added
- **Module System**
  - `gvc module create` - Create module scaffold
  - `gvc module install` - Install modules globally
  - `gvc module add` - Activate modules in repository
  - `gvc module remove` - Deactivate modules
  - `gvc module list` - List installed/active modules
  - `gvc module info` - Show module details
  - TOML-based module manifests
  - Semantic versioning validation
  - Global module storage (`~/.gvc/modules`)
  - Symlink-based activation

- **Hook System**
  - 6 hook types: pre-commit, post-commit, pre-checkout, post-checkout, pre-push, post-pull
  - Repository hooks (`.gvc/hooks/`)
  - Module hooks
  - Multi-hook support
  - Environment variable passing
  - Cross-platform execution (sh, ps1, exe)
  - Sample hook generation
  - Hook execution integrated into commit and checkout

- **Example Modules**
  - `rust-linter-module` - Runs cargo clippy before commits
  - `commit-convention-module` - Enforces conventional commit messages

- **Documentation**
  - PHASE3_COMPLETE.md - Complete Phase 3 documentation
  - COMPLETE_SUMMARY.md - All phases summary
  - TUTORIAL.md - Complete tutorial from basics to advanced
  - CHEATSHEET.md - Quick command reference
  - FAQ.md - Frequently asked questions
  - examples/README.md - Example modules guide

### Changed
- Commit operations now execute pre/post-commit hooks
- Checkout operations now execute pre/post-checkout hooks
- Module commands added to CLI
- README.md badge updated to Phase 3

### Dependencies
- Added `toml` 0.8 - TOML parsing
- Added `dirs` 5.0 - Home directory detection

---

## [0.2.0] - 2025-12-26

### Phase 2: Usability Features - Complete ✅

### Added
- **Diff Engine**
  - `gvc diff` - Show unstaged changes
  - `gvc diff --staged` - Show staged changes
  - Myers' LCS algorithm implementation
  - Unified diff format
  - Hunk grouping with context lines
  - Colorized output (green/red/cyan)
  - Binary file detection

- **Improved Status**
  - Shows staged changes (new/modified)
  - Shows unstaged changes (modified/deleted)
  - Shows untracked files
  - Colorized output
  - Helpful command hints

- **Ignore System**
  - `.gvcignore` file support
  - Glob pattern matching (*, ?, **)
  - Negation patterns (!pattern)
  - Directory-only patterns (dir/)
  - Always ignores `.gvc/` directory
  - `.gvcignore.example` provided

- **Reset Command**
  - `gvc reset` - Unstage all files
  - `gvc reset <file>...` - Unstage specific files

- **Enhanced Checkout**
  - Updates working directory files
  - Detects uncommitted changes
  - Prevents data loss
  - Atomic operation

- **Documentation**
  - PHASE2_COMPLETE.md - Complete Phase 2 documentation
  - PHASE2_SUMMARY.md - Quick reference (German)
  - Test scripts updated with Phase 2 features

### Changed
- `gvc status` now shows detailed status (staged/unstaged/untracked)
- `gvc checkout` now updates working directory files
- `gvc diff` now functional (was placeholder in Phase 1)
- Test scripts now test diff, ignore, and reset features

---

## [0.1.0] - 2025-12-26

### Phase 1: Core VCS (MVP) - Complete ✅

### Added
- **Core VCS Functionality**
  - `gvc init` - Initialize repository
  - `gvc add` - Stage files
  - `gvc commit` - Create commits
  - `gvc log` - View commit history
  - `gvc status` - Basic status
  - `gvc show` - Inspect objects

- **Object Model**
  - Blob - File content storage
  - Tree - Directory structure (BTreeMap for determinism)
  - Commit - Snapshots with metadata
  - SHA-256 hashing (64 hex characters)
  - Binary serialization (bincode)

- **Storage**
  - Content-addressable storage
  - Objects stored in `.gvc/objects/`
  - Subdirectories for performance (00-ff)
  - Automatic deduplication
  - Integrity verification

- **Staging Area (Index)**
  - File staging with metadata
  - Binary serialization
  - Persistent across commands

- **Reference Management**
  - Branches (create, delete, list)
  - Tags (create, list)
  - HEAD management (symbolic and direct)
  - Reference resolution

- **Branching**
  - `gvc branch create` - Create branch
  - `gvc branch delete` - Delete branch
  - `gvc branch list` - List branches
  - `gvc checkout` - Switch branches (HEAD only)
  - `gvc switch` - Alias for checkout

- **Tags**
  - `gvc tag create` - Create tag
  - `gvc tag list` - List tags

- **CLI**
  - Command-line interface with clap
  - Argument parsing
  - Subcommands
  - Help system

- **Documentation**
  - README.md - Project overview
  - QUICKSTART.md - 5-minute guide
  - USAGE.md - Command reference
  - ARCHITECTURE.md - Technical details
  - INSTALLATION.md - Setup instructions
  - BUILD_INSTRUCTIONS.md - Build guide (German)
  - CONTRIBUTING.md - Contribution guidelines
  - ROADMAP.md - Development plan
  - VISUAL_GUIDE.md - Visual diagrams
  - PROJECT_SUMMARY.md - Summary (German)
  - FINAL_NOTES.md - Phase 1 notes
  - INDEX.md - Documentation index
  - LICENSE - MIT License

- **Testing**
  - Unit tests for all core modules
  - `tempfile` for filesystem tests
  - Test scripts (PowerShell and Bash)

### Dependencies
- `sha2` 0.10 - SHA-256 hashing
- `hex` 0.4 - Hex encoding
- `serde` 1.0 - Serialization framework
- `bincode` 1.3 - Binary serialization
- `thiserror` 1.0 - Error handling
- `anyhow` 1.0 - Error context
- `chrono` 0.4 - Timestamps
- `walkdir` 2.4 - Directory traversal
- `clap` 4.5 - CLI parsing

---

## Project Structure

```
GulfsControlSystem/
├── gvc-core/          # Core VCS library
├── gvc-cli/           # Command-line interface
├── gvc-server/        # Server (Phase 4)
├── examples/          # Example modules (Phase 3)
└── Documentation/     # 23 markdown files
```

---

## Version History Summary

| Version | Phase | Date | Features |
|---------|-------|------|----------|
| 0.3.0 | Phase 3 | 2025-12-26 | Module system, hooks |
| 0.2.0 | Phase 2 | 2025-12-26 | Diff, ignore, improved status |
| 0.1.0 | Phase 1 | 2025-12-26 | Core VCS, branching, tags |

---

## Links

- [Repository](https://github.com/yourusername/GulfsControlSystem)
- [Documentation](INDEX.md)
- [Roadmap](ROADMAP.md)
- [Contributing](CONTRIBUTING.md)

---

[Unreleased]: https://github.com/yourusername/GulfsControlSystem/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/yourusername/GulfsControlSystem/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/yourusername/GulfsControlSystem/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/yourusername/GulfsControlSystem/releases/tag/v0.1.0

