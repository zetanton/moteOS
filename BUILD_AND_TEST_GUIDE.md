# Build and Test Guide for moteOS

This guide provides instructions for building and testing moteOS for both x86_64 and aarch64 targets.

## Prerequisites

### Rust Toolchain
```bash
# Install Rust if not already installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add UEFI targets
rustup target add x86_64-unknown-uefi
rustup target add aarch64-unknown-uefi
```

### Build Tools

**Linux (Debian/Ubuntu):**
```bash
sudo apt-get install \
    gcc-aarch64-linux-gnu \
    xorriso \
    qemu-system-x86 \
    qemu-system-arm \
    ovmf \
    qemu-efi-aarch64
```

**macOS (Homebrew):**
```bash
brew install \
    aarch64-elf-gcc \
    xorriso \
    qemu \
    edk2
```

## Build Configuration

The cross-compilation configuration is in `.cargo/config.toml`:

- **x86_64-unknown-uefi**: Uses default Rust UEFI toolchain (no custom linker needed)
- **aarch64-unknown-uefi**: Configured with custom linker for ARM64 cross-compilation

## Building

### x86_64 UEFI
```bash
# Build kernel
cargo build --release --target x86_64-unknown-uefi

# Build ISO
make iso-uefi
# or
./tools/build-iso-uefi.sh
```

### aarch64 UEFI
```bash
# Build kernel
cargo build --release --target aarch64-unknown-uefi

# Build ISO
make iso-aarch64
# or
./tools/build-iso-aarch64.sh
```

### Build All
```bash
make iso-all
```

## Testing

### Build Verification

**Test aarch64 build:**
```bash
make test-build-aarch64
# or
./tools/test-build-aarch64.sh
```

This verifies:
- ✅ Build completes without errors
- ✅ EFI binary is generated
- ✅ File size is reasonable
- ✅ File type is ARM64 ELF or PE32+
- ✅ Linker script exists
- ✅ Cargo config is present

### QEMU Integration Tests

**x86_64 Boot Test:**
```bash
make test-boot
# or
./tools/test-boot.sh
```

**aarch64 Boot Test:**
```bash
make test-boot-aarch64
# or
./tools/test-boot-aarch64.sh
```

**Network Test (x86_64):**
```bash
make test-network
# or
./tools/test-network.sh
```

**API Test (x86_64):**
```bash
make test-api
# or
./tools/test-api.sh
```

**Run All Tests:**
```bash
make test-all
```

### Quick QEMU Testing

**x86_64:**
```bash
make run-qemu-uefi
```

**aarch64:**
```bash
make run-qemu-aarch64
```

## QEMU Configuration

### x86_64 QEMU
- **Machine**: q35
- **CPU**: qemu64
- **Memory**: 1GB
- **Firmware**: OVMF (UEFI)
- **Network**: virtio-net with user networking

### aarch64 QEMU
- **Machine**: virt (QEMU ARM Virtual Machine)
- **CPU**: cortex-a72 (ARM64)
- **Memory**: 1GB
- **Firmware**: EDK2 ARM UEFI
- **Network**: virtio-net-device with user networking

## UEFI Firmware Locations

### x86_64 (OVMF)
- Linux: `/usr/share/OVMF/OVMF_CODE.fd`
- macOS: Check `/opt/homebrew/share/qemu/` or download from [EDK2](https://github.com/tianocore/edk2)

### aarch64 (EDK2 ARM)
- Linux: `/usr/share/qemu/efi/QEMU_EFI.fd` or `/usr/share/qemu/edk2-aarch64-code.fd`
- macOS: `/opt/homebrew/share/qemu/edk2-aarch64-code.fd`

## Troubleshooting

### Build Issues

**Linker not found:**
- For aarch64: Install `gcc-aarch64-linux-gnu` (Linux) or `aarch64-elf-gcc` (macOS)
- Rust may use its own linker if cross-compilation tools aren't available

**UEFI target not installed:**
```bash
rustup target add aarch64-unknown-uefi
rustup target add x86_64-unknown-uefi
```

### QEMU Issues

**OVMF firmware not found:**
- Install `ovmf` package (Linux) or download from [EDK2](https://github.com/tianocore/edk2)
- Test scripts will check common locations automatically

**ARM UEFI firmware not found:**
- Install `qemu-efi-aarch64` package (Linux)
- On macOS, install via Homebrew: `brew install edk2`

**QEMU not found:**
- Install `qemu-system-x86` and `qemu-system-arm` (Linux)
- On macOS: `brew install qemu`

### Build Artifacts

**Find EFI binaries:**
```bash
# x86_64
find target/x86_64-unknown-uefi/release -name "*.efi"

# aarch64
find target/aarch64-unknown-uefi/release -name "*.efi"
```

**Clean build artifacts:**
```bash
make clean
# or
cargo clean --target x86_64-unknown-uefi
cargo clean --target aarch64-unknown-uefi
```

## Next Steps

1. **Build both targets:**
   ```bash
   make iso-all
   ```

2. **Test builds:**
   ```bash
   make test-build-aarch64
   ```

3. **Run integration tests:**
   ```bash
   make test-boot
   make test-boot-aarch64
   ```

4. **For Raspberry Pi deployment:**
   - See `AARCH64_CROSS_COMPILATION.md` for detailed instructions
   - Flash ISO to MicroSD card
   - Install UEFI firmware (RPi4_UEFI_Firmware)

## See Also

- `AARCH64_CROSS_COMPILATION.md` - Detailed aarch64 build instructions
- `docs/TECHNICAL_SPECIFICATIONS.md` - Technical specifications
- `tools/` - Build and test scripts
