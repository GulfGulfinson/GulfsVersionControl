#!/bin/bash
# GVC Rust Linter Module - Pre-commit Hook
# Runs cargo clippy to ensure code quality

echo "🔍 Running Rust linter (clippy)..."

# Check if this is a Rust project
if [ ! -f "Cargo.toml" ]; then
    echo "⚠️  No Cargo.toml found - skipping Rust linting"
    exit 0
fi

# Run clippy
cargo clippy -- -D warnings

RESULT=$?

if [ $RESULT -eq 0 ]; then
    echo "✅ Clippy checks passed!"
else
    echo "❌ Clippy found issues - commit blocked"
    echo ""
    echo "Fix the issues above or run:"
    echo "  cargo clippy --fix"
    echo ""
    echo "To skip this hook (not recommended):"
    echo "  gvc module remove rust-linter@1.0.0"
fi

exit $RESULT

