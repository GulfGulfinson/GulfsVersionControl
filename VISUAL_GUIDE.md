# GVC Visual Guide

Visual representation of GVC's architecture and workflows.

## Repository Structure

```
my-project/
│
├── .gvc/                          # GVC repository data
│   │
│   ├── objects/                   # Content-addressable storage
│   │   ├── 00/
│   │   │   └── 1a2b3c...         # Object file (hash: 001a2b3c...)
│   │   ├── 01/
│   │   ├── ...
│   │   └── ff/
│   │
│   ├── refs/                      # References (branches, tags)
│   │   ├── heads/
│   │   │   ├── main              # Branch: points to commit hash
│   │   │   └── feature-x
│   │   └── tags/
│   │       └── v1.0              # Tag: points to commit hash
│   │
│   ├── HEAD                       # Current branch or commit
│   ├── index                      # Staging area (binary)
│   ├── config                     # Repository configuration
│   ├── modules/                   # Modules (Phase 3)
│   └── hooks/                     # Hooks (Phase 3)
│
├── README.md                      # Your project files
├── main.rs
└── src/
    └── lib.rs
```

## Object Model

```
┌─────────────────────────────────────────────────────────┐
│                     COMMIT OBJECT                       │
│  Hash: a1b2c3d4e5f6...                                 │
│                                                         │
│  tree:      e4f5a6b7c8d9...  ──────┐                   │
│  parent:    7c8d9e0a1b2c...        │                   │
│  author:    John Doe               │                   │
│  timestamp: 1735210800             │                   │
│  message:   "Initial commit"       │                   │
└─────────────────────────────────────┘                   │
                                      │                   │
                                      ▼                   │
                            ┌──────────────────┐          │
                            │   TREE OBJECT    │          │
                            │  Hash: e4f5a6b.. │          │
                            │                  │          │
                            │  README.md  ──┐  │          │
                            │  main.rs    ──┼──┼──┐       │
                            │  src/       ──┼──┘  │       │
                            └───────────────┘     │       │
                                      │           │       │
                    ┌─────────────────┼───────────┘       │
                    │                 │                   │
                    ▼                 ▼                   │
          ┌──────────────┐  ┌──────────────┐             │
          │ BLOB OBJECT  │  │ BLOB OBJECT  │             │
          │ Hash: 1a2b.. │  │ Hash: 3c4d.. │             │
          │              │  │              │             │
          │ "# My Proj"  │  │ "fn main()"  │             │
          └──────────────┘  └──────────────┘             │
```

## Commit History (Linear)

```
main
  ↓
  ●  a1b2c3d "Add feature X"
  │
  ●  e4f5a6b "Fix bug"
  │
  ●  7c8d9e0 "Initial commit"
```

## Branching

```
         main                feature-x
           ↓                     ↓
           ●  a1b2c3d            ●  9f8e7d6 "Add feature X"
           │                     │
           ●  e4f5a6b ───────────┘
           │
           ●  7c8d9e0 "Initial commit"
```

## Workflow: Add & Commit

```
┌─────────────────┐
│ Working Directory│
│                 │
│  README.md      │  ← Edit files
│  main.rs        │
└─────────────────┘
        │
        │ gvc add .
        ▼
┌─────────────────┐
│  Staging Area   │
│    (Index)      │
│                 │
│  README.md  ●   │  ← Files staged
│  main.rs    ●   │
└─────────────────┘
        │
        │ gvc commit -m "message"
        ▼
┌─────────────────┐
│   Repository    │
│   (.gvc/objects)│
│                 │
│  Commit  ●      │  ← New commit created
│  Tree    ●      │
│  Blobs   ●●     │
└─────────────────┘
```

## Object Storage: Content-Addressable

```
File: "Hello, GVC!"
  │
  │ SHA-256 Hash
  ▼
Hash: a1b2c3d4e5f6789012345678901234567890123456789012345678901234

Storage Path:
.gvc/objects/a1/b2c3d4e5f6789012345678901234567890123456789012345678901234
             ^^  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
             │   Rest of hash (62 chars)
             First 2 chars (directory)

Benefits:
✓ Same content → Same hash → Stored once (deduplication)
✓ Hash verifies integrity
✓ Immutable objects
```

## Tree Building (from Index)

```
Index (Staging Area):
  src/main.rs   → blob:1a2b3c
  src/lib.rs    → blob:4d5e6f
  README.md     → blob:7g8h9i

Step 1: Build subtree for src/
  Tree(src) → [main.rs:1a2b3c, lib.rs:4d5e6f]
  Store → hash:abc123

Step 2: Build root tree
  Tree(root) → [src/:abc123, README.md:7g8h9i]
  Store → hash:def456

Step 3: Create commit
  Commit → tree:def456, parent:..., message:...
  Store → hash:ghi789

Step 4: Update branch
  refs/heads/main → ghi789
```

## HEAD Management

### Normal (Symbolic)

```
HEAD
  │
  │ ref: refs/heads/main
  ▼
refs/heads/main
  │
  │ a1b2c3d4e5f6...
  ▼
Commit a1b2c3d
```

### Detached HEAD

```
HEAD
  │
  │ a1b2c3d4e5f6...
  ▼
Commit a1b2c3d
```

## Branch Operations

### Create Branch

```
Before:
  main
    ↓
    ●  a1b2c3d

After: gvc branch create feature-x
  main    feature-x
    ↓         ↓
    ●  a1b2c3d
```

### Checkout Branch

```
Before:
  HEAD → main
           ↓
           ●  a1b2c3d

After: gvc checkout feature-x
  HEAD → feature-x
              ↓
              ●  a1b2c3d
```

## Complete Workflow Example

```
1. Initialize
   $ gvc init
   
   Creates: .gvc/ directory structure

2. Create files
   $ echo "Hello" > file.txt
   
   Working Directory:
     file.txt (untracked)

3. Stage files
   $ gvc add file.txt
   
   Index:
     file.txt → blob:abc123
   
   Objects:
     .gvc/objects/ab/c123... (blob)

4. Commit
   $ gvc commit -m "Add file"
   
   Objects:
     .gvc/objects/ab/c123... (blob)
     .gvc/objects/de/f456... (tree)
     .gvc/objects/gh/i789... (commit)
   
   Refs:
     refs/heads/main → ghi789
   
   HEAD:
     ref: refs/heads/main

5. View history
   $ gvc log
   
   commit ghi789...
   Author: You
   Date:   ...
   
       Add file
```

## Data Flow

```
┌──────────────┐
│   User       │
│   Commands   │
└──────┬───────┘
       │
       │ gvc add, commit, log, ...
       ▼
┌──────────────┐
│   CLI        │
│  (gvc-cli)   │
└──────┬───────┘
       │
       │ Parse args, call core functions
       ▼
┌──────────────┐
│   Core       │
│  (gvc-core)  │
│              │
│  Repository  │
│  Storage     │
│  Index       │
│  Refs        │
└──────┬───────┘
       │
       │ Read/Write
       ▼
┌──────────────┐
│  Filesystem  │
│   .gvc/      │
└──────────────┘
```

## Module Architecture

```
┌─────────────────────────────────────────┐
│          Cargo Workspace                │
│                                         │
│  ┌──────────┐  ┌──────────┐  ┌──────┐ │
│  │ gvc-core │  │ gvc-cli  │  │ gvc- │ │
│  │          │◄─┤          │  │server│ │
│  │ - object │  │ - main   │  │      │ │
│  │ - storage│  │ - commands│ │(Phase│ │
│  │ - index  │  │          │  │  4)  │ │
│  │ - refs   │  │          │  │      │ │
│  │ - repo   │  │          │  │      │ │
│  └──────────┘  └──────────┘  └──────┘ │
│                                         │
└─────────────────────────────────────────┘
```

## Phase Progression

```
Phase 1 (MVP) ✅
  ├─ Core objects
  ├─ Init, add, commit
  ├─ Log, status
  ├─ Branches, tags
  └─ Basic CLI

Phase 2 🚧
  ├─ Diff
  ├─ Improved status
  ├─ Checkout (working dir)
  └─ .gvcignore

Phase 3 📋
  ├─ Module system
  ├─ Hooks
  └─ Templates

Phase 4 📋
  ├─ Server
  ├─ Push/Pull
  └─ Remote

Phase 5 📋
  ├─ Optimization
  ├─ Packfiles
  └─ GC

Phase 6 💡
  ├─ Merge
  ├─ Rebase
  └─ Advanced features
```

## Comparison: Git vs GVC

```
┌─────────────────┬──────────────┬──────────────┐
│    Feature      │     Git      │     GVC      │
├─────────────────┼──────────────┼──────────────┤
│ Hash Algorithm  │ SHA-1→SHA-256│   SHA-256    │
│ Language        │      C       │     Rust     │
│ Storage         │  Packfiles   │ Loose (MVP)  │
│ Branches        │      ✓       │      ✓       │
│ Tags            │      ✓       │      ✓       │
│ Merge           │      ✓       │   Phase 2+   │
│ Remote          │      ✓       │   Phase 4    │
│ Modules         │  Submodules  │   Custom     │
└─────────────────┴──────────────┴──────────────┘
```

## Performance Characteristics

```
Operation          Time Complexity    Space Complexity
─────────────────────────────────────────────────────
Hash computation   O(n)              O(1)
Object storage     O(1)              O(n)
Object retrieval   O(1)              O(1)
Tree building      O(n log n)        O(n)
Commit creation    O(n)              O(1)
Log traversal      O(k)              O(k)
  where k = number of commits to show

n = file size or number of files
```

## Security Model

```
┌──────────────────────────────────────────┐
│         Content Integrity                │
│                                          │
│  File → SHA-256 → Hash                   │
│                    ↓                     │
│         Verify on read                   │
│                    ↓                     │
│         ✓ Match = OK                     │
│         ✗ Mismatch = Corrupted           │
└──────────────────────────────────────────┘

Benefits:
✓ Detects corruption
✓ Prevents tampering
✓ SHA-256 collision-resistant
```

---

## Legend

```
●  Commit
│  Parent relationship
→  Reference/pointer
┌─┐ Container/box
✓  Completed/success
✗  Error/failure
🚧 In progress
📋 Planned
💡 Future
```

---

For more details, see:
- [ARCHITECTURE.md](ARCHITECTURE.md) - Technical deep dive
- [USAGE.md](USAGE.md) - Command reference
- [ROADMAP.md](ROADMAP.md) - Development plan

