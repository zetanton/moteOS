#!/bin/bash
# Test build verification script for AArch64 cross-compilation
# Verifies that the build completes successfully and produces expected artifacts

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BUILD_DIR="$PROJECT_ROOT/target/aarch64-unknown-uefi/release"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Testing AArch64 cross-compilation build...${NC}"

# Check for required tools
command -v cargo >/dev/null 2>&1 || { echo -e "${RED}Error: cargo not found${NC}" >&2; exit 1; }

# Check for target
if ! rustup target list --installed | grep -q "aarch64-unknown-uefi"; then
    echo -e "${YELLOW}Installing aarch64-unknown-uefi target...${NC}"
    rustup target add aarch64-unknown-uefi
fi

# Clean previous build
echo -e "${GREEN}Cleaning previous build...${NC}"
cd "$PROJECT_ROOT"
cargo clean --target aarch64-unknown-uefi

# Build
echo -e "${GREEN}Building for aarch64-unknown-uefi...${NC}"
cargo build --release --target aarch64-unknown-uefi

# Verify build artifacts
echo -e "${GREEN}Verifying build artifacts...${NC}"

# Check for EFI binary
EFI_BINARY=""
if [ -f "$BUILD_DIR/moteos.efi" ]; then
    EFI_BINARY="$BUILD_DIR/moteos.efi"
elif [ -f "$BUILD_DIR/boot.efi" ]; then
    EFI_BINARY="$BUILD_DIR/boot.efi"
elif [ -f "$BUILD_DIR/kernel.efi" ]; then
    EFI_BINARY="$BUILD_DIR/kernel.efi"
else
    # Try to find any .efi file
    EFI_BINARY=$(find "$BUILD_DIR" -name "*.efi" -type f | head -n 1)
fi

if [ -z "$EFI_BINARY" ]; then
    echo -e "${RED}✗ Error: No EFI binary found${NC}" >&2
    echo -e "${YELLOW}Checked directory: $BUILD_DIR${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Found EFI binary: $EFI_BINARY${NC}"

# Check file size (should be reasonable)
FILE_SIZE=$(stat -f%z "$EFI_BINARY" 2>/dev/null || stat -c%s "$EFI_BINARY" 2>/dev/null)
if [ "$FILE_SIZE" -lt 10000 ]; then
    echo -e "${YELLOW}⚠ Warning: EFI binary seems very small ($FILE_SIZE bytes)${NC}"
elif [ "$FILE_SIZE" -gt 100000000 ]; then
    echo -e "${YELLOW}⚠ Warning: EFI binary seems very large ($FILE_SIZE bytes)${NC}"
else
    echo -e "${GREEN}✓ EFI binary size: $(numfmt --to=iec-i --suffix=B $FILE_SIZE 2>/dev/null || echo "$FILE_SIZE bytes")${NC}"
fi

# Check file type (should be ELF for ARM64)
if command -v file >/dev/null 2>&1; then
    FILE_TYPE=$(file "$EFI_BINARY")
    if echo "$FILE_TYPE" | grep -q "ARM aarch64"; then
        echo -e "${GREEN}✓ File type: ARM64 ELF${NC}"
    elif echo "$FILE_TYPE" | grep -q "PE32+"; then
        echo -e "${GREEN}✓ File type: PE32+ (UEFI)${NC}"
    else
        echo -e "${YELLOW}⚠ Warning: Unexpected file type: $FILE_TYPE${NC}"
    fi
fi

# Check for linker script
if [ -f "$PROJECT_ROOT/linker-aarch64.ld" ]; then
    echo -e "${GREEN}✓ Linker script found: linker-aarch64.ld${NC}"
else
    echo -e "${RED}✗ Error: Linker script not found${NC}" >&2
    exit 1
fi

# Check for cargo config
if [ -f "$PROJECT_ROOT/.cargo/config.toml" ]; then
    if grep -q "aarch64-unknown-uefi" "$PROJECT_ROOT/.cargo/config.toml"; then
        echo -e "${GREEN}✓ Cargo config found with aarch64 target${NC}"
    else
        echo -e "${YELLOW}⚠ Warning: Cargo config found but aarch64 target not configured${NC}"
    fi
else
    echo -e "${YELLOW}⚠ Warning: .cargo/config.toml not found${NC}"
fi

# Summary
echo -e "${GREEN}${NC}"
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Build verification complete!${NC}"
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}EFI Binary: $EFI_BINARY${NC}"
echo -e "${GREEN}Size: $(numfmt --to=iec-i --suffix=B $FILE_SIZE 2>/dev/null || echo "$FILE_SIZE bytes")${NC}"
echo -e "${GREEN}${NC}"
echo -e "${YELLOW}Next steps:${NC}"
echo -e "${YELLOW}  1. Run: ./tools/build-iso-aarch64.sh${NC}"
echo -e "${YELLOW}  2. Flash ISO to MicroSD card for Raspberry Pi${NC}"
echo -e "${YELLOW}  3. Install UEFI firmware (RPi4_UEFI_Firmware)${NC}"
