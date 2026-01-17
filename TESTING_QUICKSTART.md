# moteOS Testing Quick Start Guide

This guide will help you quickly begin testing moteOS. The full testing plan is documented in [docs/TESTING_PLAN.md](docs/TESTING_PLAN.md).

## Quick Start (5 minutes)

### 1. Verify Prerequisites

```bash
# Check if required tools are installed
which cargo rustc qemu-system-x86_64 xorriso python3

# If any are missing, install them:
# macOS:
brew install qemu xorriso
# Linux:
sudo apt-get install qemu-system-x86 xorriso python3

# Install Rust targets
rustup target add x86_64-unknown-uefi
rustup target add aarch64-unknown-uefi
```

### 2. Run All Tests Automatically

```bash
# Run comprehensive test suite
./tools/run-all-tests.sh

# Or use Make targets
make test-all
```

### 3. Run Individual Tests

```bash
# Unit tests only (fast, no QEMU)
cargo test --workspace

# Build and boot test
make test-boot

# Network test
make test-network

# API test
make test-api
```

## Manual Testing Steps

### Step 1: Build ISO

```bash
# Build x86_64 UEFI ISO (most common)
make iso-uefi

# Verify ISO was created
ls -lh moteos-x64-uefi.iso
```

### Step 2: Test Boot

```bash
# Run boot test (automated, ~30 seconds)
./tools/test-boot.sh

# Or boot manually in QEMU
qemu-system-x86_64 \
    -machine q35 \
    -cpu qemu64 \
    -m 1G \
    -cdrom moteos-x64-uefi.iso \
    -netdev user,id=net0 \
    -device virtio-net,netdev=net0 \
    -serial stdio \
    -display none
```

### Step 3: Test Network

```bash
# Automated network test (DHCP, DNS, HTTP)
./tools/test-network.sh
```

### Step 4: Test API (Mock Server)

```bash
# Starts mock LLM API server and tests connectivity
./tools/test-api.sh
```

## What to Look For

### Successful Boot Test
- ✅ QEMU starts without errors
- ✅ Kernel output appears on serial console
- ✅ Message like "moteOS" or "kernel_main" in output
- ✅ No kernel panics or infinite loops

### Successful Network Test
- ✅ DHCP acquires IP address (typically 10.0.2.15)
- ✅ DNS resolution works
- ✅ HTTP connection to test server succeeds

### Successful API Test
- ✅ Mock API server starts
- ✅ System connects to mock API
- ✅ API request sent and response received

## Common Issues

### Issue: "command not found: cargo"
**Solution**: Install Rust:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Issue: "OVMF firmware not found"
**Solution**: Install OVMF (UEFI firmware):
```bash
# macOS
brew install ovmf

# Linux
sudo apt-get install ovmf
```

### Issue: "ISO build fails"
**Solution**: Check build logs:
```bash
# Clean and rebuild
cargo clean
make iso-uefi

# Check for linker errors
cargo build --release --target x86_64-unknown-uefi
```

### Issue: "QEMU boot hangs"
**Solution**: 
- Check serial console output (should see kernel messages)
- Verify ISO integrity: `file moteos-x64-uefi.iso`
- Try booting without UEFI (BIOS mode)

### Issue: "Network test fails"
**Solution**:
- Verify QEMU user networking is working
- Check firewall settings
- Verify test HTTP server starts (check `/tmp/test-server.log`)

## Test Logs

All tests generate logs in `/tmp/`:
- `/tmp/moteos-boot-test.log` - Boot test output
- `/tmp/moteos-network-test.log` - Network test output
- `/tmp/moteos-api-test.log` - API test output
- `/tmp/moteos-test-suite.log` - Complete test suite log

## Next Steps

1. **If tests pass**: Great! The system is working. Try:
   - Booting on real hardware (flash ISO to USB)
   - Testing with real LLM API (requires API keys)
   - Testing on Raspberry Pi 4 (AArch64)

2. **If tests fail**: 
   - Check test logs for specific errors
   - Review [docs/TESTING_PLAN.md](docs/TESTING_PLAN.md) troubleshooting section
   - File issues with test output attached

3. **For detailed testing**: See [docs/TESTING_PLAN.md](docs/TESTING_PLAN.md) for comprehensive test strategy

## Getting Help

- **Test failures**: Check logs in `/tmp/moteos-*-test.log`
- **Build issues**: Check `Cargo.toml` and linker scripts
- **QEMU issues**: Verify QEMU version and firmware paths
- **Questions**: Review technical specs in `docs/TECHNICAL_SPECIFICATIONS.md`
