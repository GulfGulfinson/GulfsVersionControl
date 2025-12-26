# Contributing to GulfsVersionControl

Thank you for your interest in contributing to GVC! This document provides guidelines and information for contributors.

## Getting Started

1. **Fork the repository**
2. **Clone your fork:**
   ```bash
   git clone https://github.com/yourusername/GulfsControlSystem.git
   cd GulfsControlSystem
   ```
3. **Install Rust** (see [INSTALLATION.md](INSTALLATION.md))
4. **Build the project:**
   ```bash
   cargo build
   ```
5. **Run tests:**
   ```bash
   cargo test
   ```

## Development Workflow

### 1. Create a Branch

```bash
git checkout -b feature/your-feature-name
```

Use prefixes:
- `feature/` - new features
- `fix/` - bug fixes
- `docs/` - documentation
- `refactor/` - code refactoring
- `test/` - test improvements

### 2. Make Changes

- Write clean, idiomatic Rust code
- Follow existing code style
- Add tests for new functionality
- Update documentation

### 3. Test Your Changes

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture

# Check formatting
cargo fmt --check

# Run linter
cargo clippy
```

### 4. Commit

Write clear commit messages:

```
Short summary (50 chars or less)

More detailed explanation if needed. Wrap at 72 characters.
Explain what and why, not how.

- Bullet points are okay
- Use present tense ("Add feature" not "Added feature")
```

### 5. Push and Create Pull Request

```bash
git push origin feature/your-feature-name
```

Then create a pull request on GitHub.

## Code Style

### Rust Guidelines

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` for formatting
- Use `clippy` for linting
- Prefer explicit types over `auto`/inference when it improves clarity
- Document public APIs with `///` comments

### Comments

Based on project preferences:

- Explain **why**, not **what**
- Document tradeoffs and alternatives considered
- Include TODOs for future improvements
- Avoid temporal references ("3 days ago")
- Avoid first-person perspective
- Keep comments concise

**Good:**
```rust
// BTreeMap ensures deterministic ordering for consistent hashing
// Alternative: HashMap would be faster but non-deterministic
let entries: BTreeMap<String, TreeEntry> = BTreeMap::new();
```

**Bad:**
```rust
// I created this map 3 days ago
// This is a map that stores entries
let entries = BTreeMap::new();
```

### Error Handling

- Use `Result<T, Error>` for fallible operations
- Use `thiserror` for custom errors
- Provide context in error messages
- Never panic in library code (only in tests or CLI)

### Testing

- Write unit tests for all public functions
- Use `#[cfg(test)]` modules
- Use `tempfile` for filesystem tests
- Test edge cases and error conditions

**Example:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_repository_init() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path()).unwrap();
        assert!(temp.path().join(".gvc").exists());
    }
}
```

## Project Structure

```
gvc-core/          # Core VCS logic (no I/O in tests)
├── error.rs       # Error types
├── hash.rs        # Hashing utilities
├── object.rs      # Object model
├── storage.rs     # Object storage
├── index.rs       # Staging area
├── refs.rs        # References
└── repository.rs  # High-level operations

gvc-cli/           # CLI interface
├── main.rs        # Argument parsing
└── commands/      # Command implementations

gvc-server/        # Server (Phase 4)
```

## Adding a New Feature

### Example: Adding `gvc diff`

1. **Design** (update [ARCHITECTURE.md](ARCHITECTURE.md)):
   - What does the feature do?
   - What's the API?
   - What are the edge cases?

2. **Core Implementation** (`gvc-core`):
   ```rust
   // In repository.rs
   pub fn diff(&self, staged: bool) -> Result<Vec<FileDiff>> {
       // Implementation
   }
   ```

3. **CLI Command** (`gvc-cli`):
   ```rust
   // In commands/mod.rs
   pub fn diff(staged: bool) -> anyhow::Result<()> {
       let repo = Repository::open(&env::current_dir()?)?;
       let diffs = repo.diff(staged)?;
       // Display diffs
       Ok(())
   }
   ```

4. **Tests**:
   ```rust
   #[test]
   fn test_diff() {
       // Test implementation
   }
   ```

5. **Documentation**:
   - Update [USAGE.md](USAGE.md)
   - Update [README.md](README.md)
   - Add rustdoc comments

## Pull Request Guidelines

### Before Submitting

- [ ] Code compiles without warnings
- [ ] All tests pass
- [ ] New tests added for new features
- [ ] Documentation updated
- [ ] `cargo fmt` applied
- [ ] `cargo clippy` passes
- [ ] Commit messages are clear

### PR Description

Include:
- What does this PR do?
- Why is this change needed?
- How was it tested?
- Any breaking changes?
- Related issues (if any)

### Review Process

1. Automated checks run (CI)
2. Code review by maintainers
3. Address feedback
4. Approval and merge

## Areas Needing Help

See [ROADMAP.md](ROADMAP.md) for planned features.

**Good first issues:**
- Improve error messages
- Add more tests
- Documentation improvements
- CLI usability enhancements

**Advanced features:**
- Diff implementation (Phase 2)
- Module system (Phase 3)
- Server implementation (Phase 4)

## Questions?

- Open an issue for discussion
- Check existing issues and PRs
- Read [ARCHITECTURE.md](ARCHITECTURE.md) for technical details

## Code of Conduct

- Be respectful and constructive
- Welcome newcomers
- Focus on the code, not the person
- Assume good intentions

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

