# Cross-Compilation Setup for Raspberry Pi

This guide will help you set up cross-compilation from macOS to Raspberry Pi.

## Prerequisites

### 1. Install Rust Targets

Add the appropriate Rust target for your Raspberry Pi:

**For Raspberry Pi 1, 2, 3, Zero (32-bit):**
```bash
rustup target add arm-unknown-linux-gnueabihf
```

**For Raspberry Pi 4, 5 (64-bit):**
```bash
rustup target add aarch64-unknown-linux-gnu
```

### 2. Install Cross-Compiler Toolchain

#### On macOS (using Homebrew):

**For 32-bit Raspberry Pi:**
```bash
brew install arm-linux-gnueabihf-binutils
```

**For 64-bit Raspberry Pi:**
```bash
brew install aarch64-elf-gcc
```

**Alternative (if the above don't work):**
```bash
# Install via osx-cross/arm
brew tap osx-cross/arm
brew install arm-linux-gnueabihf-binutils

# Or use crosstool-ng for more control
```

#### On Linux (Ubuntu/Debian):

**For 32-bit Raspberry Pi:**
```bash
sudo apt-get update
sudo apt-get install gcc-arm-linux-gnueabihf
```

**For 64-bit Raspberry Pi:**
```bash
sudo apt-get update
sudo apt-get install gcc-aarch64-linux-gnu
```

### 3. Verify Installation

Check that the cross-compiler is available:
```bash
# For 32-bit
arm-linux-gnueabihf-gcc --version

# For 64-bit
aarch64-linux-gnu-gcc --version
```

## Building

### Using the Build Script

The easiest way is to use the provided build script:

```bash
# For 32-bit Raspberry Pi (default)
./build-pi.sh

# For 64-bit Raspberry Pi
./build-pi.sh aarch64-unknown-linux-gnu
```

### Manual Build

```bash
# For 32-bit Raspberry Pi
cargo build --target arm-unknown-linux-gnueabihf --release

# For 64-bit Raspberry Pi
cargo build --target aarch64-unknown-linux-gnu --release
```

The compiled binary will be at:
- `target/arm-unknown-linux-gnueabihf/release/train` (32-bit)
- `target/aarch64-unknown-linux-gnu/release/train` (64-bit)

## Deployment

### Using the Deployment Script

```bash
./deploy.sh arm-unknown-linux-gnueabihf
```

### Manual Deployment

1. Copy the binary to your Raspberry Pi:
```bash
scp target/arm-unknown-linux-gnueabihf/release/train pi@raspberrypi.local:/home/pi/
```

2. SSH into your Raspberry Pi and make it executable:
```bash
ssh pi@raspberrypi.local
chmod +x train
./train
```

## Troubleshooting

### Linker Not Found

If you get errors about the linker not being found:

1. Verify the cross-compiler is installed and in your PATH
2. Check `.cargo/config.toml` has the correct linker path
3. You may need to specify the full path to the linker in `.cargo/config.toml`:

```toml
[target.arm-unknown-linux-gnueabihf]
linker = "/opt/homebrew/bin/arm-linux-gnueabihf-gcc"
```

### Missing System Libraries

Some dependencies may require system libraries. If you encounter linking errors:

1. You may need to install additional libraries on the Raspberry Pi
2. Or use a Docker container with the Raspberry Pi sysroot for linking

### Alternative: Use Docker

If cross-compilation is problematic, you can use Docker with a Raspberry Pi base image:

```bash
docker run --rm -v $(pwd):/work -w /work rust:latest cargo build --target arm-unknown-linux-gnueabihf --release
```

## Which Target Should I Use?

- **arm-unknown-linux-gnueabihf**: Raspberry Pi 1, 2, 3, Zero, Zero W (32-bit)
- **aarch64-unknown-linux-gnu**: Raspberry Pi 4, 5 (64-bit)

If unsure, use `arm-unknown-linux-gnueabihf` as it's compatible with all Raspberry Pi models.
