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

# Choose display backend (cocoa|sdl|vnc). Default is cocoa.
# NOTE: With cocoa display, keyboard input goes to the QEMU window, not the terminal.
# To type in terminal, use: QEMU_DISPLAY=none ./run-qemu.sh (headless with serial only)
DISPLAY_BACKEND="${QEMU_DISPLAY:-cocoa}"
SERIAL_MODE="${QEMU_SERIAL:-stdio}"
# Use chardev for better terminal handling on macOS
# signal=off prevents Ctrl+C from killing QEMU
# mux=on allows multiplexing with QEMU commands (Ctrl+A x to quit)
if [ "$SERIAL_MODE" = "stdio" ]; then
    SERIAL_ARGS="-chardev stdio,id=char0,mux=on,signal=off -serial chardev:char0 -mon chardev=char0"
    echo "✓ Serial on stdio (Ctrl+A X to quit, Ctrl+A H for help)"
fi
MONITOR_ARGS=""
if [ "$SERIAL_MODE" = "pty" ]; then
    SERIAL_ARGS="-serial pty"
    echo "✓ Serial on PTY (QEMU will print the PTY path)"
    echo "  Use: screen /dev/ttyXXX (from QEMU output)"
elif [ "$SERIAL_MODE" = "tcp" ]; then
    SERIAL_PORT="${QEMU_SERIAL_PORT:-5555}"
    SERIAL_ARGS="-serial tcp:127.0.0.1:${SERIAL_PORT},server,nowait -serial tcp:127.0.0.1:$((SERIAL_PORT+1)),server,nowait"
    echo "✓ Serial on TCP ports ${SERIAL_PORT} (COM1) and $((SERIAL_PORT+1)) (COM2)"
    echo "  Connect with: nc 127.0.0.1 ${SERIAL_PORT}  (or ${SERIAL_PORT}+1)"
fi
if [ "$DISPLAY_BACKEND" = "vnc" ]; then
    VNC_ADDR="${QEMU_VNC_ADDR:-127.0.0.1:1}"
    if [ -n "${QEMU_VNC_PASSWORD:-}" ]; then
        VNC_OPTS="password=on"
        SERIAL_ARGS="-serial file:/tmp/moteos-serial.log"
        MONITOR_ARGS="-monitor stdio"
        echo "✓ VNC display enabled at $VNC_ADDR ($VNC_OPTS)"
        echo "  Set password in QEMU monitor: change vnc password <password>"
        echo "  Serial log: /tmp/moteos-serial.log"
    else
        VNC_OPTS="${QEMU_VNC_OPTS:-password=off}"
        echo "✓ VNC display enabled at $VNC_ADDR ($VNC_OPTS)"
    fi
    DISPLAY_ARGS="-display none -vnc ${VNC_ADDR},${VNC_OPTS}"
    echo "  Connect with: open vnc://127.0.0.1:5901"
elif [ "$DISPLAY_BACKEND" = "sdl" ]; then
    DISPLAY_ARGS="-sdl"
    echo "✓ SDL display enabled (legacy -sdl flag)"
elif [ "$DISPLAY_BACKEND" = "none" ]; then
    DISPLAY_ARGS="-display none"
    echo "✓ Headless mode (no display, serial only)"
else
    # Default resolution for cocoa display (smaller window)
    QEMU_RES="${QEMU_RES:-1280x720}"
    DISPLAY_ARGS="-display cocoa,show-cursor=on -device virtio-vga,xres=${QEMU_RES%x*},yres=${QEMU_RES#*x}"
    echo "✓ Cocoa display enabled (${QEMU_RES})"
fi

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
    
        # Note on keyboard input options:
    # 1. PS/2: Click in QEMU window and type there (scancodes via i8042 controller)
    # 2. Serial: Type in this terminal (characters via COM1 serial port)
    if [ "$DISPLAY_BACKEND" = "none" ]; then
        echo "📝 Headless mode - type in this terminal for serial input"
        echo "   Ctrl+A X to quit QEMU"
    else
        echo "📝 Input options:"
        echo "   - Click in QEMU window and type there (PS/2 keyboard)"
        echo "   - Or run: QEMU_DISPLAY=none ./run-qemu.sh (for terminal input)"
    fi
    echo ""

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
        $SERIAL_ARGS \
        $MONITOR_ARGS \
        $DISPLAY_ARGS \
        -no-reboot
else
    echo "⚙️  Starting QEMU in BIOS mode..."
    qemu-system-x86_64 \
        -machine q35 \
        -cpu qemu64 \
        -m 1G \
        -k en-us \
        -cdrom "$ISO_FILE" \
        -boot d \
        $SERIAL_ARGS \
        $MONITOR_ARGS \
        $DISPLAY_ARGS \
        -no-reboot
fi

echo ""
echo "✅ QEMU session ended"
