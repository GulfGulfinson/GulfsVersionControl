# GVC Command Cheatsheet

Quick reference for all GVC commands.

---

## Repository Management

```bash
gvc init [<path>]                    # Initialize repository
```

---

## Basic Commands

```bash
gvc status                           # Show working tree status
gvc add <file>...                    # Stage files
gvc add .                            # Stage all files
gvc commit -m "<message>"            # Create commit
gvc commit -m "msg" --author "Name" # Commit with specific author
gvc log                              # Show commit history
gvc log --oneline                    # Show compact history
gvc log -n 5                         # Show last 5 commits
gvc show <hash>                      # Show object content
```

---

## Diff & Changes

```bash
gvc diff                             # Show unstaged changes
gvc diff --staged                    # Show staged changes
gvc reset                            # Unstage all files
gvc reset <file>...                  # Unstage specific files
```

---

## Branching

```bash
gvc branch list                      # List all branches
gvc branch create <name>             # Create new branch
gvc branch delete <name>             # Delete branch
gvc checkout <branch>                # Switch to branch
gvc switch <branch>                  # Switch to branch (alias)
```

---

## Tags

```bash
gvc tag list                         # List all tags
gvc tag create <name>                # Create tag at HEAD
```

---

## Modules

```bash
# Module Management
gvc module create <name>             # Create module scaffold
gvc module install <path>            # Install module globally
gvc module add <identifier>          # Activate module in repo
gvc module remove <identifier>       # Deactivate module
gvc module list                      # List all modules
gvc module list --active             # List active modules only
gvc module list --installed          # List installed modules only
gvc module info <identifier>         # Show module details

# Module identifier format: name@version
# Example: my-module@1.0.0
```

---

## Files & Patterns

### .gvcignore

```bash
# Patterns
*.log                # All .log files
*.tmp                # All .tmp files
target/              # Directory
**/*.pyc             # Recursive pattern
!important.log       # Exception (don't ignore)
```

---

## Environment Variables

```bash
# Set author
export GVC_AUTHOR="Your Name"        # Linux/macOS
$env:GVC_AUTHOR = "Your Name"        # Windows PowerShell

# Used in hooks
GVC_AUTHOR          # Commit author
GVC_MESSAGE         # Commit message
GVC_COMMIT_HASH     # Commit hash (post-commit)
GVC_BRANCH          # Branch name (checkout)
MODULE_NAME         # Module name (in module hooks)
MODULE_PATH         # Module path (in module hooks)
```

---

## Hook Types

| Hook | When | Can Block | Use Case |
|------|------|-----------|----------|
| `pre-commit` | Before commit | ✅ | Linting, tests |
| `post-commit` | After commit | ❌ | Notifications |
| `pre-checkout` | Before checkout | ✅ | Validation |
| `post-checkout` | After checkout | ❌ | Setup |
| `pre-push` | Before push* | ✅ | CI checks |
| `post-pull` | After pull* | ❌ | Updates |

\* Phase 4

---

## Common Workflows

### Start New Project

```bash
mkdir project && cd project
gvc init
echo "*.log" > .gvcignore
gvc add .
gvc commit -m "Initial commit"
```

### Feature Branch

```bash
gvc branch create feature-x
gvc checkout feature-x
# ... make changes ...
gvc add .
gvc commit -m "Add feature X"
gvc checkout main
```

### Check Before Commit

```bash
gvc status
gvc diff --staged
gvc commit -m "message"
```

### Undo Staging

```bash
gvc add wrong-file.txt
gvc reset wrong-file.txt
```

### Install & Use Module

```bash
# Install
gvc module install ./my-module

# Activate
gvc module add my-module@1.0.0

# Verify
gvc module list

# Test
gvc commit -m "test"  # Runs module hooks
```

---

## Tips

### Aliases (Bash/Zsh)

```bash
alias gs='gvc status'
alias ga='gvc add'
alias gc='gvc commit'
alias gl='gvc log --oneline'
alias gd='gvc diff'
alias gco='gvc checkout'
alias gb='gvc branch list'
```

### Aliases (PowerShell)

```powershell
Set-Alias gs 'gvc status'
Set-Alias ga 'gvc add'
Set-Alias gc 'gvc commit'
Set-Alias gl 'gvc log --oneline'
Set-Alias gd 'gvc diff'
```

---

## File Locations

```bash
~/.gvc/modules/              # Global modules
.gvc/                        # Repository data
.gvc/hooks/                  # Repository hooks
.gvc/modules/active/         # Active modules (symlinks)
.gvcignore                   # Ignore patterns
```

---

## Exit Codes

```
0   Success
1   Error (generic)
```

---

## Help

```bash
gvc --help                   # General help
gvc <command> --help         # Command-specific help
gvc --version                # Show version
```

---

## Examples

### Complete Session

```bash
# Setup
gvc init
export GVC_AUTHOR="John Doe"

# Work
echo "Hello" > file.txt
gvc add file.txt
gvc commit -m "Add file"

# Branch
gvc branch create feature
gvc checkout feature
echo "More" >> file.txt
gvc add file.txt
gvc commit -m "Update file"

# Status & Log
gvc status
gvc log --oneline
gvc diff

# Tag
gvc checkout main
gvc tag create v1.0
```

### With Modules

```bash
# Create linter module
gvc module create my-linter
cd my-linter
# ... edit hooks ...
gvc module install .

# Use in project
cd ../project
gvc module add my-linter@0.1.0
gvc commit -m "test"  # Linter runs automatically
```

---

## Troubleshooting

| Problem | Solution |
|---------|----------|
| "Repository not found" | Run `gvc init` |
| "Nothing to commit" | Run `gvc add <files>` first |
| "Branch not found" | Check `gvc branch list` |
| Hook doesn't run | Check permissions, make executable |
| Module not working | Run `gvc module list`, verify active |

---

## More Information

- **Tutorial:** [TUTORIAL.md](TUTORIAL.md)
- **Full Reference:** [USAGE.md](USAGE.md)
- **Architecture:** [ARCHITECTURE.md](ARCHITECTURE.md)
- **Examples:** [examples/](examples/)

---

**Print this page for quick reference!** 📄

