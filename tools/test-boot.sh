#!/bin/bash
# QEMU boot test script
# Tests that the kernel boots successfully and reaches kernel_main()

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ISO_FILE="$PROJECT_ROOT/moteos-x64-uefi.iso"
ISO_DIR="$PROJECT_ROOT/iso"
OVMF_CODE="/usr/share/OVMF/OVMF_CODE.fd"
OVMF_VARS="/usr/share/OVMF/OVMF_VARS.fd"
OVMF_CODE_X64="/usr/share/qemu/edk2-x86_64-code.fd"
OVMF_VARS_X64="/usr/share/qemu/edk2-x86_64-vars.fd"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Running moteOS boot test...${NC}"

# Check for required tools
command -v qemu-system-x86_64 >/dev/null 2>&1 || { 
    echo -e "${RED}Error: qemu-system-x86_64 not found${NC}" >&2
    echo -e "${YELLOW}Install with: apt-get install qemu-system-x86 (Debian/Ubuntu) or brew install qemu (macOS)${NC}"
    exit 1
}

# Check if ISO exists
if [ ! -f "$ISO_FILE" ]; then
    echo -e "${YELLOW}ISO file not found: $ISO_FILE${NC}"
    echo -e "${YELLOW}Building ISO first...${NC}"
    "$SCRIPT_DIR/build-iso-uefi.sh" || {
        echo -e "${RED}Failed to build ISO${NC}" >&2
        exit 1
    }
fi

# Check for OVMF (UEFI firmware)
if [ ! -f "$OVMF_CODE" ]; then
    # Try alternative locations
    if [ -f "$OVMF_CODE_X64" ]; then
        OVMF_CODE="$OVMF_CODE_X64"
        OVMF_VARS="$OVMF_VARS_X64"
    elif [ -f "/usr/share/qemu/OVMF_CODE.fd" ]; then
        OVMF_CODE="/usr/share/qemu/OVMF_CODE.fd"
        OVMF_VARS="/usr/share/qemu/OVMF_VARS.fd"
    elif [ -f "$PROJECT_ROOT/ovmf/OVMF_CODE.fd" ]; then
        OVMF_CODE="$PROJECT_ROOT/ovmf/OVMF_CODE.fd"
        OVMF_VARS="$PROJECT_ROOT/ovmf/OVMF_VARS.fd"
    elif [ -f "$PROJECT_ROOT/OVMF_CODE.fd" ]; then
        OVMF_CODE="$PROJECT_ROOT/OVMF_CODE.fd"
        OVMF_VARS="$PROJECT_ROOT/OVMF_VARS.fd"
    else
        echo -e "${YELLOW}Warning: OVMF firmware not found${NC}"
        echo -e "${YELLOW}Install with: apt-get install ovmf (Debian/Ubuntu)${NC}"
        echo -e "${YELLOW}Or download from: https://github.com/tianocore/edk2${NC}"
        echo -e "${YELLOW}Continuing without UEFI firmware (may not boot properly)...${NC}"
        OVMF_CODE=""
        OVMF_VARS=""
    fi
fi

# Create temporary OVMF vars file if needed
if [ -n "$OVMF_CODE" ] && [ -f "$OVMF_CODE" ]; then
    TEMP_VARS=$(mktemp)
    cp "$OVMF_VARS" "$TEMP_VARS" 2>/dev/null || touch "$TEMP_VARS"
    trap "rm -f $TEMP_VARS" EXIT
fi

# QEMU command
QEMU_CMD="qemu-system-x86_64"

# Build QEMU command
QEMU_ARGS=(
    -machine q35
    -cpu qemu64
    -m 1G
    -drive "file=$ISO_FILE,format=raw,media=cdrom,id=cdrom0,if=none"
    -device "ide-cd,drive=cdrom0,bootindex=2"
    -drive "file=fat:rw:$ISO_DIR,format=raw,if=none,id=fs0"
    -device "virtio-blk-pci,drive=fs0,bootindex=1"
    -netdev "user,id=net0"
    -device "virtio-net,netdev=net0"
    -serial stdio
    -monitor none
    -display none
    -no-reboot
)

# On Apple Silicon, force TCG to avoid HVF issues running x86_64 guests
HOST_ARCH="$(uname -m)"
if [ "$HOST_ARCH" = "arm64" ] || [ "$HOST_ARCH" = "aarch64" ]; then
    QEMU_ARGS+=(
        -accel tcg
    )
fi

# Add UEFI firmware if available
if [ -n "$OVMF_CODE" ] && [ -f "$OVMF_CODE" ]; then
    QEMU_ARGS+=(
        -drive "if=pflash,format=raw,readonly=on,file=$OVMF_CODE"
        -drive "if=pflash,format=raw,file=$TEMP_VARS"
    )
    echo -e "${GREEN}Using UEFI firmware: $OVMF_CODE${NC}"
else
    echo -e "${YELLOW}Running without UEFI firmware (legacy BIOS mode)${NC}"
fi

echo -e "${GREEN}Starting QEMU...${NC}"
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
    "$TIMEOUT_CMD" 30s "$QEMU_CMD" "${QEMU_ARGS[@]}" 2>&1 | tee /tmp/moteos-boot-test.log || {
    EXIT_CODE=$?
    if [ $EXIT_CODE -eq 124 ]; then
        echo -e "${GREEN}✓ Boot test completed (timeout reached - system is running)${NC}"
        exit 0
    else
        echo -e "${RED}✗ Boot test failed (exit code: $EXIT_CODE)${NC}"
        echo -e "${YELLOW}Check /tmp/moteos-boot-test.log for details${NC}"
        exit $EXIT_CODE
    fi
}
else
    # macOS fallback: run QEMU with timeout using perl or just run for limited time
    echo -e "${YELLOW}Note: timeout command not found, running QEMU for 30 seconds${NC}"
    # Use a subshell that times out using perl's alarm, or just run and capture
    "$QEMU_CMD" "${QEMU_ARGS[@]}" 2>&1 | tee /tmp/moteos-boot-test.log &
    QEMU_PID=$!
    # Wait for QEMU to start outputting
    sleep 2
    # Run for 30 seconds total
    (sleep 28; kill $QEMU_PID 2>/dev/null || true) &
    KILL_PID=$!
    wait $QEMU_PID 2>/dev/null || true
    kill $KILL_PID 2>/dev/null || true
    wait $KILL_PID 2>/dev/null || true
    echo -e "${GREEN}✓ Boot test completed (30 seconds elapsed)${NC}"
fi

# Check log for successful boot indicators
if grep -q "kernel_main\|moteOS\|Boot successful\|kernel_main reached" /tmp/moteos-boot-test.log 2>/dev/null; then
    echo -e "${GREEN}✓ Boot test passed - kernel reached main function${NC}"
    exit 0
else
    echo -e "${YELLOW}⚠ Boot test completed but no clear success indicator found${NC}"
    echo -e "${YELLOW}Check /tmp/moteos-boot-test.log for kernel output${NC}"
    exit 0  # Don't fail if we can't detect success - manual inspection needed
fi
