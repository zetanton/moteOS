# 🚀 See moteOS in Action!

## ✅ Everything is Ready!

Your moteOS ISO is built and ready to run:
- **ISO**: `moteos-x64-uefi.iso` (372K)
- **EFI Binary**: `target/x86_64-unknown-uefi/release/boot.efi`
- **QEMU**: Installed and ready

## 🎬 Run moteOS Now

### Quick Start (Recommended)
```bash
./run-qemu.sh
```

This will:
- Open a QEMU window
- Boot the moteOS ISO
- Show boot output in terminal

### What You'll See

When moteOS boots, you'll see:
1. **QEMU window opens** - Graphics display
2. **Boot process** - UEFI bootloader starting
3. **Kernel initialization** - Boot messages appear
4. **System ready** - moteOS UI (if implemented)

### Boot Output

Boot messages appear in your terminal. You should see:
- UEFI firmware messages
- Bootloader output
- Kernel initialization
- System ready messages

### Controls

- **Stop QEMU**: Press `Ctrl+C` in terminal, or close QEMU window
- **QEMU Monitor**: Press `Ctrl+Alt+2` (to switch to monitor)
- **Serial Console**: Press `Ctrl+Alt+1` (to switch to console)

## 📝 Note About UEFI Firmware

The current setup runs in BIOS mode. For full UEFI boot with proper firmware:

1. Download OVMF firmware from:
   https://github.com/tianocore/edk2/releases
   
2. Extract and place in `ovmf/` directory:
   - `OVMF_CODE.fd`
   - `OVMF_VARS.fd`

3. The `run-qemu.sh` script will automatically use them if found.

## 🐛 Troubleshooting

**QEMU window doesn't appear:**
- Try changing `-display sdl` to `-display cocoa` in `run-qemu.sh`
- Or use `-display gtk` if GTK is installed

**Boot fails:**
- Verify ISO exists: `ls -lh moteos-x64-uefi.iso`
- Check EFI binary: `ls -lh target/x86_64-unknown-uefi/release/boot.efi`
- Rebuild if needed: `make iso-uefi`

**Want to see more output:**
- Add `-d guest_errors` to QEMU command for debug output
- Check terminal for serial console messages

---

**Ready? Run `./run-qemu.sh` now!** 🚀
