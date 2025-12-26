# Commit Convention Module for GVC

Enforces [Conventional Commits](https://www.conventionalcommits.org/) specification for commit messages.

## Format

```
<type>(<scope>): <subject>

[optional body]

[optional footer]
```

**Example:**
```
feat(auth): add JWT token validation

Implements JWT token validation middleware for API endpoints.
Includes unit tests and documentation.

Closes #123
```

## Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks
- `perf`: Performance improvements

## Installation

```bash
gvc module install /path/to/commit-convention-module
cd your-project
gvc module add commit-convention@1.0.0
```

## Usage

The module validates commit messages automatically:

```bash
# ✅ Valid commit
gvc commit -m "feat: add user login"

# ✅ Valid commit with scope
gvc commit -m "fix(api): handle edge case"

# ❌ Invalid commit
gvc commit -m "added new feature"
# Error: Invalid commit message format!
```

## Configuration

Edit `module.toml`:

```toml
[config]
types = ["feat", "fix", "docs", "style", "refactor", "test", "chore", "perf"]
require_scope = false
max_subject_length = 72
```

## Benefits

- **Consistency**: All commits follow same format
- **Automation**: Generate changelogs automatically
- **Clarity**: Understand changes at a glance
- **Standards**: Follow industry best practices

## License

MIT

