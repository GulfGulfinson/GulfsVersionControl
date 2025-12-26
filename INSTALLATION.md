# Installation Guide - GulfsVersionControl

## Prerequisites

### Install Rust

GVC is written in Rust, so you need to install the Rust toolchain first.

#### Windows

1. Download and run [rustup-init.exe](https://rustup.rs/)
2. Follow the on-screen instructions
3. Restart your terminal after installation
4. Verify installation:
   ```powershell
   rustc --version
   cargo --version
   ```

#### Linux / macOS

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustc --version
cargo --version
```

## Building GVC

Once Rust is installed:

```bash
# Clone or navigate to the project directory
cd GulfsControlSystem

# Build the project
cargo build --release

# The binary will be at: target/release/gvc (or gvc.exe on Windows)
```

## Installation

### Option 1: Add to PATH

Copy the binary to a directory in your PATH:

**Windows:**
```powershell
copy target\release\gvc.exe C:\Windows\System32\
```

**Linux/macOS:**
```bash
sudo cp target/release/gvc /usr/local/bin/
```

### Option 2: Use cargo install

```bash
cargo install --path gvc-cli
```

This installs `gvc` to `~/.cargo/bin/` (which should be in your PATH).

## Verify Installation

```bash
gvc --version
gvc --help
```

## Quick Start

```bash
# Initialize a new repository
gvc init

# Create a test file
echo "Hello GVC" > test.txt

# Stage the file
gvc add test.txt

# Commit
gvc commit -m "Initial commit"

# View history
gvc log
```

## Development Build

For development and testing:

```bash
# Build in debug mode (faster compilation)
cargo build

# Run without installing
cargo run --bin gvc -- init

# Run tests
cargo test

# Run tests with output
cargo test -- --nocapture
```

## Troubleshooting

### "cargo: command not found"

- Ensure Rust is installed: `rustc --version`
- Restart your terminal
- Check PATH includes `~/.cargo/bin`

### Build Errors

- Update Rust: `rustup update`
- Clean build: `cargo clean && cargo build`

### Permission Denied (Linux/macOS)

```bash
chmod +x target/release/gvc
```

## Next Steps

See [README.md](README.md) for usage documentation and [ARCHITECTURE.md](ARCHITECTURE.md) for technical details.

