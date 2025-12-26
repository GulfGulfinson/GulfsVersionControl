# GVC Example Modules

This directory contains complete, ready-to-use example modules demonstrating the GVC module system.

---

## Available Modules

### 1. Rust Linter Module

**Path:** `rust-linter-module/`  
**Version:** 1.0.0  
**Purpose:** Automatically runs `cargo clippy` before commits to ensure code quality.

**Features:**
- ✅ Runs clippy with `-D warnings`
- ✅ Blocks commits if linting fails
- ✅ Skips check if not a Rust project
- ✅ Post-commit notifications

**Installation:**
```bash
gvc module install ./examples/rust-linter-module
cd your-rust-project
gvc module add rust-linter@1.0.0
```

**Use Case:** Ensure all Rust code passes clippy checks before committing.

---

### 2. Commit Convention Module

**Path:** `commit-convention-module/`  
**Version:** 1.0.0  
**Purpose:** Enforces [Conventional Commits](https://www.conventionalcommits.org/) format.

**Features:**
- ✅ Validates commit message format
- ✅ Requires type (feat, fix, docs, etc.)
- ✅ Optional scope support
- ✅ Clear error messages with examples

**Installation:**
```bash
gvc module install ./examples/commit-convention-module
cd your-project
gvc module add commit-convention@1.0.0
```

**Use Case:** Maintain consistent commit message format across team.

---

## Quick Start

### Install All Examples

```bash
# From project root
cd examples

# Install rust linter
gvc module install ./rust-linter-module

# Install commit convention
gvc module install ./commit-convention-module
```

### Use in Your Project

```bash
cd your-project
gvc init  # if not already initialized

# Activate modules
gvc module add rust-linter@1.0.0
gvc module add commit-convention@1.0.0

# Verify
gvc module list
```

### Test

```bash
# Make changes
echo "fn main() {}" > test.rs
gvc add test.rs

# Try committing (both modules will run)
gvc commit -m "feat: add test file"
```

Expected output:
```
📝 Validating commit message...
✅ Commit message format is valid
🔍 Running Rust linter (clippy)...
✅ Clippy checks passed!
[abc123d] feat: add test file
✅ Commit successful!
📦 Rust linter verified your code
```

---

## Module Structure

Each module follows the same structure:

```
module-name/
├── module.toml       # Module manifest
├── hooks/            # Hook scripts
│   ├── pre-commit.sh
│   └── post-commit.sh
├── templates/        # Optional templates
└── README.md         # Module documentation
```

### module.toml

```toml
[module]
name = "module-name"
version = "1.0.0"
description = "Module description"
authors = ["Author Name"]
license = "MIT"

[hooks]
pre-commit = "hooks/pre-commit.sh"
post-commit = "hooks/post-commit.sh"

[config]
# Module-specific configuration
```

---

## Creating Your Own Module

### 1. Create Scaffold

```bash
gvc module create my-awesome-module --author "Your Name"
cd my-awesome-module
```

### 2. Edit module.toml

```toml
[module]
name = "my-awesome-module"
version = "0.1.0"
description = "Does something awesome"
authors = ["Your Name"]

[hooks]
pre-commit = "hooks/check.sh"
```

### 3. Create Hook

```bash
cat > hooks/check.sh << 'EOF'
#!/bin/bash
echo "Running awesome checks..."
# Your logic here
exit 0
EOF

chmod +x hooks/check.sh
```

### 4. Test

```bash
# Install
gvc module install .

# Use in test project
cd ../test-project
gvc module add my-awesome-module@0.1.0

# Test
gvc commit -m "test"
```

---

## Module Ideas

### Suggested Modules to Build

1. **Test Runner**
   - Run tests before commit
   - Block commit if tests fail

2. **Code Formatter**
   - Auto-format code (rustfmt, prettier, black)
   - Stage formatted files

3. **TODO Checker**
   - Warn if TODO/FIXME in commit message
   - List TODOs in code

4. **Security Scanner**
   - Check for hardcoded secrets
   - Scan dependencies for vulnerabilities

5. **Documentation Generator**
   - Auto-generate docs on commit
   - Update changelog

6. **Notification System**
   - Send Slack/Discord notifications
   - Email on important commits

7. **Branch Naming Validator**
   - Enforce branch naming conventions
   - Check branch prefixes

8. **File Size Checker**
   - Warn about large files
   - Block files over size limit

9. **Language Detector**
   - Detect project language
   - Run language-specific checks

10. **CI Integration**
    - Trigger CI builds
    - Check CI status

---

## Hook Best Practices

### 1. Exit Codes

```bash
# Success
exit 0

# Failure (blocks commit)
exit 1
```

### 2. Error Messages

```bash
if [ $? -ne 0 ]; then
    echo "❌ Error: Operation failed"
    echo ""
    echo "To fix:"
    echo "  1. Do this"
    echo "  2. Then that"
    exit 1
fi
```

### 3. Environment Variables

```bash
# Use GVC environment variables
echo "Author: $GVC_AUTHOR"
echo "Message: $GVC_MESSAGE"
echo "Module: $MODULE_NAME"
```

### 4. Performance

```bash
# Quick checks first
if [ ! -f "Cargo.toml" ]; then
    # Skip if not relevant
    exit 0
fi

# Expensive checks later
cargo test
```

### 5. User Feedback

```bash
echo "🔍 Running checks..."
# ... do work ...
echo "✅ All checks passed!"
```

---

## Cross-Platform Hooks

### Bash (Linux/macOS)

```bash
#!/bin/bash
echo "Running on Unix"
```

### PowerShell (Windows)

```powershell
# GVC Hook
Write-Host "Running on Windows"
```

### Make Executable

```bash
# Linux/macOS
chmod +x hooks/pre-commit.sh

# Windows (PowerShell)
# No action needed for .ps1 files
```

---

## Debugging Modules

### Check Module Installation

```bash
gvc module list
ls ~/.gvc/modules/
```

### Check Active Modules

```bash
gvc module list --active
ls .gvc/modules/active/
```

### Test Hook Manually

```bash
# Set environment variables
export GVC_AUTHOR="Test"
export GVC_MESSAGE="test message"

# Run hook
bash .gvc/modules/active/module@version/hooks/pre-commit.sh
```

### View Hook Output

```bash
# Hooks output to stdout/stderr
gvc commit -m "test"  # Watch output
```

---

## Module Distribution

### Share Your Module

1. **Create Repository**
   ```bash
   cd my-module
   git init
   git add .
   git commit -m "Initial commit"
   git remote add origin https://github.com/you/my-module
   git push -u origin main
   ```

2. **Document Installation**
   ```markdown
   # Installation
   
   git clone https://github.com/you/my-module
   gvc module install ./my-module
   gvc module add my-module@1.0.0
   ```

3. **Version Releases**
   - Update version in `module.toml`
   - Tag releases: `git tag v1.0.0`
   - Document changes in README

---

## FAQ

**Q: Can I use multiple modules?**  
A: Yes! Activate as many as you need. They all run in order.

**Q: How do I temporarily disable a module?**  
A: `gvc module remove module@version`

**Q: Can hooks modify files?**  
A: Yes, but re-stage modified files: `gvc add <file>`

**Q: What if a hook fails?**  
A: Pre-hooks block the operation. Post-hooks don't.

**Q: Can I write hooks in Python/Ruby/etc?**  
A: Yes! Just use a shebang: `#!/usr/bin/env python3`

**Q: Where are modules stored?**  
A: Globally in `~/.gvc/modules/`, active via symlinks in `.gvc/modules/active/`

---

## Resources

- [Module System Documentation](../PHASE3_COMPLETE.md)
- [Tutorial](../TUTORIAL.md)
- [Architecture Guide](../ARCHITECTURE.md)

---

## Contributing

Have a useful module? Share it!

1. Create the module
2. Document it well
3. Open a PR to add it to this directory

---

**Happy Module Building!** 🚀

