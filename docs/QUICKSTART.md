# moteOS Quick Start Guide

Get moteOS running in QEMU in 5 minutes!

## 🚀 Quick Start (x86_64)

### Step 1: Set up Rust environment

```bash
# If Rust isn't in your PATH, add it:
source ~/.cargo/env

# Or add to your ~/.zshrc:
echo 'source ~/.cargo/env' >> ~/.zshrc
source ~/.zshrc
```

### Step 2: Install Rust targets

```bash
rustup target add x86_64-unknown-uefi
rustup target add aarch64-unknown-uefi
```

### Step 3: Build the system

```bash
# Build x86_64 UEFI ISO (easiest to start with)
make iso-uefi
```

This will:
1. Compile the kernel for x86_64 UEFI
2. Create a bootable ISO image
3. Output: `moteos-x64-uefi.iso`

### Step 4: Run in QEMU

```bash
# Quick way - builds and runs in one command
make run-qemu-uefi

# Or manually:
make test-boot
```

You should see:
- QEMU starting up
- UEFI firmware loading
- Kernel boot output in the terminal
- System initializing

## 🎯 What You'll See

When running in QEMU, you'll see:
- UEFI firmware initialization
- Kernel boot messages
- System components loading (memory, network, etc.)
- TUI interface if configured
- Serial console output

**Press Ctrl+C** to stop QEMU when done testing.

## 📋 Full Test Suite

### Test x86_64 Boot
```bash
make test-boot
```

### Test aarch64 Build (if you want ARM support)
```bash
make test-build-aarch64
make test-boot-aarch64
```

### Test Network Connectivity
```bash
make test-network
```

### Run All Tests
```bash
make test-all
```

## 🛠️ Troubleshooting

### "command not found: rustup" or "cargo not found"

```bash
# Install Rust if you haven't:
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### "qemu-system-x86_64: command not found"

**macOS:**
```bash
brew install qemu
```

**Linux (Debian/Ubuntu):**
```bash
sudo apt-get install qemu-system-x86 qemu-system-arm
```

### "OVMF firmware not found" warning

The system will still try to boot without it, but you may need UEFI firmware:

**macOS:**
```bash
brew install edk2
```

**Linux:**
```bash
sudo apt-get install ovmf
```

### Build Errors

If you see linker errors or missing dependencies:
```bash
# Check what's needed
make help

# Clean and rebuild
make clean
make iso-uefi
```

## 🎨 Next Steps

Once you see it booting:

1. **Explore the TUI** - If the TUI is enabled, you'll see the interface
2. **Test Network** - Run `make test-network` to test connectivity
3. **Check Logs** - Boot logs are saved to `/tmp/moteos-boot-test.log`
4. **Read the Code** - Check `boot/src/` and `kernel/src/` to understand the system

## 📚 More Information

- `BUILD_AND_TEST_GUIDE.md` - Detailed build instructions
- `docs/TECHNICAL_SPECIFICATIONS.md` - Full technical specs
- `AARCH64_CROSS_COMPILATION.md` - ARM/Raspberry Pi instructions

## 💡 Tips

- **First time?** Start with x86_64 - it's the most straightforward
- **Want faster builds?** Use `cargo build --release` directly
- **Testing changes?** Use `cargo build` (debug mode) for faster iteration
- **View output?** Boot output goes to stdout - you'll see it in your terminal
- **QEMU not responding?** Press Ctrl+C, then Ctrl+A then X to force quit

---

**Ready?** Run: `make iso-uefi && make run-qemu-uefi`
