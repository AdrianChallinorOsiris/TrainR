#!/bin/bash
# Deployment script for Raspberry Pi
# This script builds and deploys the application to a Raspberry Pi

set -e

# Configuration
TARGET="${1:-arm-unknown-linux-gnueabihf}"
PI_HOST="${2:-pi@raspberrypi.local}"
PI_PATH="${3:-/home/pi/train}"
BUILD_TYPE="${4:-release}"

echo "Train Set Control - Deployment Script"
echo "======================================"
echo "Target: $TARGET"
echo "Raspberry Pi: $PI_HOST"
echo "Deploy path: $PI_PATH"
echo "Build type: $BUILD_TYPE"
echo ""

# Build the project
echo "Building project..."
if [ "$BUILD_TYPE" = "release" ]; then
    cargo build --target "$TARGET" --release
    BINARY="target/$TARGET/release/train"
else
    cargo build --target "$TARGET"
    BINARY="target/$TARGET/debug/train"
fi

if [ ! -f "$BINARY" ]; then
    echo "Error: Binary not found at $BINARY"
    exit 1
fi

# Copy to Raspberry Pi
echo "Deploying to Raspberry Pi..."
scp "$BINARY" "$PI_HOST:$PI_PATH"

# Make executable
echo "Setting permissions..."
ssh "$PI_HOST" "chmod +x $PI_PATH"

echo ""
echo "Deployment complete!"
echo "To run on Raspberry Pi:"
echo "  ssh $PI_HOST"
echo "  $PI_PATH"
