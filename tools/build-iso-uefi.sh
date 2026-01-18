#!/bin/bash
# Build script for UEFI boot ISO (x86_64)
# Creates a bootable ISO image for UEFI systems

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BUILD_DIR="$PROJECT_ROOT/target"
ISO_DIR="$PROJECT_ROOT/iso"
OUTPUT_ISO="$PROJECT_ROOT/moteos-x64-uefi.iso"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Building moteOS UEFI boot ISO...${NC}"

# Prefer rustup's cargo over Homebrew cargo (rustup cargo has target support)
if [ -d "$HOME/.cargo/bin" ]; then
    export PATH="$HOME/.cargo/bin:$PATH"
fi

# Check for required tools
command -v cargo >/dev/null 2>&1 || { echo -e "${RED}Error: cargo not found${NC}" >&2; exit 1; }
command -v rustup >/dev/null 2>&1 || { echo -e "${RED}Error: rustup not found. Install rustup to manage Rust toolchains and targets.${NC}" >&2; exit 1; }
command -v xorriso >/dev/null 2>&1 || { echo -e "${RED}Error: xorriso not found. Install with: apt-get install xorriso (Debian/Ubuntu) or brew install xorriso (macOS)${NC}" >&2; exit 1; }

# Check for UEFI target
if ! rustup target list --installed | grep -q "x86_64-unknown-uefi"; then
    echo -e "${YELLOW}Installing x86_64-unknown-uefi target...${NC}"
    rustup target add x86_64-unknown-uefi
fi

# Build kernel for UEFI (only boot crate to avoid network dependencies)
echo -e "${GREEN}Building kernel for UEFI (x86_64)...${NC}"
cd "$PROJECT_ROOT"
cargo build --release --target x86_64-unknown-uefi -p boot

# Find EFI binary (could be moteos.efi, boot.efi, or kernel.efi)
EFI_BINARY=""
if [ -f "$BUILD_DIR/x86_64-unknown-uefi/release/moteos.efi" ]; then
    EFI_BINARY="$BUILD_DIR/x86_64-unknown-uefi/release/moteos.efi"
elif [ -f "$BUILD_DIR/x86_64-unknown-uefi/release/boot.efi" ]; then
    EFI_BINARY="$BUILD_DIR/x86_64-unknown-uefi/release/boot.efi"
elif [ -f "$BUILD_DIR/x86_64-unknown-uefi/release/kernel.efi" ]; then
    EFI_BINARY="$BUILD_DIR/x86_64-unknown-uefi/release/kernel.efi"
else
    # Try to find any .efi file
    EFI_BINARY=$(find "$BUILD_DIR/x86_64-unknown-uefi/release" -name "*.efi" -type f | head -n 1)
    if [ -z "$EFI_BINARY" ]; then
        echo -e "${RED}Error: Build failed - no .efi file found${NC}" >&2
        echo -e "${YELLOW}Checked for: moteos.efi, boot.efi, kernel.efi${NC}"
        echo -e "${YELLOW}Please check target/x86_64-unknown-uefi/release/ for .efi files${NC}"
        echo -e "${YELLOW}Note: You may need to add a [[bin]] target to boot/Cargo.toml${NC}"
        exit 1
    fi
    echo -e "${YELLOW}Found EFI binary: $EFI_BINARY${NC}"
fi

# Create ISO directory structure
echo -e "${GREEN}Creating ISO directory structure...${NC}"
rm -rf "$ISO_DIR"
mkdir -p "$ISO_DIR/EFI/BOOT"

# Copy EFI binary
echo -e "${GREEN}Copying EFI binary...${NC}"
cp "$EFI_BINARY" "$ISO_DIR/EFI/BOOT/BOOTX64.EFI"

# Add startup.nsh to auto-launch the EFI binary in UEFI shell
cat > "$ISO_DIR/startup.nsh" << 'EOF'
echo -off
map -r
fs0:
\EFI\BOOT\BOOTX64.EFI
fs1:
\EFI\BOOT\BOOTX64.EFI
fs2:
\EFI\BOOT\BOOTX64.EFI
EOF

# Create ISO
echo -e "${GREEN}Creating ISO image...${NC}"
xorriso -as mkisofs \
    -R \
    -J \
    -e EFI/BOOT/BOOTX64.EFI \
    -no-emul-boot \
    -boot-load-size 4 \
    -boot-info-table \
    -eltorito-platform efi \
    -isohybrid-gpt-basdat \
    -o "$OUTPUT_ISO" \
    "$ISO_DIR" || {
    echo -e "${RED}Error: ISO creation failed${NC}" >&2
    exit 1
}

echo -e "${GREEN}✓ UEFI boot ISO created: $OUTPUT_ISO${NC}"
echo -e "${GREEN}  Size: $(du -h "$OUTPUT_ISO" | cut -f1)${NC}"
