# moteOS Demo Guide

## Quick Start - See moteOS in Action

### Prerequisites
✅ ISO built: `moteos-x64-uefi.iso`
✅ OVMF firmware: `ovmf/OVMF_CODE.fd` and `ovmf/OVMF_VARS.fd`
✅ QEMU installed and ready

### Run moteOS

**Option 1: Use the convenience script (recommended)**
```bash
./run-qemu.sh
```

**Option 2: Manual QEMU command**
```bash
qemu-system-x86_64 \
    -machine q35 \
    -cpu qemu64 \
    -m 1G \
    -drive file=moteos-x64-uefi.iso,format=raw,media=cdrom,id=cdrom0,if=none \
    -device ide-cd,drive=cdrom0,bootindex=1 \
    -drive if=pflash,format=raw,readonly=on,file=ovmf/OVMF_CODE.fd \
    -drive if=pflash,format=raw,file=ovmf/OVMF_VARS.fd \
    -serial stdio \
    -display sdl \
    -no-reboot
```

### What You'll See

1. **QEMU window opens** - Shows the boot process
2. **UEFI firmware** - TianoCore/OVMF boot screen
3. **moteOS bootloader** - UEFI bootloader starting
4. **Kernel initialization** - Boot messages on serial console

### Interactive Commands

- **Stop QEMU**: Press `Ctrl+C` in the terminal, or close the QEMU window
- **QEMU Monitor**: Press `Ctrl+Alt+2` to access QEMU monitor
- **Serial Console**: Press `Ctrl+Alt+1` to see serial output

### Troubleshooting

**If QEMU window doesn't open:**
- Try using `-display gtk` instead of `-display sdl`
- Or use `-display cocoa` on macOS

**If boot fails:**
- Check that OVMF firmware files are in `ovmf/` directory
- Verify ISO file exists: `ls -lh moteos-x64-uefi.iso`
- Check QEMU version: `qemu-system-x86_64 --version`
