#!/bin/bash
# Test script for GVC (Linux/macOS)
# This script demonstrates basic GVC functionality

set -e  # Exit on error

echo "=== GVC Test Script ==="
echo ""

# Create test directory
TEST_DIR="gvc-test-$(date +%Y%m%d-%H%M%S)"
echo "Creating test directory: $TEST_DIR"
mkdir "$TEST_DIR"
cd "$TEST_DIR"

# Initialize repository
echo ""
echo "1. Initializing repository..."
gvc init

# Create test files
echo ""
echo "2. Creating test files..."
echo "# Test Project" > README.md
echo 'fn main() { println!("Hello, GVC!"); }' > main.rs
mkdir src
echo 'pub fn greet() { println!("Hello!"); }' > src/lib.rs

# Check status
echo ""
echo "3. Checking status..."
gvc status

# Add files
echo ""
echo "4. Adding files..."
gvc add .

# Check status again
echo ""
echo "5. Status after add..."
gvc status

# Commit
echo ""
echo "6. Creating commit..."
gvc commit -m "Initial commit"

# View log
echo ""
echo "7. Viewing commit history..."
gvc log --oneline

# Test ignore functionality
echo ""
echo "7a. Testing .gvcignore..."
echo "*.log" > .gvcignore
echo "This should be ignored" > debug.log
gvc status
echo "debug.log should not appear above"

# Test diff
echo ""
echo "7b. Testing diff..."
echo "Additional content" >> README.md
gvc diff

# Test reset
echo ""
echo "7c. Testing reset..."
gvc add README.md
gvc reset README.md
gvc status

# Create branch
echo ""
echo "8. Creating feature branch..."
gvc branch create feature-test

# List branches
echo ""
echo "9. Listing branches..."
gvc branch list

# Switch to feature branch
echo ""
echo "10. Switching to feature branch..."
gvc checkout feature-test

# Make changes
echo ""
echo "11. Making changes on feature branch..."
echo "" >> src/lib.rs
echo "// New feature" >> src/lib.rs
gvc add src/lib.rs
gvc commit -m "Add feature"

# View log
echo ""
echo "12. Viewing updated history..."
gvc log --oneline

# Create tag
echo ""
echo "13. Creating tag..."
gvc tag create v0.1.0

# List tags
echo ""
echo "14. Listing tags..."
gvc tag list

# Switch back to main
echo ""
echo "15. Switching back to main..."
gvc checkout main

# Show object
echo ""
echo "16. Showing commit object..."
FIRST_COMMIT=$(gvc log --oneline | head -1 | cut -d' ' -f1)
echo "Showing commit: $FIRST_COMMIT"
gvc show "$FIRST_COMMIT"

echo ""
echo "=== All tests passed! ==="

# Cleanup info
cd ..
echo ""
echo "Test directory: $TEST_DIR"
echo "To clean up: rm -rf $TEST_DIR"

