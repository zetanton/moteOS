# AArch64 Cross-Compilation for Raspberry Pi

This document describes the implementation of aarch64 cross-compilation support for moteOS, targeting Raspberry Pi 4 and other ARM64 UEFI systems.

## Overview

The implementation adds support for building moteOS for the aarch64 (ARM64) architecture, specifically targeting Raspberry Pi 4 which supports UEFI boot. This enables running moteOS on ARM-based single-board computers.

## Implementation Details

### 1. Target Configuration (`.cargo/config.toml`)

Added configuration for the `aarch64-unknown-uefi` target with:
- Custom linker: `aarch64-linux-gnu-ld`
- Linker script: `linker-aarch64.ld`
- Linker flags for 4KB page size alignment (required for ARM64 MMU)

### 2. ARM64 Linker Script (`linker-aarch64.ld`)

Created a linker script specifically for ARM64 UEFI boot:
- Entry point: `efi_main`
- Base address: `0x80000` (512KB - typical UEFI load address)
- Memory layout: text, rodata, data, and BSS sections
- 4KB page alignment for MMU compatibility
- Stack top at 16MB (adjustable based on system)

### 3. AArch64 UEFI Boot Implementation (`boot/src/uefi/aarch64.rs`)

Implemented the ARM64 UEFI boot code:
- **Entry Point**: `efi_main` function following UEFI calling conventions
- **Framebuffer Acquisition**: Uses Graphics Output Protocol (GOP) to get framebuffer
- **Memory Map**: Retrieves and parses UEFI memory map
- **Heap Setup**: Finds largest usable memory region and reserves 64MB for heap
- **MMU Configuration**: Placeholder for MMU setup (UEFI typically handles this)
- **Boot Services Exit**: Properly exits UEFI boot services before kernel initialization

Key differences from x86_64:
- Uses ARM64 assembly instructions (`wfi` for halt instead of `hlt`)
- MMU configuration considerations (4KB pages, 48-bit addressing)
- No x86_64-specific instructions or registers

### 4. Build Scripts

#### `tools/build-iso-aarch64.sh`
- Builds moteOS for `aarch64-unknown-uefi` target
- Creates bootable ISO with proper EFI directory structure
- Generates `BOOTAA64.EFI` (ARM64 UEFI boot file)
- Creates Raspberry Pi specific configuration files:
  - `config.txt`: Raspberry Pi firmware configuration
  - `README.md`: Installation and troubleshooting guide

#### `tools/test-build-aarch64.sh`
- Verifies cross-compilation build completes successfully
- Checks for expected build artifacts (`.efi` file)
- Validates file size and type (ARM64 ELF or PE32+)
- Verifies linker script and cargo config are present

### 5. Makefile Updates

Added new targets:
- `make iso-aarch64`: Build AArch64 UEFI ISO
- `make test-build-aarch64`: Test AArch64 build verification
- Updated `make iso-all`: Now includes aarch64 ISO
- Updated `make clean`: Cleans aarch64 build artifacts

### 6. Raspberry Pi Configuration

The build script generates Raspberry Pi specific files:

**`config.txt`**:
- Enables 64-bit mode (`arm_64bit=1`)
- Configures GPU memory split (64MB minimum)
- Sets up HDMI output
- Disables overscan
- Configures boot delay

**Installation Requirements**:
- Raspberry Pi 4 or 400 (ARM64 support required)
- UEFI firmware (e.g., RPi4_UEFI_Firmware from https://github.com/pftf/RPi4)
- MicroSD card (8GB minimum, 16GB recommended)
- USB keyboard for input

### 7. Binary Target Configuration

Updated `boot/Cargo.toml`:
- Added `[[bin]]` target named `boot` pointing to `src/uefi_bin.rs`
- Made `x86_64` dependency conditional (only for x86_64 builds)
- Created `boot/src/uefi_bin.rs` as the binary entry point that delegates to architecture-specific implementations

## Building for AArch64

### Prerequisites

1. **Rust toolchain** with `aarch64-unknown-uefi` target:
   ```bash
   rustup target add aarch64-unknown-uefi
   ```

2. **Cross-compilation toolchain** (optional, Rust may use its own):
   - Debian/Ubuntu: `sudo apt-get install gcc-aarch64-linux-gnu`
   - macOS: `brew install aarch64-elf-gcc`

3. **Build tools**:
   - `xorriso` for ISO creation
   - `cargo` (Rust package manager)

### Build Commands

**Test the build**:
```bash
make test-build-aarch64
```

**Build the ISO**:
```bash
make iso-aarch64
```

**Build all ISOs** (x86_64 and aarch64):
```bash
make iso-all
```

### Manual Build

```bash
# Build the kernel
cargo build --release --target aarch64-unknown-uefi

# Create ISO
./tools/build-iso-aarch64.sh
```

## Installation on Raspberry Pi

1. **Flash the ISO** to a MicroSD card:
   ```bash
   sudo dd if=moteos-aarch64-uefi.iso of=/dev/sdX bs=4M status=progress
   ```

2. **Install UEFI firmware**:
   - Download RPi4_UEFI_Firmware from https://github.com/pftf/RPi4
   - Extract and copy `RPI_EFI.fd` to the boot partition
   - Ensure `BOOTAA64.EFI` is in `EFI/BOOT/` directory

3. **Insert MicroSD card** into Raspberry Pi 4

4. **Power on** - the system should boot into moteOS

## Technical Specifications

### Memory Layout
- **Base Address**: 0x80000 (512KB)
- **Stack Top**: 0x1000000 (16MB)
- **Heap**: Minimum 64MB from largest usable memory region
- **Page Size**: 4KB (ARM64 standard)

### UEFI Compatibility
- **Firmware**: Requires UEFI 2.x compliant firmware
- **Boot File**: `BOOTAA64.EFI` (ARM64 UEFI application)
- **Graphics**: Uses Graphics Output Protocol (GOP)
- **Memory Map**: Parses UEFI memory descriptors

### Architecture Support
- **Primary Target**: Raspberry Pi 4 Model B
- **Architecture**: AArch64 (64-bit ARM)
- **Endianness**: Little-endian
- **Instruction Set**: ARMv8-A

## Limitations and Notes

1. **Raspberry Pi 3 Not Supported**: Only 32-bit ARM support
2. **UEFI Firmware Required**: Standard Raspberry Pi firmware doesn't support UEFI
3. **USB Networking**: May require additional driver support
4. **MMU Setup**: Currently placeholder - UEFI typically handles MMU initialization
5. **Device Tree**: ARM64 systems may use Device Tree instead of ACPI (not yet implemented)

## Testing

The build verification script (`test-build-aarch64.sh`) checks:
- ✅ Build completes without errors
- ✅ EFI binary is generated
- ✅ File size is reasonable
- ✅ File type is ARM64 ELF or PE32+
- ✅ Linker script exists
- ✅ Cargo config is present

## Future Improvements

1. **MMU Implementation**: Full MMU setup and page table management
2. **Device Tree Support**: Parse and use Device Tree for hardware discovery
3. **Raspberry Pi Specific Drivers**: GPIO, USB, network drivers
4. **QEMU Testing**: Add QEMU aarch64 test targets
5. **Serial Console**: Enhanced UART debugging support

## References

- [UEFI Specification](https://uefi.org/specifications)
- [Raspberry Pi 4 UEFI Firmware](https://github.com/pftf/RPi4)
- [ARM Architecture Reference Manual](https://developer.arm.com/documentation/ddi0487/latest)
- [Rust Embedded Book - Cross Compilation](https://docs.rust-embedded.org/book/intro/install.html#cross-compilation)

## See Also

- `docs/TECHNICAL_SPECIFICATIONS.md` Section 3.8.7 - AArch64 Cross-Compilation
- `boot/src/uefi/aarch64.rs` - AArch64 UEFI implementation
- `linker-aarch64.ld` - ARM64 linker script
- `tools/build-iso-aarch64.sh` - Build script
- `tools/test-build-aarch64.sh` - Test script
