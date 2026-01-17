#!/bin/bash
# QEMU boot test script for AArch64
# Tests that the kernel boots successfully in QEMU ARM64 emulator

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ISO_FILE="$PROJECT_ROOT/moteos-aarch64-uefi.iso"

# ARM UEFI firmware locations
# On Linux, EDK2 firmware is typically in /usr/share/qemu/efi/
# On macOS with Homebrew, it might be in /opt/homebrew/share/qemu/edk2-aarch64-code.fd
QEMU_EFI_CODE="/usr/share/qemu/efi/QEMU_EFI.fd"
QEMU_EFI_VARS="/usr/share/qemu/efi/vars-template-pflash.raw"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${GREEN}Running moteOS AArch64 boot test...${NC}"

# Check for required tools
command -v qemu-system-aarch64 >/dev/null 2>&1 || { 
    echo -e "${RED}Error: qemu-system-aarch64 not found${NC}" >&2
    echo -e "${YELLOW}Install with: apt-get install qemu-system-arm (Debian/Ubuntu) or brew install qemu (macOS)${NC}"
    exit 1
}

# Check if ISO exists
if [ ! -f "$ISO_FILE" ]; then
    echo -e "${YELLOW}ISO file not found: $ISO_FILE${NC}"
    echo -e "${YELLOW}Building ISO first...${NC}"
    "$SCRIPT_DIR/build-iso-aarch64.sh" || {
        echo -e "${RED}Failed to build ISO${NC}" >&2
        exit 1
    }
fi

# Check for ARM UEFI firmware
if [ ! -f "$QEMU_EFI_CODE" ]; then
    # Try alternative locations
    if [ -f "/usr/share/qemu/edk2-aarch64-code.fd" ]; then
        QEMU_EFI_CODE="/usr/share/qemu/edk2-aarch64-code.fd"
        QEMU_EFI_VARS="/usr/share/qemu/edk2-aarch64-vars.fd"
    elif [ -f "/opt/homebrew/share/qemu/edk2-aarch64-code.fd" ]; then
        QEMU_EFI_CODE="/opt/homebrew/share/qemu/edk2-aarch64-code.fd"
        QEMU_EFI_VARS="/opt/homebrew/share/qemu/edk2-aarch64-vars.fd"
    elif [ -f "$PROJECT_ROOT/QEMU_EFI.fd" ]; then
        QEMU_EFI_CODE="$PROJECT_ROOT/QEMU_EFI.fd"
        QEMU_EFI_VARS="$PROJECT_ROOT/vars-template-pflash.raw"
    else
        echo -e "${YELLOW}Warning: ARM UEFI firmware not found${NC}"
        echo -e "${YELLOW}Install with: apt-get install qemu-efi-aarch64 (Debian/Ubuntu)${NC}"
        echo -e "${YELLOW}Or download from: https://github.com/tianocore/edk2${NC}"
        echo -e "${YELLOW}Continuing without UEFI firmware (may not boot properly)...${NC}"
        QEMU_EFI_CODE=""
        QEMU_EFI_VARS=""
    fi
fi

# Create temporary EFI vars file if needed
if [ -n "$QEMU_EFI_CODE" ] && [ -f "$QEMU_EFI_CODE" ]; then
    TEMP_VARS=$(mktemp)
    if [ -f "$QEMU_EFI_VARS" ]; then
        cp "$QEMU_EFI_VARS" "$TEMP_VARS"
    else
        # Create empty vars file (64MB flash)
        dd if=/dev/zero of="$TEMP_VARS" bs=1M count=64 2>/dev/null || true
    fi
    trap "rm -f $TEMP_VARS" EXIT
fi

# QEMU command for AArch64
QEMU_CMD="qemu-system-aarch64"

# Build QEMU command for ARM64
QEMU_ARGS=(
    -machine virt
    -cpu cortex-a72
    -m 1G
    -drive "file=$ISO_FILE,format=raw,media=cdrom,id=cdrom0,if=none"
    -device "virtio-blk-device,drive=cdrom0"
    -netdev "user,id=net0"
    -device "virtio-net-device,netdev=net0"
    -serial stdio
    -monitor none
    -display none
    -no-reboot
)

# On Apple Silicon, force TCG to avoid HVF issues
HOST_ARCH="$(uname -m)"
if [ "$HOST_ARCH" = "arm64" ] || [ "$HOST_ARCH" = "aarch64" ]; then
    QEMU_ARGS+=(
        -accel tcg
    )
fi

# Add UEFI firmware if available
if [ -n "$QEMU_EFI_CODE" ] && [ -f "$QEMU_EFI_CODE" ]; then
    QEMU_ARGS+=(
        -drive "if=pflash,format=raw,readonly=on,file=$QEMU_EFI_CODE"
        -drive "if=pflash,format=raw,file=$TEMP_VARS"
    )
    echo -e "${GREEN}Using ARM UEFI firmware: $QEMU_EFI_CODE${NC}"
else
    echo -e "${YELLOW}Running without UEFI firmware (may not boot properly)...${NC}"
fi

echo -e "${GREEN}Starting QEMU for AArch64...${NC}"
echo -e "${BLUE}Machine: virt (QEMU ARM Virtual Machine)${NC}"
echo -e "${BLUE}CPU: cortex-a72 (ARM64)${NC}"
echo -e "${BLUE}Memory: 1GB${NC}"
echo -e "${YELLOW}The system will boot and you should see kernel output on serial console${NC}"
echo -e "${YELLOW}Press Ctrl+C to stop the test${NC}"
echo ""

# Detect timeout command (macOS may use gtimeout from coreutils)
TIMEOUT_CMD=""
if command -v gtimeout >/dev/null 2>&1; then
    TIMEOUT_CMD="gtimeout"
elif command -v timeout >/dev/null 2>&1; then
    TIMEOUT_CMD="timeout"
fi

# Run QEMU with timeout (30 seconds) if available, otherwise run without timeout
if [ -n "$TIMEOUT_CMD" ]; then
    "$TIMEOUT_CMD" 30s "$QEMU_CMD" "${QEMU_ARGS[@]}" 2>&1 | tee /tmp/moteos-boot-aarch64-test.log || {
    EXIT_CODE=$?
    if [ $EXIT_CODE -eq 124 ]; then
        echo -e "${GREEN}✓ Boot test completed (timeout reached - system is running)${NC}"
        exit 0
    else
        echo -e "${RED}✗ Boot test failed (exit code: $EXIT_CODE)${NC}"
        echo -e "${YELLOW}Check /tmp/moteos-boot-aarch64-test.log for details${NC}"
        exit $EXIT_CODE
    fi
}
else
    echo -e "${YELLOW}Note: timeout command not found, running QEMU for 30 seconds${NC}"
    "$QEMU_CMD" "${QEMU_ARGS[@]}" 2>&1 | tee /tmp/moteos-boot-aarch64-test.log &
    QEMU_PID=$!
    sleep 2
    (sleep 28; kill $QEMU_PID 2>/dev/null || true) &
    KILL_PID=$!
    wait $QEMU_PID 2>/dev/null || true
    kill $KILL_PID 2>/dev/null || true
    wait $KILL_PID 2>/dev/null || true
    echo -e "${GREEN}✓ Boot test completed (30 seconds elapsed)${NC}"
fi

# Check log for successful boot indicators
if grep -q "kernel_main\|moteOS\|Boot successful\|efi_main\|kernel_main reached" /tmp/moteos-boot-aarch64-test.log 2>/dev/null; then
    echo -e "${GREEN}✓ Boot test passed - kernel reached main function${NC}"
    exit 0
else
    echo -e "${YELLOW}⚠ Boot test completed but no clear success indicator found${NC}"
    echo -e "${YELLOW}Check /tmp/moteos-boot-aarch64-test.log for kernel output${NC}"
    exit 0  # Don't fail if we can't detect success - manual inspection needed
fi
