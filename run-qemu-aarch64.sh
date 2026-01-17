#!/bin/bash
# Quick QEMU runner for moteOS (AArch64)

set -e

ROOT_DIR="$(cd "$(dirname "$0")" && pwd)"
ISO_FILE="$ROOT_DIR/moteos-aarch64-uefi.iso"

QEMU_EFI_CODE="/opt/homebrew/share/qemu/edk2-aarch64-code.fd"
QEMU_EFI_VARS="/opt/homebrew/share/qemu/edk2-aarch64-vars.fd"
ALT_EFI_CODE="/usr/share/qemu/edk2-aarch64-code.fd"
ALT_EFI_VARS="/usr/share/qemu/edk2-aarch64-vars.fd"

echo "🚀 Starting moteOS (AArch64) in QEMU..."
echo "ISO: $ISO_FILE"
echo ""

if [ ! -f "$ISO_FILE" ]; then
    echo "❌ Error: ISO file not found: $ISO_FILE"
    echo "Build it with: make iso-aarch64"
    exit 1
fi

# Find ARM UEFI firmware
if [ -f "$QEMU_EFI_CODE" ]; then
    EFI_CODE="$QEMU_EFI_CODE"
    EFI_VARS="$QEMU_EFI_VARS"
    echo "✓ Using ARM UEFI firmware from QEMU installation"
elif [ -f "$ALT_EFI_CODE" ]; then
    EFI_CODE="$ALT_EFI_CODE"
    EFI_VARS="$ALT_EFI_VARS"
    echo "✓ Using ARM UEFI firmware from system location"
else
    EFI_CODE=""
    EFI_VARS=""
    echo "⚠ No ARM UEFI firmware found - boot may fail"
fi

TEMP_VARS=""
if [ -n "$EFI_CODE" ] && [ -f "$EFI_CODE" ]; then
    TEMP_VARS="$(mktemp)"
    if [ -f "$EFI_VARS" ]; then
        cp "$EFI_VARS" "$TEMP_VARS"
    else
        dd if=/dev/zero of="$TEMP_VARS" bs=1M count=64 2>/dev/null || true
    fi
    trap "rm -f $TEMP_VARS" EXIT
fi

# Allow override of QEMU binary, accelerator, CPU, and extra args
QEMU_BIN="${QEMU_BIN:-qemu-system-aarch64}"
ACCEL="${QEMU_ACCEL:-tcg}"
CPU_MODEL="${QEMU_CPU:-cortex-a72}"
EXTRA_ARGS="${QEMU_EXTRA_ARGS:-}"

$QEMU_BIN \
    -machine virt \
    -cpu "$CPU_MODEL" \
    -m 1G \
    -accel "$ACCEL" \
    -drive "file=$ISO_FILE,format=raw,media=cdrom,id=cdrom0,if=none" \
    -device "virtio-blk-device,drive=cdrom0" \
    -netdev "user,id=net0" \
    -device "virtio-net-device,netdev=net0" \
    -serial stdio \
    -display cocoa \
    -no-reboot \
    ${EFI_CODE:+-drive "if=pflash,format=raw,readonly=on,file=$EFI_CODE"} \
    ${TEMP_VARS:+-drive "if=pflash,format=raw,file=$TEMP_VARS"} \
    $EXTRA_ARGS

echo ""
echo "✅ QEMU session ended"
