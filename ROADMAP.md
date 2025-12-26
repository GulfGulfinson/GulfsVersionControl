# GulfsVersionControl - Development Roadmap

## ✅ Phase 1: Core (MVP) - COMPLETED

**Goal:** Basic version control functionality

### Completed Features
- ✅ Repository initialization (`gvc init`)
- ✅ Content-addressable storage (SHA-256)
- ✅ Object model (Blob, Tree, Commit)
- ✅ Staging area (Index)
- ✅ Add files (`gvc add`)
- ✅ Create commits (`gvc commit`)
- ✅ View history (`gvc log`)
- ✅ Basic branching (`gvc branch create/delete/list`)
- ✅ Checkout branches (`gvc checkout`)
- ✅ Tags (`gvc tag create/list`)
- ✅ Show objects (`gvc show`)
- ✅ CLI interface with clap

### Architecture
- Workspace structure with 3 crates (core, cli, server)
- Deterministic tree building
- Reference management (HEAD, branches, tags)
- Binary serialization with bincode

---

## ✅ Phase 2: Usability - COMPLETED

**Goal:** Make GVC practical for daily use

### Completed Features

#### Status Improvements
- [ ] Show unstaged changes (modified, deleted)
- [ ] Show untracked files
- [ ] Show staged vs unstaged separately
- [ ] Colorized output

#### Diff Implementation
- [x] `gvc diff` - show unstaged changes
- [x] `gvc diff --staged` - show staged changes
- [x] Line-by-line diff algorithm (Myers' LCS)
- [x] Unified diff format
- [x] Colorized diff output
- [x] Support for new/modified/deleted files
- [ ] `gvc diff <commit>` - compare with commit (Phase 3)
- [ ] `gvc diff <commit1> <commit2>` - compare two commits (Phase 3)

#### Checkout Improvements
- [x] Update working directory files on checkout
- [x] Detect and prevent uncommitted changes loss
- [x] Atomic working directory updates
- [x] Update index on checkout
- [ ] Detached HEAD support (Phase 3)
- [ ] `gvc checkout <commit>` - checkout specific commit (Phase 3)
- [ ] `gvc checkout <file>` - restore file from HEAD (Phase 3)

#### Status Improvements
- [x] Show unstaged changes (modified, deleted)
- [x] Show untracked files
- [x] Show staged vs unstaged separately
- [x] Colorized output
- [x] Helpful command hints

#### Additional Commands
- [x] `gvc reset` - unstage files
- [x] `gvc reset` (no args) - unstage all
- [ ] `gvc restore` - discard changes (Phase 3)
- [ ] `gvc rm` - remove files (Phase 3)
- [ ] `gvc mv` - move/rename files (Phase 3)

#### Configuration
- [ ] User configuration (`~/.gvcconfig`) (Phase 3)
- [ ] Repository configuration (`.gvc/config`) (Phase 3)
- [ ] `gvc config` command (Phase 3)
- [ ] Default author from config (Phase 3)

#### Ignore System
- [x] `.gvcignore` file support
- [x] Glob pattern matching (*, ?, **)
- [x] Negation patterns (!pattern)
- [x] Directory-only patterns (dir/)
- [x] Respect `.gvcignore` in add/status
- [x] Default ignore for .gvc/

---

## ✅ Phase 3: Module System - COMPLETED

**Goal:** Extensibility through modules

### Completed Features

#### Module Format
- [x] Define module manifest format (TOML)
- [x] Module metadata (name, version, author, description)
- [x] Semantic versioning validation
- [x] Module types (template, hook, config)
- [ ] Module dependencies resolution (Phase 3.5)

#### Module Storage
```
.gvc/modules/
├── installed/           # Global cache
│   └── module-name@1.0.0/
│       ├── manifest.toml
│       ├── files/
│       ├── hooks/
│       └── templates/
└── active/              # Project-specific
    └── module-name -> ../installed/module-name@1.0.0/
```

#### Module Commands
- [x] `gvc module create <name>` - scaffold new module
- [x] `gvc module install <path>` - install from path
- [x] `gvc module add <name>` - activate in project
- [x] `gvc module remove <name>` - deactivate in project
- [x] `gvc module list` - list installed/active modules
- [x] `gvc module info <name>` - show module details
- [ ] `gvc module update <name>` - update module (Phase 3.5)
- [ ] `gvc module publish` - publish to registry (Phase 3.5)
- [ ] `gvc module uninstall` - remove from global (Phase 3.5)

#### Module Capabilities

**Templates:**
- [ ] File/directory scaffolding
- [ ] Variable substitution
- [ ] Conditional generation

**Hooks:**
- [x] `pre-commit` - validate before commit
- [x] `post-commit` - actions after commit
- [x] `pre-checkout` - validate before checkout
- [x] `post-checkout` - actions after checkout
- [x] `pre-push` - validate before push (integrated, Phase 4 needed)
- [x] `post-pull` - actions after pull (integrated, Phase 4 needed)
- [x] Hook execution environment (env vars)
- [x] Multi-hook support (repo + modules)
- [x] Cross-platform (sh, ps1, exe)

**Configuration:**
- [ ] Extend repository config
- [ ] Module-specific settings
- [ ] Config validation

**CLI Extensions (Optional):**
- [ ] Custom subcommands
- [ ] Plugin system for CLI

#### Module Registry (Optional)
- [ ] Central module registry
- [ ] Search modules
- [ ] Version resolution
- [ ] Dependency management

---

## 🌍 Phase 4: Remote & Server

**Goal:** Distributed version control

### Server Implementation

#### Basic Server
- [ ] HTTP REST API (axum)
- [ ] Repository hosting
- [ ] Multi-repository support
- [ ] Repository creation/deletion

#### Authentication
- [ ] Token-based authentication
- [ ] User management
- [ ] Per-repository permissions
- [ ] HTTPS/TLS support

#### Object Transfer Protocol
- [ ] List refs endpoint
- [ ] Compute missing objects
- [ ] Batch object transfer
- [ ] Compression (zstd)
- [ ] Progress reporting

### Client Implementation

#### Remote Management
- [ ] `gvc remote add <name> <url>`
- [ ] `gvc remote remove <name>`
- [ ] `gvc remote list`
- [ ] `gvc remote rename <old> <new>`
- [ ] Store remotes in config

#### Push
- [ ] `gvc push <remote> <branch>`
- [ ] Upload missing objects
- [ ] Update remote refs
- [ ] Force push (with safety)
- [ ] Push tags

#### Pull
- [ ] `gvc pull <remote> <branch>`
- [ ] Fetch remote objects
- [ ] Update local refs
- [ ] Merge changes (basic)

#### Fetch
- [ ] `gvc fetch <remote>`
- [ ] Download objects without merging
- [ ] Update remote-tracking branches

#### Clone
- [ ] `gvc clone <url> [<directory>]`
- [ ] Initialize local repository
- [ ] Fetch all objects
- [ ] Set up remote
- [ ] Checkout default branch

### Server Deployment
- [ ] Docker image
- [ ] Configuration file
- [ ] Logging
- [ ] Monitoring endpoints
- [ ] Backup/restore

---

## 🔧 Phase 5: Stabilization & Performance

**Goal:** Production-ready quality

### Testing
- [ ] Comprehensive unit tests (>80% coverage)
- [ ] Integration tests for all commands
- [ ] End-to-end tests
- [ ] Performance benchmarks
- [ ] Stress tests (large repos, many files)
- [ ] Cross-platform testing (Windows, Linux, macOS)

### Error Handling
- [ ] Better error messages
- [ ] Error recovery strategies
- [ ] Validation at all entry points
- [ ] Graceful degradation

### Performance Optimizations

#### Object Storage
- [ ] Packfiles (combine objects)
- [ ] Delta compression
- [ ] Object caching
- [ ] Lazy loading

#### Index
- [ ] Binary search for entries
- [ ] Incremental updates
- [ ] Index v2 format (with extensions)

#### Filesystem
- [ ] Parallel file operations
- [ ] Streaming large files
- [ ] Memory-mapped files

#### Garbage Collection
- [ ] `gvc gc` - remove unreachable objects
- [ ] Repack objects
- [ ] Prune old objects
- [ ] Optimize repository size

### Documentation
- [ ] API documentation (rustdoc)
- [ ] User manual
- [ ] Tutorial videos
- [ ] Architecture deep-dive
- [ ] Contributing guide
- [ ] Code of conduct

### Developer Experience
- [ ] Better debug output (`--verbose` flag)
- [ ] Profiling support
- [ ] Benchmarking suite
- [ ] CI/CD pipeline
- [ ] Automated releases

---

## 🚀 Phase 6: Advanced Features (Future)

**Goal:** Innovation beyond Git

### Merge & Conflict Resolution
- [ ] Three-way merge
- [ ] Conflict detection
- [ ] Conflict markers
- [ ] Merge strategies (recursive, ours, theirs)
- [ ] Interactive conflict resolution
- [ ] `gvc merge <branch>`
- [ ] `gvc rebase <branch>`

### History Manipulation
- [ ] `gvc rebase` - rewrite history
- [ ] `gvc cherry-pick` - apply specific commits
- [ ] `gvc revert` - undo commits
- [ ] Interactive rebase

### Advanced Branching
- [ ] Merge commits (multiple parents)
- [ ] Commit graph visualization
- [ ] Branch comparison
- [ ] Fast-forward detection

### Stash
- [ ] `gvc stash` - save work in progress
- [ ] `gvc stash pop` - restore stashed work
- [ ] Multiple stashes
- [ ] Stash with message

### Worktrees
- [ ] Multiple working directories
- [ ] `gvc worktree add`
- [ ] `gvc worktree list`

### Submodules (Different from Modules)
- [ ] Nested repositories
- [ ] `gvc submodule add`
- [ ] `gvc submodule update`

### Hooks Enhancement
- [ ] GUI hooks (desktop notifications)
- [ ] Async hooks
- [ ] Hook marketplace

### AI Integration (Experimental)
- [ ] Smart commit message generation
- [ ] Code review suggestions
- [ ] Conflict resolution assistance
- [ ] Pattern detection

### Blockchain Integration (Experimental)
- [ ] Commit verification on blockchain
- [ ] Decentralized repository hosting
- [ ] Immutable audit trail

---

## Timeline (Estimated)

| Phase | Duration | Status |
|-------|----------|--------|
| Phase 1 | 2-3 weeks | ✅ Complete |
| Phase 2 | 3-4 weeks | 🚧 Next |
| Phase 3 | 4-6 weeks | 📋 Planned |
| Phase 4 | 4-6 weeks | 📋 Planned |
| Phase 5 | 4-8 weeks | 📋 Planned |
| Phase 6 | Ongoing | 💡 Future |

---

## Contributing

Want to contribute? Pick a feature from Phase 2 or 3 and:

1. Open an issue to discuss
2. Create a branch
3. Implement with tests
4. Submit pull request

See [ARCHITECTURE.md](ARCHITECTURE.md) for technical details.

---

## Feedback

Have ideas for features? Open an issue with:
- Feature description
- Use case
- Why it's important
- Proposed implementation (optional)

