#!/bin/bash
# GVC Commit Convention Module - Pre-commit Hook
# Validates commit messages follow conventional commits format

echo "📝 Validating commit message..."

# Pattern: type(scope): subject
# type: feat|fix|docs|style|refactor|test|chore|perf
# scope: optional
# subject: 1-72 characters

TYPES="feat|fix|docs|style|refactor|test|chore|perf"
PATTERN="^(${TYPES})(\(.+\))?: .{1,72}$"

if ! echo "$GVC_MESSAGE" | grep -qE "$PATTERN"; then
    echo "❌ Invalid commit message format!"
    echo ""
    echo "Current message:"
    echo "  $GVC_MESSAGE"
    echo ""
    echo "Expected format:"
    echo "  <type>(<scope>): <subject>"
    echo ""
    echo "Types: feat, fix, docs, style, refactor, test, chore, perf"
    echo "Scope: optional, e.g., (api), (ui)"
    echo "Subject: 1-72 characters"
    echo ""
    echo "Examples:"
    echo "  feat: add user authentication"
    echo "  fix(api): handle null pointer exception"
    echo "  docs: update README with installation instructions"
    echo ""
    exit 1
fi

echo "✅ Commit message format is valid"
exit 0

