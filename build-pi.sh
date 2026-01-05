#!/bin/bash
# Build script for Raspberry Pi cross-compilation
# This script builds the project for Raspberry Pi targets

set -e

echo "Train Set Control - Raspberry Pi Build Script"
echo "=============================================="

# Detect which Raspberry Pi target to use
# Default to 32-bit (arm-unknown-linux-gnueabihf) for compatibility
TARGET="${1:-arm-unknown-linux-gnueabihf}"

echo "Target: $TARGET"
echo ""

# Check if target is installed
if ! rustup target list --installed | grep -q "$TARGET"; then
    echo "Installing Rust target: $TARGET"
    rustup target add "$TARGET"
fi

# Build for the target
echo "Building for $TARGET..."
cargo build --target "$TARGET" --release

if [ $? -eq 0 ]; then
    echo ""
    echo "Build successful!"
    echo "Binary location: target/$TARGET/release/train"
    echo ""
    echo "To deploy to Raspberry Pi:"
    echo "  scp target/$TARGET/release/train pi@raspberrypi.local:/home/pi/"
    echo ""
    echo "Then on the Raspberry Pi:"
    echo "  chmod +x train"
    echo "  ./train"
else
    echo ""
    echo "Build failed!"
    exit 1
fi
