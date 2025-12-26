# GulfsVersionControl - Architecture Documentation

## Overview

GVC is a Git-inspired version control system built from scratch in Rust. This document describes the internal architecture, design decisions, and implementation details.

## Project Structure

```
GulfsControlSystem/
├── gvc-core/          # Core VCS logic
│   ├── src/
│   │   ├── lib.rs           # Public API
│   │   ├── error.rs         # Error types
│   │   ├── hash.rs          # SHA-256 hashing
│   │   ├── object.rs        # Object model (Blob, Tree, Commit)
│   │   ├── storage.rs       # Content-addressable storage
│   │   ├── index.rs         # Staging area
│   │   ├── refs.rs          # References (branches, tags)
│   │   └── repository.rs    # High-level repository operations
│   └── Cargo.toml
├── gvc-cli/           # Command-line interface
│   ├── src/
│   │   ├── main.rs          # CLI argument parsing
│   │   └── commands/        # Command implementations
│   └── Cargo.toml
├── gvc-server/        # Remote repository server (Phase 4)
│   └── Cargo.toml
└── Cargo.toml         # Workspace configuration
```

## Core Concepts

### 1. Content-Addressable Storage

All objects are stored based on their SHA-256 hash:

```
.gvc/objects/ab/cdef1234567890...
              ^^  ^^^^^^^^^^^^^^
              |   rest of hash
              first 2 hex chars (for filesystem performance)
```

**Benefits:**
- Deduplication: identical content stored once
- Integrity: hash verifies content hasn't changed
- Immutability: objects never modified, only created

### 2. Object Model

#### Blob
Stores raw file content.

```rust
struct Blob {
    data: Vec<u8>
}
```

#### Tree
Represents directory structure, contains entries for files and subdirectories.

```rust
struct Tree {
    entries: BTreeMap<String, TreeEntry>
}

struct TreeEntry {
    mode: u32,           // File permissions
    name: String,        // File/directory name
    hash: Hash,          // Hash of blob or tree
    obj_type: ObjectType // Blob or Tree
}
```

**Why BTreeMap?**
- Deterministic ordering (sorted by name)
- Ensures same directory structure always produces same hash

#### Commit
Snapshot of repository state with metadata.

```rust
struct Commit {
    tree: Hash,           // Root tree
    parents: Vec<Hash>,   // Parent commit(s) - supports merges
    author: String,
    committer: String,
    timestamp: i64,
    message: String
}
```

### 3. Index (Staging Area)

The index tracks files staged for the next commit:

```rust
struct Index {
    entries: BTreeMap<PathBuf, IndexEntry>
}

struct IndexEntry {
    path: PathBuf,
    hash: Hash,
    size: u64,
    mtime: i64  // Modification time for change detection
}
```

**Stored at:** `.gvc/index` (binary serialized with bincode)

### 4. References

References provide human-readable names for commits:

```
.gvc/
├── HEAD                    # Current branch or commit
├── refs/
│   ├── heads/
│   │   ├── main           # Branch: contains commit hash
│   │   └── feature-x
│   └── tags/
│       └── v1.0           # Tag: contains commit hash
```

**HEAD can be:**
- Symbolic: `ref: refs/heads/main` (normal branch)
- Direct: `abc123...` (detached HEAD)

## Key Operations

### Init

1. Create `.gvc/` directory structure
2. Initialize object storage (create 00-ff subdirectories)
3. Create default branch (`refs/heads/main`)
4. Set HEAD to `ref: refs/heads/main`
5. Create empty index

### Add

1. Read file content
2. Create Blob object
3. Compute hash and store in `.gvc/objects/`
4. Add entry to index with path, hash, size, mtime
5. Save index

**Note:** At this stage, tree structure is not yet built (happens at commit).

### Commit

1. Load index (staged files)
2. Build tree structure from index:
   - Group files by directory
   - Create Tree objects bottom-up
   - Store each tree, get hash
   - Root tree represents entire repository state
3. Get parent commit(s) from HEAD
4. Create Commit object with tree hash, parents, metadata
5. Store commit object, get hash
6. Update current branch to point to new commit
7. Index remains unchanged (allows incremental staging)

**Tree Building Algorithm:**

```
files:
  src/main.rs
  src/lib.rs
  README.md

→ trees:
  Tree(src) → [main.rs, lib.rs]
  Tree(root) → [src/, README.md]
```

### Log

1. Get HEAD commit hash
2. Load commit object
3. Display commit info
4. Follow parent link
5. Repeat until no more parents

**Linear history for now** - merge support in Phase 2+.

### Checkout

1. Verify branch exists
2. Update HEAD to point to branch
3. (Phase 2) Update working directory files

**Current limitation:** Working directory not yet updated.

## Design Decisions

### Why Rust?

- **Memory safety:** No segfaults, no data races
- **Performance:** Zero-cost abstractions, comparable to C
- **Cross-platform:** Windows and Linux support out of the box
- **Modern tooling:** Cargo, rustfmt, clippy
- **Strong type system:** Catch errors at compile time

### Why bincode for serialization?

- Fast binary format
- Smaller than JSON
- Type-safe with serde
- Good enough for MVP

**Alternative considered:** Custom format (more control, but more work)

### Why BTreeMap instead of HashMap?

- **Deterministic ordering:** Same input always produces same hash
- **Sorted keys:** Better for diffs and debugging
- Slightly slower, but correctness > performance for MVP

### Why SHA-256 instead of SHA-1?

- **Security:** SHA-1 is broken (collision attacks)
- **Future-proof:** Industry standard
- **Overkill?** Yes, but better safe than sorry

### Object Storage Layout

```
objects/ab/cdef123...
```

**Why split into subdirectories?**
- Filesystem performance degrades with too many files in one directory
- Git uses same approach (first 2 hex chars)
- 256 subdirectories = good balance

## Error Handling

Uses `thiserror` for custom error types:

```rust
pub enum Error {
    Io(std::io::Error),
    RepositoryNotFound(PathBuf),
    ObjectNotFound(String),
    // ...
}
```

**Strategy:**
- Core library returns `Result<T, Error>`
- CLI converts to `anyhow::Result<T>` for better error messages
- No panics in library code (except for bugs)

## Testing Strategy

### Unit Tests
- Each module has `#[cfg(test)]` section
- Test core functionality in isolation
- Use `tempfile` crate for temporary directories

### Integration Tests
- Test CLI commands end-to-end
- Verify repository state after operations

**Current coverage:** Basic tests for core modules

**TODO:** Expand test coverage in Phase 5

## Performance Considerations

### Current Approach (MVP)
- Full snapshots (no deltas)
- No compression
- Simple linear history
- Load entire objects into memory

**This is intentional** - optimize later.

### Future Optimizations (Phase 5)
- **Packfiles:** Combine objects, use deltas
- **Compression:** zstd for objects
- **Streaming:** Large files don't fit in memory
- **Index caching:** Avoid repeated filesystem scans
- **Garbage collection:** Remove unreachable objects

## Limitations (Current)

1. **No merge support** - only linear history
2. **No working directory update** - checkout only updates HEAD
3. **No diff implementation** - coming in Phase 2
4. **No remote support** - coming in Phase 4
5. **No conflict resolution** - coming with merge support
6. **No .gvcignore** - tracks everything
7. **No submodules** - module system is different (Phase 3)

## Module System (Phase 3)

Planned architecture:

```
.gvc/modules/
├── installed/         # Global module cache
│   └── module-name@version/
└── active/            # Project-specific modules
    └── module-name -> ../../installed/module-name@version/
```

**Modules can provide:**
- Templates (file scaffolding)
- Hooks (pre-commit, post-commit, etc.)
- Configuration extensions
- CLI extensions (optional)

**Different from Git submodules:**
- Not nested repositories
- More like package manager (npm, cargo)
- Versioned and installable

## Server Architecture (Phase 4)

Planned design:

```
gvc-server (HTTP REST API)
├── Repository hosting
├── Authentication (token-based)
├── Push: receive objects + refs
├── Pull: send objects + refs
└── Object transfer optimization
```

**Protocol:**
1. Client sends list of local refs
2. Server computes missing objects
3. Transfer only missing objects
4. Update refs atomically

## Security Considerations

### Current
- Hash verification on object load
- Atomic file writes (write to temp, then rename)
- No network code yet (no attack surface)

### Future
- **Server authentication:** Token-based, HTTPS only
- **Object validation:** Verify object types, prevent malicious objects
- **Rate limiting:** Prevent DoS
- **Access control:** Per-repository permissions

## Comparison with Git

| Feature | Git | GVC |
|---------|-----|-----|
| Hash | SHA-1 (migrating to SHA-256) | SHA-256 |
| Storage | Packfiles + loose objects | Loose objects (MVP) |
| Serialization | Custom format | bincode |
| Language | C | Rust |
| Module System | Submodules | Custom (Phase 3) |
| Server | git-daemon, HTTP | HTTP REST (Phase 4) |

**GVC is NOT a Git replacement** - it's a learning project and potential innovation platform.

## Development Phases

### ✅ Phase 1 (MVP) - COMPLETE
- Core objects (Blob, Tree, Commit)
- Repository init
- Add, commit, log
- Basic CLI

### Phase 2 (Usability)
- Status (show changes)
- Diff (compare versions)
- Branches (full implementation)
- Checkout (update working directory)

### Phase 3 (Modules)
- Module format definition
- Global installation
- Project activation
- Hook system

### Phase 4 (Server)
- Remote repository hosting
- Push/pull
- Object transfer
- Authentication

### Phase 5 (Stabilization)
- Comprehensive testing
- Error handling improvements
- Performance optimization
- Documentation

## Contributing

When adding features:

1. **Design first:** Update this document
2. **Test:** Add unit tests
3. **Document:** Update README and inline comments
4. **Iterate:** Start simple, optimize later

## References

- [Git Internals](https://git-scm.com/book/en/v2/Git-Internals-Plumbing-and-Porcelain)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Content-Addressable Storage](https://en.wikipedia.org/wiki/Content-addressable_storage)

