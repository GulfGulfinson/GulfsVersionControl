# Rust Linter Module for GVC

Automatically runs `cargo clippy` before each commit to ensure code quality.

## Features

- ✅ Runs clippy with `-D warnings` (treat warnings as errors)
- ✅ Blocks commits if linting fails
- ✅ Skips check if not a Rust project
- ✅ Helpful error messages
- ✅ Post-commit notifications

## Installation

```bash
# Install the module
gvc module install /path/to/rust-linter-module

# Activate in your Rust project
cd your-rust-project
gvc init  # if not already initialized
gvc module add rust-linter@1.0.0
```

## Usage

Once activated, the module automatically runs on every commit:

```bash
# Make changes
vim src/main.rs

# Stage changes
gvc add src/main.rs

# Commit - clippy runs automatically
gvc commit -m "Add new feature"
```

If clippy finds issues, the commit is blocked:

```
🔍 Running Rust linter (clippy)...
❌ Clippy found issues - commit blocked

warning: unused variable: `x`
  --> src/main.rs:5:9
   |
5  |     let x = 42;
   |         ^ help: if this is intentional, prefix it with an underscore: `_x`
```

## Configuration

Edit `module.toml` to customize:

```toml
[config]
clippy_args = ["-D", "warnings"]  # Clippy arguments
skip_tests = false                 # Skip test files
```

## Deactivation

To temporarily disable:

```bash
gvc module remove rust-linter@1.0.0
```

To remove completely:

```bash
# Remove from global installation
rm -rf ~/.gvc/modules/rust-linter@1.0.0
```

## Development

To modify the module:

1. Edit hooks or configuration
2. Reinstall:
   ```bash
   gvc module install .
   ```

## License

MIT

