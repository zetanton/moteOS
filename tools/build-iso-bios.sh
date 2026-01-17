#!/bin/bash
# Build script for BIOS boot ISO (x86_64)
# Creates a bootable ISO image for legacy BIOS systems using Multiboot2

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BUILD_DIR="$PROJECT_ROOT/target"
ISO_DIR="$PROJECT_ROOT/iso-bios"
OUTPUT_ISO="$PROJECT_ROOT/moteos-x64-bios.iso"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Building moteOS BIOS boot ISO...${NC}"

# Check for required tools
command -v cargo >/dev/null 2>&1 || { echo -e "${RED}Error: cargo not found${NC}" >&2; exit 1; }
command -v xorriso >/dev/null 2>&1 || { echo -e "${RED}Error: xorriso not found. Install with: apt-get install xorriso (Debian/Ubuntu) or brew install xorriso (macOS)${NC}" >&2; exit 1; }
command -v grub-mkrescue >/dev/null 2>&1 || { echo -e "${YELLOW}Warning: grub-mkrescue not found. Will use xorriso with isolinux${NC}" >&2; }

# Check for BIOS target (typically x86_64-unknown-none or custom)
# For BIOS boot, we typically need a custom target or use a bootloader
echo -e "${YELLOW}Note: BIOS boot requires a Multiboot2-compatible bootloader${NC}"
echo -e "${YELLOW}This script assumes the kernel binary is built for Multiboot2${NC}"

# Build kernel for BIOS (assuming custom target or x86_64-unknown-none)
# For now, we'll check if a BIOS build exists
BIOS_TARGET="x86_64-unknown-none"
if rustup target list --installed | grep -q "$BIOS_TARGET"; then
    echo -e "${GREEN}Building kernel for BIOS (x86_64)...${NC}"
    cd "$PROJECT_ROOT"
    cargo build --release --target "$BIOS_TARGET" || {
        echo -e "${YELLOW}Warning: BIOS target build failed. You may need to set up a custom target${NC}"
    }
fi

# Create ISO directory structure
echo -e "${GREEN}Creating ISO directory structure...${NC}"
rm -rf "$ISO_DIR"
mkdir -p "$ISO_DIR/boot/grub"
mkdir -p "$ISO_DIR/boot/kernel"

# Copy kernel binary (adjust path as needed)
KERNEL_BINARY=""
if [ -f "$BUILD_DIR/$BIOS_TARGET/release/moteos" ]; then
    KERNEL_BINARY="$BUILD_DIR/$BIOS_TARGET/release/moteos"
elif [ -f "$BUILD_DIR/$BIOS_TARGET/release/kernel" ]; then
    KERNEL_BINARY="$BUILD_DIR/$BIOS_TARGET/release/kernel"
else
    echo -e "${YELLOW}Warning: No BIOS kernel binary found. Creating ISO structure anyway.${NC}"
    echo -e "${YELLOW}You may need to build the kernel with a Multiboot2-compatible bootloader${NC}"
fi

if [ -n "$KERNEL_BINARY" ] && [ -f "$KERNEL_BINARY" ]; then
    cp "$KERNEL_BINARY" "$ISO_DIR/boot/kernel/moteos"
fi

# Create GRUB configuration
cat > "$ISO_DIR/boot/grub/grub.cfg" << 'EOF'
set timeout=0
set default=0

menuentry "moteOS" {
    multiboot2 /boot/kernel/moteos
    boot
}
EOF

# Create ISO using grub-mkrescue if available, otherwise use xorriso
if command -v grub-mkrescue >/dev/null 2>&1; then
    echo -e "${GREEN}Creating ISO with grub-mkrescue...${NC}"
    grub-mkrescue -o "$OUTPUT_ISO" "$ISO_DIR" || {
        echo -e "${RED}Error: ISO creation failed${NC}" >&2
        exit 1
    }
else
    echo -e "${GREEN}Creating ISO with xorriso...${NC}"
    echo -e "${YELLOW}Note: For proper BIOS boot, grub-mkrescue is recommended${NC}"
    # This is a simplified ISO creation - may not boot on all BIOS systems
    xorriso -as mkisofs \
        -R \
        -J \
        -b boot/grub/i386-pc/eltorito.img \
        -no-emul-boot \
        -boot-load-size 4 \
        -boot-info-table \
        -o "$OUTPUT_ISO" \
        "$ISO_DIR" || {
        echo -e "${RED}Error: ISO creation failed${NC}" >&2
        echo -e "${YELLOW}Try installing grub-pc-bin and using grub-mkrescue instead${NC}"
        exit 1
    }
fi

echo -e "${GREEN}✓ BIOS boot ISO created: $OUTPUT_ISO${NC}"
echo -e "${GREEN}  Size: $(du -h "$OUTPUT_ISO" | cut -f1)${NC}"
echo -e "${YELLOW}Note: BIOS boot requires proper Multiboot2 bootloader setup${NC}"
