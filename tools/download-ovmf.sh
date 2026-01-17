#!/bin/bash
# Download OVMF firmware for macOS/QEMU
# Based on: https://github.com/tianocore/tianocore.github.io/wiki/OVMF

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
OVMF_DIR="$PROJECT_ROOT/ovmf"

mkdir -p "$OVMF_DIR"
cd "$OVMF_DIR"

echo "Downloading OVMF firmware from kraxel.org..."
echo ""
echo "Note: We'll try to download pre-built firmware files directly."
echo "If this fails, you can:"
echo "  1. Install OVMF via Homebrew: brew install --cask qemu-utils"
echo "  2. Or download from: https://www.kraxel.org/repos/jenkins/edk2/"
echo "  3. Or build from source: https://github.com/tianocore/edk2"
echo ""

# Try to find a direct download link
echo "Checking for available OVMF builds..."

# For macOS, we can try using Homebrew's qemu package or download manually
if command -v brew >/dev/null 2>&1; then
    echo "Homebrew found. Checking if qemu includes OVMF..."
    QEMU_SHARE=$(brew --prefix qemu 2>/dev/null || echo "")
    if [ -n "$QEMU_SHARE" ]; then
        if [ -d "$QEMU_SHARE/share/qemu" ]; then
            echo "✓ Found QEMU share directory: $QEMU_SHARE/share/qemu"
            find "$QEMU_SHARE/share/qemu" -name "*efi*.fd" -o -name "*OVMF*.fd" | while read fw; do
                echo "  Found: $fw"
            done
        fi
    fi
fi

echo ""
echo "For manual download, visit:"
echo "  https://www.kraxel.org/repos/jenkins/edk2/"
echo ""
echo "Download the latest edk2.git-ovmf-x64 RPM file and extract:"
echo "  rpm2cpio file.rpm | cpio -idmv"
echo ""
echo "Or use Homebrew to install OVMF if available."
