# moteOS Testing Plan

This document outlines a comprehensive testing strategy for moteOS covering all 8 workstreams.

## Testing Overview

moteOS testing is organized into three levels:
1. **Unit Tests** - Test individual components in isolation
2. **Integration Tests** - Test component interactions (QEMU-based)
3. **End-to-End Tests** - Test full system boot and functionality

## Prerequisites

### Required Tools
- **Rust**: Nightly toolchain with `x86_64-unknown-uefi` and `aarch64-unknown-uefi` targets
- **QEMU**: `qemu-system-x86_64` and `qemu-system-aarch64`
- **OVMF**: UEFI firmware (for UEFI boot tests)
- **xorriso**: ISO generation
- **Python3**: For mock API servers in tests

### Installation (macOS)
```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup toolchain install nightly
rustup default nightly
rustup target add x86_64-unknown-uefi
rustup target add aarch64-unknown-uefi

# Install QEMU
brew install qemu

# Install OVMF (UEFI firmware)
brew install ovmf

# Install ISO tools
brew install xorriso
```

### Installation (Linux/Debian)
```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup toolchain install nightly
rustup default nightly
rustup target add x86_64-unknown-uefi
rustup target add aarch64-unknown-uefi

# QEMU and tools
sudo apt-get install qemu-system-x86 qemu-system-arm xorriso ovmf python3
```

## Test Execution Strategy

### Phase 1: Unit Tests (No QEMU required)
Run unit tests for all crates to verify individual component functionality.

### Phase 2: Build Verification
Verify that ISOs can be built for all target platforms.

### Phase 3: Boot Tests
Verify that the kernel boots successfully in QEMU.

### Phase 4: Integration Tests
Test network, API, and TUI functionality in QEMU.

### Phase 5: End-to-End Tests
Test complete workflows (boot → network → API → TUI).

## Test Categories

### 1. Boot System Tests (Workstream 1 & 8)

#### 1.1 UEFI Boot (x86_64)
- **Test**: `make test-boot` or `./tools/test-boot.sh`
- **Expected**: Kernel boots, reaches `kernel_main()`, displays framebuffer output
- **Duration**: ~30 seconds

#### 1.2 BIOS Boot (x86_64)
- **Test**: Build BIOS ISO and boot in QEMU
- **Expected**: Multiboot2 boot, kernel initialization
- **Note**: May require Multiboot2 bootloader

#### 1.3 AArch64 Boot
- **Test**: `./tools/test-build-aarch64.sh` then boot in QEMU
- **Expected**: ARM64 EFI binary produced, boots on Raspberry Pi 4

### 2. Network Tests (Workstream 2)

#### 2.1 DHCP Client
- **Test**: `make test-network` or `./tools/test-network.sh`
- **Expected**: DHCP IP acquisition, gateway/router configuration
- **Check**: IP address assigned (typically 10.0.2.15 in QEMU)

#### 2.2 DNS Resolver
- **Test**: Included in network test
- **Expected**: DNS queries resolve (e.g., `google.com` → IP address)
- **Check**: DNS resolution logs in test output

#### 2.3 TLS/HTTPS Connection
- **Test**: TLS handshake with real servers
- **Expected**: Successful TLS 1.3 handshake
- **Target**: `api.openai.com:443`, `api.anthropic.com:443`

#### 2.4 HTTP Client
- **Test**: HTTP GET/POST requests
- **Expected**: HTTP/1.1 requests sent, responses parsed
- **Note**: Test with mock server first (`test-api.sh`)

### 3. LLM Provider Tests (Workstream 4)

#### 3.1 OpenAI Client
- **Test**: Connection to OpenAI API (or mock)
- **Expected**: Streaming responses parsed correctly
- **Check**: Token streaming, completion events

#### 3.2 Anthropic Client
- **Test**: Connection to Anthropic API (or mock)
- **Expected**: Server-Sent Events (SSE) parsing
- **Check**: Message streaming

#### 3.3 Groq Client
- **Test**: Connection to Groq API
- **Expected**: Fast inference API responses

#### 3.4 xAI Client
- **Test**: Connection to xAI API
- **Expected**: API-compatible requests/responses

#### 3.5 Mock API Test
- **Test**: `make test-api` or `./tools/test-api.sh`
- **Expected**: Mock server receives requests, sends streaming responses
- **Check**: Request/response logs

### 4. Local Inference Tests (Workstream 6)

#### 4.1 GGUF Parser
- **Test**: Unit tests in `inference/src/gguf.rs`
- **Expected**: GGUF file header parsed correctly
- **Note**: Requires test GGUF file

#### 4.2 Tokenizer
- **Test**: Unit tests in `inference/src/tokenizer.rs`
- **Expected**: BPE tokenization produces correct tokens
- **Check**: Token IDs match expected values

#### 4.3 Tensor Operations
- **Test**: Unit tests in `inference/src/ops.rs`
- **Expected**: MatMul, activation functions work correctly
- **Check**: Numerical accuracy (within floating-point tolerance)

#### 4.4 Transformer Forward Pass
- **Test**: Integration test with small model
- **Expected**: Forward pass completes without errors
- **Note**: Requires SmolLM-360M or smaller test model

### 5. TUI Tests (Workstream 3)

#### 5.1 Framebuffer Rendering
- **Test**: Unit tests in `boot/src/framebuffer.rs`
- **Expected**: Color/rect drawing functions work
- **Check**: Visual output in QEMU

#### 5.2 Font Rendering
- **Test**: Character rendering unit tests
- **Expected**: PSF font loaded, characters rendered correctly

#### 5.3 Input Widget
- **Test**: Unit tests in `tui/src/widgets/input.rs`
- **Expected**: Text input, cursor movement, editing work
- **Check**: Interactive input in QEMU

#### 5.4 Chat Screen
- **Test**: Visual test in QEMU
- **Expected**: Message list, input field, scrolling work
- **Check**: UI layout and responsiveness

#### 5.5 Theme System
- **Test**: Unit tests in `tui/src/theme.rs`, `tui/tests/color_rendering.rs`
- **Expected**: Dark theme colors applied correctly
- **Check**: Visual appearance matches design

### 6. Configuration Tests (Workstream 7)

#### 6.1 TOML Parser
- **Test**: Unit tests in `config/src/toml.rs`
- **Expected**: TOML config files parsed correctly
- **Check**: API keys, provider settings loaded

#### 6.2 EFI Variable Storage
- **Test**: Unit tests in `config/src/storage/efi.rs`
- **Expected**: Config saved/loaded from EFI variables
- **Note**: Requires UEFI environment

#### 6.3 Setup Wizard
- **Test**: Visual test in QEMU
- **Expected**: Wizard prompts for config, saves settings
- **Check**: Config persistence across boots

### 7. Build System Tests (Workstream 8)

#### 7.1 x86_64 UEFI ISO
- **Test**: `make iso-uefi`
- **Expected**: `moteos-x64-uefi.iso` created successfully
- **Check**: ISO size, structure (EFI bootable)

#### 7.2 x86_64 BIOS ISO
- **Test**: `make iso-bios`
- **Expected**: `moteos-x64-bios.iso` created successfully
- **Check**: ISO boots from CD/DVD

#### 7.3 AArch64 ISO
- **Test**: `make iso-aarch64`
- **Expected**: `moteos-aarch64-uefi.iso` created successfully
- **Check**: ISO boots on Raspberry Pi 4

### 8. Integration Tests

#### 8.1 Boot → Network → API Flow
- **Test**: Full boot, network init, API call
- **Expected**: System boots, connects to network, makes API request
- **Duration**: ~90 seconds

#### 8.2 Boot → Config → TUI Flow
- **Test**: Boot with config, display TUI
- **Expected**: Config loaded, TUI rendered, input responsive
- **Check**: Visual verification

#### 8.3 Complete User Flow
- **Test**: Boot → Config → Network → API → TUI → Conversation
- **Expected**: End-to-end functionality works
- **Duration**: ~2 minutes

## Running Tests

### Quick Start
```bash
# Run all tests (recommended)
make test-all

# Or run individually
make test-boot        # Boot test
make test-network     # Network test
make test-api         # API test
make test-build-aarch64  # AArch64 build test
```

### Manual Test Execution

#### Unit Tests
```bash
# Test all crates
cargo test --workspace

# Test specific crate
cargo test -p network
cargo test -p tui
cargo test -p inference
cargo test -p config
```

#### Build Tests
```bash
# Build x86_64 UEFI
make iso-uefi

# Build x86_64 BIOS
make iso-bios

# Build AArch64
make iso-aarch64

# Build all
make iso-all
```

#### Integration Tests (QEMU)
```bash
# Boot test
./tools/test-boot.sh

# Network test
./tools/test-network.sh

# API test
./tools/test-api.sh
```

## Test Logs and Artifacts

### Log Locations
- **Boot test**: `/tmp/moteos-boot-test.log`
- **Network test**: `/tmp/moteos-network-test.log`
- **API test**: `/tmp/moteos-api-test.log`
- **Mock API server**: `/tmp/mock-api-server.log`

### Test Artifacts
- **ISOs**: `moteos-x64-uefi.iso`, `moteos-x64-bios.iso`, `moteos-aarch64-uefi.iso`
- **Build artifacts**: `target/x86_64-unknown-uefi/release/`, `target/aarch64-unknown-uefi/release/`

## Test Success Criteria

### Minimum Viable Test Results
- ✅ All unit tests pass
- ✅ All ISOs build successfully
- ✅ x86_64 UEFI boot reaches `kernel_main()`
- ✅ Network test shows DHCP/IP acquisition
- ✅ Mock API test shows request/response flow

### Full Test Suite Pass
- ✅ All unit tests pass (100%)
- ✅ All ISOs build and boot
- ✅ Network: DHCP, DNS, TLS, HTTP all working
- ✅ LLM API: At least one provider works (mock or real)
- ✅ TUI: Framebuffer, input, chat screen functional
- ✅ Config: TOML parsing, EFI storage working

## Troubleshooting

### QEMU Issues
- **OVMF not found**: Install `ovmf` package or download from Tianocore
- **Boot hangs**: Check serial console output, verify ISO integrity
- **Network not working**: Verify QEMU user networking, check firewall

### Build Issues
- **Missing target**: Run `rustup target add x86_64-unknown-uefi`
- **Linker errors**: Check `.cargo/config.toml` and linker scripts
- **Size issues**: Check release profile settings (`opt-level = "z"`)

### Test Issues
- **Tests hang**: Check for infinite loops, verify timeouts
- **Mock server fails**: Check Python3 installation, port conflicts
- **Logs unclear**: Increase verbosity, check QEMU serial output

## Next Steps After Testing

1. **Document Issues**: Create GitHub issues for any failures
2. **Prioritize Fixes**: Focus on boot and network first
3. **Iterate**: Fix → Test → Repeat
4. **CI/CD**: Set up automated testing pipeline (future work)

## Continuous Testing (Future)

Planned CI/CD integration:
- GitHub Actions for unit tests
- Automated QEMU tests on PR
- ISO build verification
- Cross-platform testing (x86_64, ARM64)
