#!/bin/bash
# Build script for AArch64 UEFI boot ISO (Raspberry Pi)
# Creates a bootable ISO image for ARM64 UEFI systems

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BUILD_DIR="$PROJECT_ROOT/target"
ISO_DIR="$PROJECT_ROOT/iso-aarch64"
OUTPUT_ISO="$PROJECT_ROOT/moteos-aarch64-uefi.iso"
RPI_DIR="$ISO_DIR/rpi"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Building moteOS AArch64 UEFI boot ISO for Raspberry Pi...${NC}"

# Check for required tools
command -v cargo >/dev/null 2>&1 || { echo -e "${RED}Error: cargo not found${NC}" >&2; exit 1; }
command -v xorriso >/dev/null 2>&1 || { echo -e "${RED}Error: xorriso not found. Install with: apt-get install xorriso (Debian/Ubuntu) or brew install xorriso (macOS)${NC}" >&2; exit 1; }

# Check for cross-compilation toolchain
if ! command -v aarch64-linux-gnu-gcc >/dev/null 2>&1; then
    echo -e "${YELLOW}Warning: aarch64-linux-gnu-gcc not found${NC}"
    echo -e "${YELLOW}Install with: sudo apt-get install gcc-aarch64-linux-gnu (Debian/Ubuntu)${NC}"
    echo -e "${YELLOW}or: brew install aarch64-elf-gcc (macOS)${NC}"
    echo -e "${YELLOW}Continuing anyway - Rust may use its own linker${NC}"
fi

# Check for UEFI target
if ! rustup target list --installed | grep -q "aarch64-unknown-uefi"; then
    echo -e "${YELLOW}Installing aarch64-unknown-uefi target...${NC}"
    rustup target add aarch64-unknown-uefi
fi

# Build bootloader for AArch64 UEFI (minimal kernel linked)
echo -e "${GREEN}Building bootloader for UEFI (aarch64)...${NC}"
cd "$PROJECT_ROOT"
cargo build --release --target aarch64-unknown-uefi -p boot

# Find EFI binary (could be moteos.efi, boot.efi, or kernel.efi)
EFI_BINARY=""
if [ -f "$BUILD_DIR/aarch64-unknown-uefi/release/moteos.efi" ]; then
    EFI_BINARY="$BUILD_DIR/aarch64-unknown-uefi/release/moteos.efi"
elif [ -f "$BUILD_DIR/aarch64-unknown-uefi/release/boot.efi" ]; then
    EFI_BINARY="$BUILD_DIR/aarch64-unknown-uefi/release/boot.efi"
elif [ -f "$BUILD_DIR/aarch64-unknown-uefi/release/kernel.efi" ]; then
    EFI_BINARY="$BUILD_DIR/aarch64-unknown-uefi/release/kernel.efi"
else
    # Try to find any .efi file
    EFI_BINARY=$(find "$BUILD_DIR/aarch64-unknown-uefi/release" -name "*.efi" -type f | head -n 1)
    if [ -z "$EFI_BINARY" ]; then
        echo -e "${RED}Error: Build failed - no .efi file found${NC}" >&2
        echo -e "${YELLOW}Checked for: moteos.efi, boot.efi, kernel.efi${NC}"
        echo -e "${YELLOW}Please check target/aarch64-unknown-uefi/release/ for .efi files${NC}"
        echo -e "${YELLOW}Note: You may need to add a [[bin]] target to boot/Cargo.toml${NC}"
        exit 1
    fi
    echo -e "${YELLOW}Found EFI binary: $EFI_BINARY${NC}"
fi

# Create ISO directory structure
echo -e "${GREEN}Creating ISO directory structure...${NC}"
rm -rf "$ISO_DIR"
mkdir -p "$ISO_DIR/EFI/BOOT"
mkdir -p "$RPI_DIR"

# Copy EFI binary
echo -e "${GREEN}Copying EFI binary...${NC}"
cp "$EFI_BINARY" "$ISO_DIR/EFI/BOOT/BOOTAA64.EFI"

# Add startup.nsh to auto-launch the EFI binary in UEFI shell
cat > "$ISO_DIR/startup.nsh" << 'EOF'
echo -off
map -r
fs0:
\EFI\BOOT\BOOTAA64.EFI
fs1:
\EFI\BOOT\BOOTAA64.EFI
fs2:
\EFI\BOOT\BOOTAA64.EFI
EOF

# Create Raspberry Pi specific configuration
echo -e "${GREEN}Creating Raspberry Pi configuration...${NC}"
cat > "$RPI_DIR/config.txt" << 'EOF'
# moteOS Raspberry Pi Configuration
# This file configures the Raspberry Pi firmware for UEFI boot

# Enable UEFI boot
enable_uart=1
arm_64bit=1

# GPU memory split (minimum for UEFI)
gpu_mem=64

# Disable overscan
disable_overscan=1

# HDMI settings
hdmi_force_hotplug=1
hdmi_group=2
hdmi_mode=87
hdmi_cvt=1920 1080 60 6 0 0 0

# Boot delay (0 = no delay)
boot_delay=0

# UEFI firmware
# Note: Raspberry Pi 4 requires UEFI firmware (e.g., from rpi-uefi)
# Place UEFI firmware files in the boot partition
EOF

# Create README for Raspberry Pi
cat > "$RPI_DIR/README.md" << 'EOF'
# moteOS for Raspberry Pi

## Requirements

- Raspberry Pi 4 or Raspberry Pi 400 (ARM64 support)
- UEFI firmware (e.g., from https://github.com/pftf/RPi4)
- MicroSD card (8GB minimum, 16GB recommended)
- USB keyboard (for input)

## Installation

1. Flash the ISO to a MicroSD card:
   ```bash
   sudo dd if=moteos-aarch64-uefi.iso of=/dev/sdX bs=4M status=progress
   ```

2. Copy UEFI firmware files to the boot partition:
   - Download RPi4_UEFI_Firmware_v1.XX.zip from the RPi4 UEFI project
   - Extract and copy `RPI_EFI.fd` to the boot partition as `EFI/BOOT/startup.nsh`

3. Insert the MicroSD card into your Raspberry Pi

4. Power on the Raspberry Pi

5. The system should boot into moteOS

## Troubleshooting

- If the system doesn't boot, ensure UEFI firmware is properly installed
- Check that the MicroSD card is properly formatted
- Verify that `BOOTAA64.EFI` is in `EFI/BOOT/` directory
- For serial console debugging, connect to UART pins (GPIO 14/15)

## Notes

- Raspberry Pi 3 is not supported (32-bit only)
- Raspberry Pi 4 Model B is the primary target
- USB networking may require additional driver support
EOF

# Create ISO
echo -e "${GREEN}Creating ISO image...${NC}"
xorriso -as mkisofs \
    -R \
    -J \
    -e EFI/BOOT/BOOTAA64.EFI \
    -no-emul-boot \
    -boot-load-size 4 \
    -boot-info-table \
    -o "$OUTPUT_ISO" \
    "$ISO_DIR" || {
    echo -e "${RED}Error: ISO creation failed${NC}" >&2
    exit 1
}

echo -e "${GREEN}✓ AArch64 UEFI boot ISO created: $OUTPUT_ISO${NC}"
echo -e "${GREEN}  Size: $(du -h "$OUTPUT_ISO" | cut -f1)${NC}"
echo -e "${YELLOW}Note: For Raspberry Pi, you'll need to:${NC}"
echo -e "${YELLOW}  1. Flash the ISO to a MicroSD card${NC}"
echo -e "${YELLOW}  2. Install UEFI firmware (RPi4_UEFI_Firmware)${NC}"
echo -e "${YELLOW}  3. Ensure BOOTAA64.EFI is in EFI/BOOT/ directory${NC}"
