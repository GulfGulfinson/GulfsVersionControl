# Test script for GVC
# This script demonstrates basic GVC functionality

Write-Host "=== GVC Test Script ===" -ForegroundColor Cyan
Write-Host ""

# Create test directory
$testDir = "gvc-test-$(Get-Date -Format 'yyyyMMdd-HHmmss')"
Write-Host "Creating test directory: $testDir" -ForegroundColor Yellow
New-Item -ItemType Directory -Path $testDir | Out-Null
Set-Location $testDir

try {
    # Initialize repository
    Write-Host "`n1. Initializing repository..." -ForegroundColor Green
    gvc init
    if ($LASTEXITCODE -ne 0) { throw "Init failed" }

    # Create test files
    Write-Host "`n2. Creating test files..." -ForegroundColor Green
    "# Test Project" | Out-File -Encoding UTF8 README.md
    "fn main() { println!(`"Hello, GVC!`"); }" | Out-File -Encoding UTF8 main.rs
    New-Item -ItemType Directory -Path src | Out-Null
    "pub fn greet() { println!(`"Hello!`"); }" | Out-File -Encoding UTF8 src/lib.rs

    # Check status
    Write-Host "`n3. Checking status..." -ForegroundColor Green
    gvc status

    # Add files
    Write-Host "`n4. Adding files..." -ForegroundColor Green
    gvc add .
    if ($LASTEXITCODE -ne 0) { throw "Add failed" }

    # Check status again
    Write-Host "`n5. Status after add..." -ForegroundColor Green
    gvc status

    # Commit
    Write-Host "`n6. Creating commit..." -ForegroundColor Green
    gvc commit -m "Initial commit"
    if ($LASTEXITCODE -ne 0) { throw "Commit failed" }

    # View log
    Write-Host "`n7. Viewing commit history..." -ForegroundColor Green
    gvc log --oneline

    # Test ignore functionality
    Write-Host "`n7a. Testing .gvcignore..." -ForegroundColor Green
    "*.log" | Out-File -Encoding UTF8 .gvcignore
    "This should be ignored" | Out-File -Encoding UTF8 debug.log
    gvc status
    Write-Host "debug.log should not appear above" -ForegroundColor Yellow

    # Test diff
    Write-Host "`n7b. Testing diff..." -ForegroundColor Green
    "Additional content" | Out-File -Append -Encoding UTF8 README.md
    gvc diff
    
    # Test reset
    Write-Host "`n7c. Testing reset..." -ForegroundColor Green
    gvc add README.md
    gvc reset README.md
    gvc status

    # Create branch
    Write-Host "`n8. Creating feature branch..." -ForegroundColor Green
    gvc branch create feature-test
    if ($LASTEXITCODE -ne 0) { throw "Branch create failed" }

    # List branches
    Write-Host "`n9. Listing branches..." -ForegroundColor Green
    gvc branch list

    # Switch to feature branch
    Write-Host "`n10. Switching to feature branch..." -ForegroundColor Green
    gvc checkout feature-test
    if ($LASTEXITCODE -ne 0) { throw "Checkout failed" }

    # Make changes
    Write-Host "`n11. Making changes on feature branch..." -ForegroundColor Green
    "`n// New feature" | Out-File -Append -Encoding UTF8 src/lib.rs
    gvc add src/lib.rs
    gvc commit -m "Add feature"

    # View log
    Write-Host "`n12. Viewing updated history..." -ForegroundColor Green
    gvc log --oneline

    # Create tag
    Write-Host "`n13. Creating tag..." -ForegroundColor Green
    gvc tag create v0.1.0
    if ($LASTEXITCODE -ne 0) { throw "Tag create failed" }

    # List tags
    Write-Host "`n14. Listing tags..." -ForegroundColor Green
    gvc tag list

    # Switch back to main
    Write-Host "`n15. Switching back to main..." -ForegroundColor Green
    gvc checkout main
    if ($LASTEXITCODE -ne 0) { throw "Checkout main failed" }

    # Show object
    Write-Host "`n16. Showing commit object..." -ForegroundColor Green
    $commits = gvc log --oneline
    if ($commits -and $commits.Count -gt 0) {
        $firstCommit = $commits[0] -split ' ' | Select-Object -First 1
        Write-Host "Showing commit: $firstCommit" -ForegroundColor Cyan
        gvc show $firstCommit
    }

    Write-Host "`n=== All tests passed! ===" -ForegroundColor Green

} catch {
    Write-Host "`nError: $_" -ForegroundColor Red
    Write-Host "Test failed!" -ForegroundColor Red
} finally {
    # Cleanup
    Set-Location ..
    Write-Host "`nTest directory: $testDir" -ForegroundColor Yellow
    Write-Host "To clean up: Remove-Item -Recurse -Force $testDir" -ForegroundColor Yellow
}

