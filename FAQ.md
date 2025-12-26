# GVC - Frequently Asked Questions

Common questions and answers about GulfsVersionControl.

---

## General Questions

### What is GVC?

GVC (GulfsVersionControl) is a Git-inspired version control system written in Rust from scratch. It's designed for learning, experimentation, and innovation while providing real, usable functionality.

### Why create another VCS?

- **Learning:** Understand version control internals
- **Innovation:** Experiment with new ideas (module system)
- **Rust Practice:** Build a complex system in Rust
- **Independence:** Not constrained by Git's design

### Is GVC production-ready?

For **local development**: Yes! Phases 1-3 are complete and stable.  
For **remote/team use**: Not yet (Phase 4 needed).

### Can I use GVC for real projects?

Yes, for local development. It's feature-complete for:
- Version tracking
- Branching
- Diffing
- Custom workflows (modules)

### Should I replace Git with GVC?

No. GVC is educational and experimental. Git is battle-tested and widely supported.

---

## Installation & Setup

### How do I install Rust?

**Windows:**
```powershell
Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile "$env:TEMP\rustup-init.exe"
& "$env:TEMP\rustup-init.exe" -y
```

**Linux/macOS:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### How do I build GVC?

```bash
cd GulfsControlSystem
cargo build --release
cargo install --path gvc-cli
```

### Installation fails - what do I do?

1. Update Rust: `rustup update`
2. Clean build: `cargo clean && cargo build`
3. Check dependencies: `cargo check`

### Can I install GVC without Rust?

No. GVC requires Rust to build. Pre-built binaries are not yet available.

---

## Basic Usage

### How do I start using GVC?

```bash
gvc init
gvc add .
gvc commit -m "Initial commit"
```

See [QUICKSTART.md](QUICKSTART.md) for more.

### How is GVC different from Git?

| Feature | Git | GVC |
|---------|-----|-----|
| Hash | SHA-1 | SHA-256 |
| Language | C | Rust |
| Modules | Submodules | Custom module system |
| Remote | ✅ | ❌ (Phase 4) |

### Can I convert a Git repository to GVC?

Not automatically. You'd need to:
1. Initialize GVC: `gvc init`
2. Add files: `gvc add .`
3. Commit: `gvc commit -m "Import from Git"`

### Can I use GVC and Git together?

Yes! They use different directories (`.git/` vs `.gvc/`). Add `.gvc/` to `.gitignore`.

---

## Commands

### What commands are available?

See [CHEATSHEET.md](CHEATSHEET.md) for complete list. Main commands:

```bash
gvc init, add, commit, log, status, show
gvc branch create/delete/list
gvc checkout, switch
gvc tag create/list
gvc diff, reset
gvc module create/install/add/remove/list/info
```

### How do I undo a commit?

GVC doesn't have `reset --hard` yet (Phase 2+). Workaround:
```bash
# Create new commit reverting changes
gvc checkout <previous-commit>
gvc branch create revert-changes
```

### How do I see what changed?

```bash
gvc diff              # Unstaged changes
gvc diff --staged     # Staged changes
gvc log               # Commit history
```

### How do I unstage files?

```bash
gvc reset <file>      # Specific file
gvc reset             # All files
```

---

## Diff & Status

### Why doesn't diff show colors?

Colors should work by default. If not:
- Windows: Use modern terminal (Windows Terminal, not cmd.exe)
- Linux/macOS: Check terminal supports ANSI colors

### What diff algorithm does GVC use?

Myers' Longest Common Subsequence (LCS) algorithm.

### Can I diff binary files?

GVC detects binary files and shows "Binary file" message instead of diff.

### Status shows too much - how to simplify?

```bash
gvc status | grep "Changes to be committed"
gvc log --oneline
```

---

## Branching

### How do I create a branch?

```bash
gvc branch create <name>
gvc checkout <name>
```

### How do I delete a branch?

```bash
gvc checkout main           # Switch away first
gvc branch delete <name>
```

### Can I merge branches?

Not yet (Phase 3+). Currently only linear history.

### How do I see all branches?

```bash
gvc branch list
```

Current branch is marked with `*`.

---

## Ignoring Files

### How do I ignore files?

Create `.gvcignore`:
```
*.log
target/
*.tmp
!important.log
```

### What pattern syntax is supported?

```
*           # Any characters
?           # Single character
**          # Recursive
!pattern    # Negation
dir/        # Directory only
```

### Can I ignore files already committed?

No. Remove them first:
```bash
rm file.log
gvc commit -m "Remove log file"
```
Then add to `.gvcignore`.

### Does GVC respect .gitignore?

No. Use `.gvcignore` instead.

---

## Modules

### What are modules?

Modules extend GVC with custom functionality (hooks, templates, config). Think npm packages for version control.

### How do I create a module?

```bash
gvc module create my-module --author "Your Name"
cd my-module
# Edit module.toml and hooks/
gvc module install .
```

### Where are modules stored?

- Global: `~/.gvc/modules/`
- Active: `.gvc/modules/active/` (symlinks)

### How do I share modules?

1. Create git repository
2. Push to GitHub/GitLab
3. Users install: `gvc module install <path>`

### Can I have multiple modules active?

Yes! All active modules run their hooks.

### How do I debug a module?

```bash
# Check installation
gvc module list

# View manifest
cat ~/.gvc/modules/module@version/module.toml

# Test hook manually
bash ~/.gvc/modules/module@version/hooks/pre-commit.sh
```

---

## Hooks

### What hooks are available?

- `pre-commit` - Before commit (can block)
- `post-commit` - After commit
- `pre-checkout` - Before checkout (can block)
- `post-checkout` - After checkout
- `pre-push` - Before push (Phase 4)
- `post-pull` - After pull (Phase 4)

### How do I create a hook?

**Linux/macOS:**
```bash
cat > .gvc/hooks/pre-commit.sh << 'EOF'
#!/bin/bash
echo "Running pre-commit hook"
exit 0
EOF
chmod +x .gvc/hooks/pre-commit.sh
```

**Windows:**
```powershell
cat > .gvc/hooks/pre-commit.ps1 << 'EOF'
Write-Host "Running pre-commit hook"
exit 0
EOF
```

### Can hooks prevent commits?

**Pre-hooks:** Yes (exit non-zero)  
**Post-hooks:** No (always run after operation)

### What environment variables are available?

```
GVC_AUTHOR          # Commit author
GVC_MESSAGE         # Commit message
GVC_COMMIT_HASH     # Commit hash
GVC_BRANCH          # Branch name
MODULE_NAME         # Module name
MODULE_PATH         # Module path
```

### How do I disable hooks temporarily?

```bash
# Deactivate module
gvc module remove module@version

# Or rename hook
mv .gvc/hooks/pre-commit.sh .gvc/hooks/pre-commit.sh.disabled
```

---

## Performance

### Is GVC fast?

For small-medium repos, yes. Large repos may be slower due to:
- No packfiles yet (Phase 5)
- No delta compression (Phase 5)
- Full snapshots instead of incremental

### How much disk space does GVC use?

More than Git initially (no compression). Phase 5 will add optimization.

### Can I speed up operations?

- Use `.gvcignore` to skip unnecessary files
- Keep repositories focused/small
- Phase 5 will add performance optimizations

---

## Errors & Troubleshooting

### "Repository not found"

```bash
# Check if initialized
ls .gvc

# Initialize if needed
gvc init
```

### "Nothing to commit"

Stage files first:
```bash
gvc add <files>
```

### "Branch not found"

```bash
# List available branches
gvc branch list

# Create if needed
gvc branch create <name>
```

### Hook fails with permission denied

**Linux/macOS:**
```bash
chmod +x .gvc/hooks/pre-commit.sh
```

**Windows:** Check PowerShell execution policy:
```powershell
Set-ExecutionPolicy -Scope CurrentUser RemoteSigned
```

### "Module not found"

```bash
# Check installation
gvc module list

# Install if needed
gvc module install <path>
```

### Commit blocked by hook

Check hook output for reason. To bypass (not recommended):
```bash
# Deactivate module temporarily
gvc module remove module@version
# Commit
gvc commit -m "message"
# Reactivate
gvc module add module@version
```

---

## Comparison with Git

### Should I learn GVC if I know Git?

If you want to understand version control internals, yes! GVC is educational.

### Can GVC replace Git for my team?

Not yet. Wait for Phase 4 (remote/push/pull).

### What features does GVC have that Git doesn't?

- **Custom module system** (vs submodules)
- **TOML manifests** (vs limited config)
- **Integrated hook management**
- **SHA-256 by default**

### What features does Git have that GVC doesn't?

- Remote repositories (Phase 4)
- Merge (Phase 3+)
- Rebase (Phase 6)
- Stash (Phase 6)
- Packfiles (Phase 5)
- Submodules
- Large file support
- Decades of tooling

---

## Future Plans

### When will Phase 4 (Remote) be ready?

No fixed timeline. See [ROADMAP.md](ROADMAP.md) for details.

### Will GVC support GitHub/GitLab?

Phase 4 will add custom server. GitHub/GitLab integration not planned.

### Can I contribute?

Yes! See [CONTRIBUTING.md](CONTRIBUTING.md).

### Will there be GUI?

Not currently planned. CLI-first philosophy.

### Will GVC be faster than Git?

Potentially, with Phase 5 optimizations. But Git is very optimized.

---

## Technical Questions

### What hash algorithm does GVC use?

SHA-256 (64 hex characters = 256 bits).

### How are objects stored?

Content-addressable: `.gvc/objects/ab/cdef123...`

First 2 hex chars = directory, rest = filename.

### What serialization format?

Binary with `bincode` (efficient, type-safe).

### Why Rust?

- Memory safety
- Performance
- Modern tooling
- Learning opportunity

### Can I read GVC repositories with other tools?

No. GVC uses custom format (not Git-compatible).

---

## Getting Help

### Where can I get help?

1. Read documentation:
   - [TUTORIAL.md](TUTORIAL.md)
   - [USAGE.md](USAGE.md)
   - [CHEATSHEET.md](CHEATSHEET.md)

2. Check examples:
   - [examples/](examples/)

3. Run help commands:
   ```bash
   gvc --help
   gvc <command> --help
   ```

### How do I report bugs?

Open an issue on the project repository with:
- GVC version
- Operating system
- Steps to reproduce
- Expected vs actual behavior

### How do I request features?

Open an issue with:
- Use case
- Why it's important
- Proposed implementation (optional)

---

## Miscellaneous

### What does "Gulfs" mean?

It's a personal project name. Not an acronym.

### Is GVC open source?

Yes! MIT licensed.

### Can I use GVC commercially?

Yes, MIT license allows commercial use.

### Who maintains GVC?

Currently a solo/learning project. See [CONTRIBUTING.md](CONTRIBUTING.md) to help!

---

**Have more questions?** Check the [documentation](INDEX.md) or open an issue!

