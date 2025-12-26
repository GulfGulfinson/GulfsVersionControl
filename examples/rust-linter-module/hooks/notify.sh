#!/bin/bash
# GVC Rust Linter Module - Post-commit Hook
# Provides helpful feedback after successful commit

echo "✅ Commit successful!"
echo "📦 Rust linter verified your code"
echo ""
echo "Next steps:"
echo "  • Run 'gvc push' to share your changes"
echo "  • Run 'cargo test' to verify tests"

