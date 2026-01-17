# ISO Generation and QEMU Testing

This document describes how to build bootable ISO images and test them in QEMU.

## Overview

moteOS supports two boot methods:
- **UEFI boot** (x86_64) - Modern firmware boot method
- **BIOS boot** (x86_64) - Legacy BIOS boot using Multiboot2

## Prerequisites

### Required Tools

- **Rust toolchain** (nightly recommended)
  ```bash
  rustup toolchain install nightly
  rustup default nightly
  ```

- **UEFI target** (for UEFI boot)
  ```bash
  rustup target add x86_64-unknown-uefi
  ```

- **xorriso** (for ISO creation)
  ```bash
  # Debian/Ubuntu
  sudo apt-get install xorriso
  
  # macOS
  brew install xorriso
  ```

- **QEMU** (for testing)
  ```bash
  # Debian/Ubuntu
  sudo apt-get install qemu-system-x86 ovmf
  
  # macOS
  brew install qemu
  ```

- **OVMF** (UEFI firmware for QEMU)
  ```bash
  # Debian/Ubuntu
  sudo apt-get install ovmf
  
  # macOS (download manually)
  # https://github.com/tianocore/edk2/releases
  ```

### Optional Tools

- **grub-mkrescue** (for better BIOS boot support)
  ```bash
  sudo apt-get install grub-pc-bin
  ```

## Building ISOs

### UEFI Boot ISO

Build a UEFI bootable ISO:

```bash
make iso-uefi
# or
./tools/build-iso-uefi.sh
```

This will:
1. Build the kernel for `x86_64-unknown-uefi` target
2. Create ISO directory structure (`iso/EFI/BOOT/`)
3. Copy the EFI binary to `BOOTX64.EFI`
4. Create `moteos-x64-uefi.iso`

### BIOS Boot ISO

Build a BIOS bootable ISO:

```bash
make iso-bios
# or
./tools/build-iso-bios.sh
```

**Note**: BIOS boot requires proper Multiboot2 bootloader setup. The current implementation may need additional configuration.

### Build Both

```bash
make iso-all
```

## QEMU Testing

### Boot Test

Test that the kernel boots successfully:

```bash
make test-boot
# or
./tools/test-boot.sh
```

This will:
1. Build the ISO if it doesn't exist
2. Start QEMU with UEFI firmware
3. Boot the ISO
4. Check for successful boot indicators

### Network Test

Test network connectivity (DHCP, DNS, HTTP):

```bash
make test-network
# or
./tools/test-network.sh
```

This will:
1. Start a test HTTP server on the host
2. Boot moteOS in QEMU with network support
3. Test DHCP acquisition
4. Test DNS resolution
5. Test HTTP connectivity

### API Test

Test LLM API connectivity:

```bash
make test-api
# or
./tools/test-api.sh
```

This will:
1. Start a mock LLM API server
2. Boot moteOS in QEMU with network support
3. Test TLS/HTTPS connections
4. Test API request/response handling

### Run All Tests

```bash
make test-all
```

## Manual QEMU Commands

### UEFI Boot

```bash
qemu-system-x86_64 \
    -machine q35 \
    -cpu qemu64 \
    -m 1G \
    -drive if=pflash,format=raw,readonly=on,file=/usr/share/OVMF/OVMF_CODE.fd \
    -drive if=pflash,format=raw,file=/tmp/OVMF_VARS.fd \
    -cdrom moteos-x64-uefi.iso \
    -netdev user,id=net0 \
    -device virtio-net,netdev=net0 \
    -serial stdio \
    -display none
```

### BIOS Boot

```bash
qemu-system-x86_64 \
    -machine q35 \
    -cpu qemu64 \
    -m 1G \
    -cdrom moteos-x64-bios.iso \
    -netdev user,id=net0 \
    -device virtio-net,netdev=net0 \
    -serial stdio \
    -display none
```

## Troubleshooting

### Build Issues

**Error: "no .efi file found"**
- The boot crate may need a `[[bin]]` target in `boot/Cargo.toml`
- Check that the UEFI entry point (`efi_main`) is properly exported
- Verify the target is `x86_64-unknown-uefi`

**Error: "xorriso not found"**
- Install xorriso (see Prerequisites)
- On macOS: `brew install xorriso`

### QEMU Issues

**Error: "OVMF firmware not found"**
- Install OVMF package (see Prerequisites)
- Or download OVMF manually and specify path in scripts
- The scripts will continue without UEFI firmware (legacy BIOS mode)

**Network not working**
- Check that QEMU user networking is enabled
- Verify host forwarding rules in test scripts
- Check firewall settings

### ISO Boot Issues

**UEFI ISO doesn't boot**
- Verify EFI binary is correctly placed at `EFI/BOOT/BOOTX64.EFI`
- Check that OVMF firmware is properly configured
- Try booting on real hardware to verify ISO structure

**BIOS ISO doesn't boot**
- Verify Multiboot2 header is present
- Check GRUB configuration
- Ensure bootloader is properly set up

## File Structure

After building, you'll have:

```
moteOS/
├── iso/                    # UEFI ISO directory
│   └── EFI/
│       └── BOOT/
│           └── BOOTX64.EFI
├── iso-bios/               # BIOS ISO directory
│   └── boot/
│       ├── grub/
│       │   └── grub.cfg
│       └── kernel/
│           └── moteos
├── moteos-x64-uefi.iso     # UEFI bootable ISO
└── moteos-x64-bios.iso     # BIOS bootable ISO
```

## References

- [Technical Specifications](../docs/TECHNICAL_SPECIFICATIONS.md) - Section 3.8.8-9
- [UEFI Specification](https://uefi.org/specifications)
- [Multiboot2 Specification](https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html)
- [QEMU Documentation](https://www.qemu.org/documentation/)
