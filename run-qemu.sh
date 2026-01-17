#!/bin/bash
# Quick QEMU runner for moteOS

ROOT_DIR="$(cd "$(dirname "$0")" && pwd)"
ISO_FILE="$ROOT_DIR/moteos-x64-uefi.iso"
ISO_DIR="$ROOT_DIR/iso"
OVMF_CODE="$(cd "$(dirname "$0")" && pwd)/ovmf/OVMF_CODE.fd"
OVMF_VARS="$(cd "$(dirname "$0")" && pwd)/ovmf/OVMF_VARS.fd"
QEMU_OVMF_CODE="/opt/homebrew/opt/qemu/share/qemu/edk2-x86_64-code.fd"
QEMU_OVMF_VARS="/opt/homebrew/opt/qemu/share/qemu/edk2-x86_64-vars.fd"
QEMU_OVMF_CODE_I386="/opt/homebrew/opt/qemu/share/qemu/edk2-i386-code.fd"
QEMU_OVMF_VARS_I386="/opt/homebrew/opt/qemu/share/qemu/edk2-i386-vars.fd"

echo "🚀 Starting moteOS in QEMU..."
echo "ISO: $ISO_FILE"
echo ""

# Check if ISO exists
if [ ! -f "$ISO_FILE" ]; then
    echo "❌ Error: ISO file not found: $ISO_FILE"
    echo "Build it with: make iso-uefi"
    exit 1
fi

# Find OVMF firmware (prefer QEMU's x86_64 firmware)
if [ -f "$QEMU_OVMF_CODE" ]; then
    OVMF_CODE_FILE="$QEMU_OVMF_CODE"
    OVMF_VARS_FILE="$QEMU_OVMF_VARS"
    echo "✓ Using x86_64 OVMF firmware from QEMU installation"
elif [ -f "$OVMF_CODE" ]; then
    OVMF_CODE_FILE="$OVMF_CODE"
    OVMF_VARS_FILE="$OVMF_VARS"
    echo "✓ Using OVMF firmware from ovmf/ directory"
elif [ -f "$QEMU_OVMF_CODE_I386" ]; then
    OVMF_CODE_FILE="$QEMU_OVMF_CODE_I386"
    OVMF_VARS_FILE="$QEMU_OVMF_VARS_I386"
    echo "⚠ Using i386 OVMF firmware (may not boot x86_64 EFI binaries)"
else
    OVMF_CODE_FILE=""
    OVMF_VARS_FILE=""
    echo "⚠ No OVMF firmware found - running in BIOS mode (legacy boot)"
    echo "   UEFI boot requires OVMF firmware"
fi

echo ""

# Run QEMU with UEFI if firmware available, otherwise BIOS mode
if [ -n "$OVMF_CODE_FILE" ] && [ -f "$OVMF_CODE_FILE" ]; then
    echo "⚙️  Starting QEMU with UEFI firmware..."
    TEMP_VARS=$(mktemp)
    if [ -f "$OVMF_VARS_FILE" ]; then
        cp "$OVMF_VARS_FILE" "$TEMP_VARS"
    else
        dd if=/dev/zero of="$TEMP_VARS" bs=1M count=2 2>/dev/null
    fi
    trap "rm -f $TEMP_VARS" EXIT
    
    qemu-system-x86_64 \
        -machine q35 \
        -cpu qemu64 \
        -m 1G \
        -drive "file=$ISO_FILE,format=raw,media=cdrom,id=cdrom0,if=none" \
        -device "ide-cd,drive=cdrom0,bootindex=2" \
        -drive "file=fat:rw:$ISO_DIR,format=raw,if=none,id=fs0" \
        -device "virtio-blk-pci,drive=fs0,bootindex=1" \
        -drive "if=pflash,format=raw,readonly=on,file=$OVMF_CODE_FILE" \
        -drive "if=pflash,format=raw,file=$TEMP_VARS" \
        -serial stdio \
        -display cocoa \
        -no-reboot
else
    echo "⚙️  Starting QEMU in BIOS mode..."
    qemu-system-x86_64 \
        -machine q35 \
        -cpu qemu64 \
        -m 1G \
        -cdrom "$ISO_FILE" \
        -boot d \
        -serial stdio \
        -display cocoa \
        -no-reboot
fi

echo ""
echo "✅ QEMU session ended"
