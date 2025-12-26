# Windows Setup Guide for GVC

Complete setup instructions for Windows users.

---

## Prerequisites

- Windows 10 or later
- PowerShell 5.1 or later (comes with Windows)
- Internet connection

---

## Step 1: Install Rust

### Option A: Automated (Recommended)

Open **PowerShell as Administrator**:

```powershell
# Download and run rustup installer
Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile "$env:TEMP\rustup-init.exe"

# Run installer (follow prompts, press Enter to accept defaults)
& "$env:TEMP\rustup-init.exe" -y

# Refresh PATH
$env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
```

### Option B: Manual

1. Visit https://rustup.rs/
2. Download `rustup-init.exe`
3. Run it
4. Follow on-screen instructions
5. Close and reopen PowerShell

### Verify Installation

```powershell
rustc --version
cargo --version
```

Expected output:
```
rustc 1.70.0 (or newer)
cargo 1.70.0 (or newer)
```

---

## Step 2: Build GVC

### Navigate to Project

```powershell
cd C:\Users\YourUsername\Documents\proxy\privat\GulfsControlSystem
```

### Build

```powershell
# Build release version (optimized)
cargo build --release

# This will take 2-5 minutes on first build
```

### Install

```powershell
# Install to ~/.cargo/bin/
cargo install --path gvc-cli

# Verify
gvc --version
```

Expected output:
```
gvc 0.3.0
```

---

## Step 3: Test Installation

### Run Tests

```powershell
# Run all tests
cargo test

# Run integration test
.\test-gvc.ps1
```

---

## Step 4: First Repository

```powershell
# Create test directory
mkdir C:\Users\YourUsername\Documents\test-gvc
cd C:\Users\YourUsername\Documents\test-gvc

# Initialize
gvc init

# Create file
"Hello, GVC!" | Out-File -Encoding UTF8 hello.txt

# Add and commit
gvc add hello.txt
gvc commit -m "Initial commit"

# View log
gvc log
```

---

## Configuration

### Set Author Name

**Temporary (current session):**
```powershell
$env:GVC_AUTHOR = "Your Name"
```

**Permanent (add to PowerShell profile):**
```powershell
# Open profile
notepad $PROFILE

# Add this line:
$env:GVC_AUTHOR = "Your Name"

# Save and reload
. $PROFILE
```

### Create Aliases

Add to PowerShell profile (`$PROFILE`):

```powershell
Set-Alias -Name gs -Value 'gvc status'
Set-Alias -Name ga -Value 'gvc add'
Set-Alias -Name gc -Value 'gvc commit'
Set-Alias -Name gl -Value 'gvc log'
```

---

## Troubleshooting

### "cargo: command not found"

**Solution:**
1. Restart PowerShell
2. Check PATH:
   ```powershell
   $env:Path -split ';' | Select-String cargo
   ```
3. Should show: `C:\Users\YourUsername\.cargo\bin`

### "Permission denied" on hooks

**Solution:**
```powershell
Set-ExecutionPolicy -Scope CurrentUser RemoteSigned
```

### Build errors

**Solution:**
```powershell
# Clean and rebuild
cargo clean
cargo build --release
```

### Rust update needed

```powershell
rustup update
```

---

## Windows-Specific Features

### PowerShell Hooks

GVC supports `.ps1` hooks on Windows:

```powershell
# Create pre-commit hook
@"
Write-Host "Running pre-commit hook"
# Your logic here
exit 0
"@ | Out-File -Encoding UTF8 .gvc\hooks\pre-commit.ps1
```

### Terminal Recommendations

For best experience, use:
- **Windows Terminal** (recommended, supports colors)
- **PowerShell 7+** (newer, better)

Avoid:
- cmd.exe (limited color support)
- PowerShell ISE (outdated)

### Install Windows Terminal

```powershell
# Via winget (Windows 11 / Windows 10 2004+)
winget install Microsoft.WindowsTerminal
```

---

## Module Examples (Windows)

### Install Modules

```powershell
# Install example modules
gvc module install .\examples\rust-linter-module
gvc module install .\examples\commit-convention-module

# Activate in project
cd your-project
gvc module add rust-linter@1.0.0
gvc module add commit-convention@1.0.0
```

### Create Windows Module

```powershell
# Create module
gvc module create my-module --author "Your Name"
cd my-module

# Create PowerShell hook
@"
Write-Host "Running my hook"
# Your logic
exit 0
"@ | Out-File -Encoding UTF8 hooks\pre-commit.ps1

# Update module.toml to reference it
# [hooks]
# pre-commit = "hooks/pre-commit.ps1"

# Install
gvc module install .
```

---

## File Paths on Windows

GVC uses Windows paths correctly:

```
C:\Users\YourName\.gvc\modules\     # Global modules
C:\your\project\.gvc\               # Repository data
```

---

## Performance Tips

### Exclude from Windows Defender

Add `.gvc\objects\` to exclusions for faster performance:

```powershell
# Run as Administrator
Add-MpPreference -ExclusionPath "C:\path\to\project\.gvc\objects"
```

### Use SSD

Store repositories on SSD for better performance.

---

## Common Windows Issues

### Line Endings

GVC handles Windows line endings (CRLF) automatically. For consistency:

```powershell
# Create .gitattributes
@"
* text=auto
*.sh text eol=lf
*.ps1 text eol=crlf
"@ | Out-File -Encoding UTF8 .gitattributes
```

### Long Paths

If you encounter "path too long" errors:

```powershell
# Enable long paths (Windows 10 1607+, as Administrator)
New-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\FileSystem" `
  -Name "LongPathsEnabled" -Value 1 -PropertyType DWORD -Force
```

---

## Uninstall

### Remove GVC

```powershell
# Remove binary
Remove-Item "$env:USERPROFILE\.cargo\bin\gvc.exe"

# Remove modules (optional)
Remove-Item -Recurse "$env:USERPROFILE\.gvc"
```

### Remove Rust

```powershell
rustup self uninstall
```

---

## Next Steps

1. ✅ GVC is installed!
2. 📖 Read [TUTORIAL.md](TUTORIAL.md)
3. 📋 See [CHEATSHEET.md](CHEATSHEET.md)
4. 🔧 Try example modules

---

## Windows-Specific Resources

- [PowerShell Documentation](https://docs.microsoft.com/powershell/)
- [Windows Terminal](https://aka.ms/terminal)
- [Rust on Windows](https://www.rust-lang.org/tools/install)

---

**Enjoy GVC on Windows!** 🚀

