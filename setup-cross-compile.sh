#!/bin/bash
# Setup script for cross-compilation to Raspberry Pi
# This script helps set up the cross-compilation environment

set -e

echo "Train Set Control - Cross-Compilation Setup"
echo "============================================"
echo ""

# Detect OS
if [[ "$OSTYPE" == "darwin"* ]]; then
    OS="macos"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    OS="linux"
else
    echo "Unsupported OS: $OSTYPE"
    exit 1
fi

echo "Detected OS: $OS"
echo ""

# Ask which target
echo "Which Raspberry Pi target do you want to set up?"
echo "1) arm-unknown-linux-gnueabihf (32-bit - Pi 1, 2, 3, Zero) [Recommended]"
echo "2) aarch64-unknown-linux-gnu (64-bit - Pi 4, 5)"
read -p "Enter choice [1]: " choice
choice=${choice:-1}

if [ "$choice" == "1" ]; then
    TARGET="arm-unknown-linux-gnueabihf"
    if [ "$OS" == "macos" ]; then
        COMPILER="arm-linux-gnueabihf-binutils"
        COMPILER_CMD="arm-linux-gnueabihf-gcc"
    else
        COMPILER="gcc-arm-linux-gnueabihf"
        COMPILER_CMD="arm-linux-gnueabihf-gcc"
    fi
elif [ "$choice" == "2" ]; then
    TARGET="aarch64-unknown-linux-gnu"
    if [ "$OS" == "macos" ]; then
        COMPILER="aarch64-elf-gcc"
        COMPILER_CMD="aarch64-linux-gnu-gcc"
    else
        COMPILER="gcc-aarch64-linux-gnu"
        COMPILER_CMD="aarch64-linux-gnu-gcc"
    fi
else
    echo "Invalid choice"
    exit 1
fi

echo ""
echo "Setting up for target: $TARGET"
echo ""

# Step 1: Install Rust target
echo "Step 1: Installing Rust target..."
if rustup target list --installed | grep -q "$TARGET"; then
    echo "  ✓ Rust target $TARGET is already installed"
else
    echo "  Installing Rust target: $TARGET"
    rustup target add "$TARGET"
    echo "  ✓ Rust target installed"
fi
echo ""

# Step 2: Check for cross-compiler
echo "Step 2: Checking for cross-compiler..."
if command -v "$COMPILER_CMD" &> /dev/null; then
    echo "  ✓ Cross-compiler found: $($COMPILER_CMD --version | head -1)"
else
    echo "  ✗ Cross-compiler not found"
    echo ""
    echo "  Please install the cross-compiler:"
    if [ "$OS" == "macos" ]; then
        echo "    brew install $COMPILER"
    else
        echo "    sudo apt-get install $COMPILER"
    fi
    echo ""
    read -p "Press Enter after installing the cross-compiler, or Ctrl+C to cancel..."
    
    # Check again
    if command -v "$COMPILER_CMD" &> /dev/null; then
        echo "  ✓ Cross-compiler found: $($COMPILER_CMD --version | head -1)"
    else
        echo "  ✗ Cross-compiler still not found. Please install it manually."
        exit 1
    fi
fi
echo ""

# Step 3: Verify .cargo/config.toml
echo "Step 3: Verifying .cargo/config.toml..."
if [ -f ".cargo/config.toml" ]; then
    if grep -q "$TARGET" .cargo/config.toml; then
        echo "  ✓ Configuration file exists and includes $TARGET"
    else
        echo "  ⚠ Configuration file exists but may need updating"
    fi
else
    echo "  ⚠ Configuration file not found (should be created automatically)"
fi
echo ""

# Step 4: Test build
echo "Step 4: Testing cross-compilation..."
echo "  Attempting a test build..."
if cargo build --target "$TARGET" 2>&1 | tail -5; then
    echo "  ✓ Cross-compilation test successful!"
    echo ""
    echo "Setup complete! You can now build with:"
    echo "  cargo build --target $TARGET --release"
    echo ""
    echo "Or use the build script:"
    echo "  ./build-pi.sh $TARGET"
else
    echo "  ✗ Cross-compilation test failed"
    echo ""
    echo "Please check the error messages above and ensure:"
    echo "  1. The Rust target is installed: rustup target add $TARGET"
    echo "  2. The cross-compiler is installed and in PATH"
    echo "  3. The .cargo/config.toml has the correct linker configuration"
    exit 1
fi
