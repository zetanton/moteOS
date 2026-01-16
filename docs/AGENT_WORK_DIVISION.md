# Agent Work Division & Vibe Kanban Setup Guide

## Overview

This document explains how to set up Vibe Kanban for the moteOS project and how to divide work among multiple AI agents based on the 8 workstreams defined in the PRD.

---

## Vibe Kanban Setup

### Prerequisites

1. **Node.js ≥ 18** (LTS recommended)
2. **Git** installed and configured
3. **GitHub CLI** (`gh`) authenticated
4. **Coding agent** authenticated (Cursor CLI, Claude API, etc.)

### Installation & Launch

```bash
# From project root
cd /Users/zetanton/Development/TADASERV/moteOS

# Launch Vibe Kanban (will auto-download on first run)
npx vibe-kanban

# Or specify a port
PORT=8080 npx vibe-kanban
```

Vibe Kanban will:
- Download automatically on first run (~27MB)
- Bind to an available port (or use PORT env var)
- Open in your browser automatically
- Print the URL (e.g., `http://127.0.0.1:58507`)

### Project Setup in Vibe Kanban

1. **Create Project**: Use "From existing git repository"
   - Select the `moteOS` repository
   - This allows VK to see code and history

2. **Configure Project Settings**:
   - **Setup Script**: `cargo build` (when Rust code exists)
   - **Dev Server**: Not applicable (unikernel)
   - **Cleanup Script**: `cargo clean` (optional)
   - **Build Script**: `make iso-x64` (for ISO generation)

3. **Agent Integration**:
   - Install and authenticate your coding agents
   - Configure agent profiles/variants in global settings
   - Set default agent for tasks

### GitHub Integration

- VK will use your existing `gh` authentication
- Tasks can be converted to PRs automatically
- PR monitoring runs every 60 seconds

---

## Agent Work Division Strategy

### Architecture Overview

moteOS is divided into **8 independent workstreams** that can be developed in parallel:

```
┌─────────────────────────────────────────────────────────────┐
│                    moteOS Workstreams                       │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  [1] Boot & Core      [2] Network Stack   [3] TUI         │
│  [4] LLM Clients      [5] WiFi Stack      [6] Inference    │
│  [7] Config System    [8] Integration                      │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Workstream Dependencies

```
Boot & Core (1)
    │
    ├──> Network Stack (2) ──> LLM Clients (4)
    │
    ├──> TUI (3)
    │
    ├──> WiFi Stack (5) ──> Network Stack (2)
    │
    ├──> Inference (6)
    │
    └──> Config System (7)

Integration (8) ──> All workstreams
```

**Critical Path** (must complete in order):
1. Boot & Core → Network Stack → LLM Clients → TUI → Integration

**Parallel Work** (can happen simultaneously):
- TUI (after Boot)
- WiFi Stack (after Boot)
- Inference (after Boot)
- Config System (after Boot)

---

## Workstream 1: Boot & Core

### Agent Assignment
- **Agent Name**: `boot-agent` or `Agent-1-Boot`
- **Priority**: P0 (Foundation - blocks all others)

### Tasks Breakdown

#### Phase 1: Foundation (Week 1)
- [ ] **Task 1.1**: Project scaffolding (Cargo workspace structure)
  - Create workspace `Cargo.toml`
  - Set up crate structure: `boot/`, `shared/`
  - Configure `no_std` for all crates
  
- [ ] **Task 1.2**: x86_64 UEFI boot entry
  - Implement UEFI entry point using `uefi-rs`
  - Acquire framebuffer via Graphics Output Protocol
  - Get memory map and exit boot services
  - Jump to `kernel_main()`

- [ ] **Task 1.3**: Memory allocator setup
  - Integrate `linked_list_allocator`
  - Initialize heap from memory map
  - Register global allocator
  - Test allocation/deallocation

#### Phase 2: Core Systems (Week 1-2)
- [ ] **Task 1.4**: Interrupt setup (x86_64)
  - Create Interrupt Descriptor Table (IDT)
  - Implement timer interrupt handler
  - Implement keyboard interrupt handler
  - Implement panic handler

- [ ] **Task 1.5**: Framebuffer interface
  - Create `FramebufferInfo` struct
  - Implement safe wrapper functions
  - Basic pixel writing functions
  - Test text rendering

- [ ] **Task 1.6**: Timer setup
  - Configure HPET or APIC timer
  - Implement `sleep_ms()` function
  - Test timer interrupts

#### Phase 3: Additional Boot Methods (Week 2)
- [ ] **Task 1.7**: x86_64 BIOS boot (Multiboot2)
  - Create multiboot2 header
  - Implement BIOS entry point
  - Parse multiboot info structure

- [ ] **Task 1.8**: ARM64 UEFI boot
  - Implement ARM64 UEFI entry
  - Configure MMU
  - ARM64-specific initialization

### Deliverables
- Bootable kernel that reaches `kernel_main()`
- Memory allocator functional
- Framebuffer accessible
- Timer operational
- `BootInfo` struct passed to kernel

### Interface Contract
See `docs/TECHNICAL_SPECIFICATIONS.md` Section 3.1 for detailed interface definitions.

---

## Workstream 2: Network Stack

### Agent Assignment
- **Agent Name**: `network-agent` or `Agent-2-Network`
- **Priority**: P0 (Required for LLM clients)
- **Dependencies**: Boot & Core (memory allocator)

### Tasks Breakdown

#### Phase 1: Drivers (Week 1-2)
- [ ] **Task 2.1**: virtio-net driver
  - PCI device discovery
  - Virtio queue setup
  - Packet transmission/reception
  - Interrupt handling

- [ ] **Task 2.2**: e1000 driver
  - PCI device discovery
  - Register access (MMIO)
  - TX/RX descriptor rings
  - Interrupt handling

- [ ] **Task 2.3**: RTL8139 driver (optional, P1)
  - PCI device discovery
  - I/O port access
  - Buffer management

#### Phase 2: Network Stack (Week 2)
- [ ] **Task 2.4**: smoltcp integration
  - Create `NetworkStack` struct
  - Implement `NetworkDriver` trait
  - Set up interface, neighbor cache, routes
  - Implement polling mechanism

- [ ] **Task 2.5**: DHCP client
  - Create DHCP socket
  - Implement Discover/Offer/Request/Ack flow
  - Configure interface with IP, gateway, DNS

- [ ] **Task 2.6**: DNS resolver
  - Create UDP socket
  - Send DNS queries (A records)
  - Parse DNS responses

#### Phase 3: TLS & HTTP (Week 2-3)
- [ ] **Task 2.7**: TCP connection management
  - Implement `tcp_connect()`
  - Implement `tcp_send()` / `tcp_receive()`
  - Handle connection states

- [ ] **Task 2.8**: TLS 1.3 support
  - Integrate `rustls` or implement minimal TLS
  - TLS handshake
  - Certificate verification
  - Record layer encryption/decryption

- [ ] **Task 2.9**: HTTP/1.1 client
  - Parse URLs
  - Build HTTP requests
  - Parse HTTP responses
  - Handle headers and body

### Deliverables
- Working TCP/IP stack via smoltcp
- TLS 1.3 connections
- HTTP/1.1 client for API calls
- DHCP and DNS resolution

### Interface Contract
See `docs/TECHNICAL_SPECIFICATIONS.md` Section 3.2 for detailed interface definitions.

---

## Workstream 3: TUI Framework

### Agent Assignment
- **Agent Name**: `tui-agent` or `Agent-3-TUI`
- **Priority**: P0 (User interface)
- **Dependencies**: Boot & Core (framebuffer)

### Tasks Breakdown

#### Phase 1: Rendering Foundation (Week 1)
- [ ] **Task 3.1**: Framebuffer rendering
  - Implement pixel writing functions
  - Implement rectangle filling
  - Implement line drawing
  - Safe wrapper functions

- [ ] **Task 3.2**: Font system
  - Load PSF font format
  - Implement glyph rendering
  - Text rendering function

- [ ] **Task 3.3**: Color system
  - Implement `Color` struct
  - Color palette definitions
  - Color conversion functions

#### Phase 2: Theme System (Week 1)
- [ ] **Task 3.4**: Dark theme implementation
  - Define all colors from PRD
  - Create `DARK_THEME` constant
  - Test color rendering

- [ ] **Task 3.5**: Light theme implementation
  - Define all colors from PRD
  - Create `LIGHT_THEME` constant
  - Theme switching

#### Phase 3: Widgets (Week 2)
- [ ] **Task 3.6**: Base widget system
  - Define `Widget` trait
  - Implement `Screen` struct
  - Widget rendering infrastructure

- [ ] **Task 3.7**: Input widget
  - Text input with cursor
  - Character insertion/deletion
  - Focus management

- [ ] **Task 3.8**: Message widget
  - User/Assistant message bubbles
  - Text wrapping
  - Timestamp display

- [ ] **Task 3.9**: Modal system
  - Modal overlay
  - Button handling
  - Focus management

#### Phase 4: Advanced Features (Week 2-3)
- [ ] **Task 3.10**: Syntax highlighting
  - Tokenizer for code
  - Color mapping per token type
  - Code block rendering

- [ ] **Task 3.11**: Markdown rendering
  - Parse markdown (bold, italic, code, headers)
  - Render markdown elements
  - Integration with message widget

- [ ] **Task 3.12**: Chat screen
  - Message list with scrolling
  - Input area
  - Status bar
  - Hotkey bar

### Deliverables
- Complete TUI rendering system
- Dark and light themes
- All widgets functional
- Syntax highlighting
- Markdown rendering

### Interface Contract
See `docs/TECHNICAL_SPECIFICATIONS.md` Section 3.3 for detailed interface definitions.

---

## Workstream 4: LLM API Clients

### Agent Assignment
- **Agent Name**: `llm-agent` or `Agent-4-LLM`
- **Priority**: P0 (Core functionality)
- **Dependencies**: Network Stack (HTTP client)

### Tasks Breakdown

#### Phase 1: Common Infrastructure (Week 1)
- [ ] **Task 4.1**: Common types
  - `Message` struct
  - `Role` enum
  - `GenerationConfig` struct
  - `ModelInfo` struct

- [ ] **Task 4.2**: LLM provider trait
  - Define `LlmProvider` trait
  - Error types
  - Streaming callback interface

#### Phase 2: Provider Implementations (Week 1-2)
- [ ] **Task 4.3**: OpenAI client
  - API endpoint configuration
  - Request/response handling
  - SSE streaming parser
  - Error handling

- [ ] **Task 4.4**: Anthropic client
  - API endpoint configuration
  - Request/response handling
  - SSE streaming parser
  - Error handling

- [ ] **Task 4.5**: Groq client
  - API endpoint configuration
  - Request/response handling
  - SSE streaming parser

- [ ] **Task 4.6**: xAI client
  - API endpoint configuration
  - Request/response handling
  - SSE streaming parser

#### Phase 3: Polish (Week 2)
- [ ] **Task 4.7**: Error handling & retries
  - Rate limit handling
  - Retry logic with backoff
  - Network error handling
  - Auth error handling

- [ ] **Task 4.8**: Model enumeration
  - List models per provider
  - Model selection UI integration

### Deliverables
- Working clients for all 4 cloud providers
- Streaming support for all
- Error handling and retries
- Model enumeration

### Interface Contract
See `docs/TECHNICAL_SPECIFICATIONS.md` Section 3.4 for detailed interface definitions.

---

## Workstream 5: WiFi Stack

### Agent Assignment
- **Agent Name**: `wifi-agent` or `Agent-5-WiFi`
- **Priority**: P1 (Nice-to-have for MVP)
- **Dependencies**: Boot & Core (USB stack needed)

### Tasks Breakdown

#### Phase 1: USB Stack (Week 2-3)
- [ ] **Task 5.1**: USB host controller (xHCI)
  - PCI device discovery
  - Register setup
  - Device enumeration

- [ ] **Task 5.2**: USB device interface
  - Control transfers
  - Bulk transfers
  - Interrupt transfers

#### Phase 2: WiFi Driver (Week 3)
- [ ] **Task 5.3**: MT7601U driver
  - USB device detection
  - Register initialization
  - Frame transmission/reception

- [ ] **Task 5.4**: 802.11 frame handling
  - Frame parsing
  - Management/Control/Data frames
  - Frame construction

#### Phase 3: WPA2 (Week 3-4)
- [ ] **Task 5.5**: WiFi scanning
  - Probe requests
  - Beacon parsing
  - Network list

- [ ] **Task 5.6**: WPA2-PSK authentication
  - 4-way handshake
  - PMK derivation
  - CCMP encryption/decryption

- [ ] **Task 5.7**: Network driver integration
  - Implement `NetworkDriver` trait
  - Integration with smoltcp

### Deliverables
- WiFi connectivity with WPA2
- Network scanning
- Integration with network stack

### Interface Contract
See `docs/TECHNICAL_SPECIFICATIONS.md` Section 3.5 for detailed interface definitions.

---

## Workstream 6: Local Inference Engine

### Agent Assignment
- **Agent Name**: `inference-agent` or `Agent-6-Inference`
- **Priority**: P0 (Bundled model is core feature)
- **Dependencies**: Boot & Core (memory)

### Tasks Breakdown

#### Phase 1: GGUF Parser (Week 2)
- [ ] **Task 6.1**: GGUF file format parser
  - Parse GGUF header
  - Extract metadata
  - Load tensor data

- [ ] **Task 6.2**: Model weights loading
  - Weight structure definitions
  - Quantized tensor support (Q4_K_M)
  - Memory-efficient loading

#### Phase 2: Tokenizer (Week 2)
- [ ] **Task 6.3**: BPE tokenizer
  - Load vocab from GGUF
  - Encode/decode functions
  - Special token handling

#### Phase 3: Inference Engine (Week 3)
- [ ] **Task 6.4**: Tensor operations
  - Matrix multiplication (F32, Q4K)
  - Activation functions (SiLU, softmax)
  - Layer normalization
  - RoPE implementation

- [ ] **Task 6.5**: Transformer forward pass
  - Attention layer
  - FFN layer
  - Layer-by-layer processing

- [ ] **Task 6.6**: KV cache
  - Cache structure
  - Append operations
  - Memory management

#### Phase 4: Generation (Week 3)
- [ ] **Task 6.7**: Sampling
  - Temperature sampling
  - Top-k sampling
  - Top-p (nucleus) sampling

- [ ] **Task 6.8**: Generation loop
  - Prefill phase
  - Generation loop
  - Streaming tokens
  - Stop conditions

- [ ] **Task 6.9**: SIMD optimizations
  - SSE4.2/AVX2 for x86_64
  - NEON for ARM64
  - Quantized matmul optimization

#### Phase 5: Integration (Week 3)
- [ ] **Task 6.10**: LlmProvider implementation
  - Implement trait for local model
  - Model info
  - Streaming support

- [ ] **Task 6.11**: Bundle SmolLM-360M
  - Include model in build
  - Model loading at runtime
  - Memory management

### Deliverables
- GGUF model loading
- Transformer inference
- Token generation with streaming
- SIMD optimizations
- Bundled SmolLM-360M

### Interface Contract
See `docs/TECHNICAL_SPECIFICATIONS.md` Section 3.6 for detailed interface definitions.

---

## Workstream 7: Configuration System

### Agent Assignment
- **Agent Name**: `config-agent` or `Agent-7-Config`
- **Priority**: P1 (Can work in parallel)
- **Dependencies**: Boot & Core (EFI services)

### Tasks Breakdown

#### Phase 1: Parser & Types (Week 1)
- [ ] **Task 7.1**: TOML parser (no_std)
  - Minimal TOML parser
  - Key-value pairs
  - Nested tables
  - Arrays

- [ ] **Task 7.2**: Config types
  - `MoteConfig` struct
  - `NetworkConfig` struct
  - `ProviderConfigs` struct
  - `Preferences` struct

#### Phase 2: Storage (Week 1-2)
- [ ] **Task 7.3**: EFI variable storage
  - Read/write EFI variables
  - Config serialization
  - Config deserialization

- [ ] **Task 7.4**: File storage (fallback)
  - Boot partition access
  - File read/write
  - Error handling

#### Phase 3: Security (Week 2)
- [ ] **Task 7.5**: API key encryption
  - Key derivation
  - AES-256-GCM encryption
  - Decryption

#### Phase 4: Wizard (Week 2)
- [ ] **Task 7.6**: Setup wizard
  - State machine
  - Network configuration UI
  - API key input UI
  - Validation

### Deliverables
- Persistent configuration
- EFI variable storage
- API key encryption
- Setup wizard

### Interface Contract
See `docs/TECHNICAL_SPECIFICATIONS.md` Section 3.7 for detailed interface definitions.

---

## Workstream 8: Integration & Build

### Agent Assignment
- **Agent Name**: `integration-agent` or `Agent-8-Integration`
- **Priority**: P0 (Final assembly)
- **Dependencies**: All other workstreams

### Tasks Breakdown

#### Phase 1: Project Setup (Week 1)
- [ ] **Task 8.1**: Cargo workspace
  - Root `Cargo.toml`
  - Workspace members
  - Shared dependencies

- [ ] **Task 8.2**: Build system
  - Makefile
  - Build scripts
  - QEMU test scripts

#### Phase 2: Main Kernel (Week 3-4)
- [ ] **Task 8.3**: Kernel entry point
  - `kernel_main()` function
  - Component initialization
  - Error handling

- [ ] **Task 8.4**: Input handling
  - PS/2 keyboard
  - USB keyboard
  - Key event processing

- [ ] **Task 8.5**: Main event loop
  - Input polling
  - Network polling
  - Screen updates
  - Timer management

#### Phase 3: Integration (Week 4)
- [ ] **Task 8.6**: Component integration
  - Network + LLM clients
  - TUI + LLM streaming
  - Config + providers
  - Provider switching

- [ ] **Task 8.7**: Conversation management
  - Message history
  - New chat functionality
  - State persistence

#### Phase 4: Build & Test (Week 4)
- [ ] **Task 8.8**: ISO generation
  - UEFI boot ISO
  - BIOS boot ISO
  - ARM64 ISO

- [ ] **Task 8.9**: QEMU testing
  - Boot test
  - Network test
  - API test
  - Integration test

- [ ] **Task 8.10**: Documentation
  - README updates
  - Build instructions
  - Troubleshooting guide

### Deliverables
- Bootable ISO
- Working end-to-end system
- Build scripts
- Documentation

### Interface Contract
See `docs/TECHNICAL_SPECIFICATIONS.md` Section 3.8 for detailed interface definitions.

---

## Vibe Kanban Task Creation

### Task Naming Convention

```
[Workstream-Number] Task-Name
```

Examples:
- `[1] UEFI boot entry point`
- `[2] virtio-net driver`
- `[3] Dark theme implementation`
- `[4] OpenAI client with streaming`

### Task Labels/Tags

- `workstream-1` through `workstream-8`
- `priority-p0`, `priority-p1`
- `blocking`, `parallel`
- `foundation`, `feature`, `polish`

### Task Dependencies

In Vibe Kanban, mark dependencies:
- Task 2.x depends on Task 1.1 (memory allocator)
- Task 4.x depends on Task 2.9 (HTTP client)
- Task 8.x depends on all other workstreams

### Agent Assignment

When creating tasks:
1. Select appropriate agent profile
2. Assign to workstream-specific agent
3. Set branch base (usually `main`)
4. Use "Create & Start" to begin work

### Task Lifecycle

1. **To Do**: Task created, not started
2. **In Progress**: Agent working on task
3. **In Review**: Code complete, needs review
4. **Done**: Merged to main branch

---

## Parallel Execution Strategy

### Week 1: Foundation
- **Agent 1**: Boot & Core (foundation)
- **Agent 3**: TUI foundation (after framebuffer ready)
- **Agent 7**: Config types & parser
- **Agent 8**: Project setup

### Week 2: Core Features
- **Agent 1**: Complete boot (all methods)
- **Agent 2**: Network stack (after memory allocator)
- **Agent 3**: TUI widgets & themes
- **Agent 4**: LLM clients (after HTTP client)
- **Agent 6**: Inference engine foundation
- **Agent 7**: Config storage & wizard

### Week 3: Integration
- **Agent 2**: TLS & HTTP polish
- **Agent 3**: Advanced TUI features
- **Agent 4**: Provider polish
- **Agent 5**: WiFi stack (if time permits)
- **Agent 6**: Inference completion
- **Agent 8**: Main integration

### Week 4: Polish & Ship
- **Agent 8**: Final integration
- **All**: Bug fixes
- **Agent 8**: ISO generation & testing

---

## Communication Protocol

### Commit Messages

Use workstream tags in commit messages:

```
[boot] UEFI entry point working, framebuffer acquired
[network] virtio-net driver sending/receiving packets
[tui] Text rendering at 60fps, colors implemented
[llm] OpenAI streaming working end-to-end
```

### Blocking Issues

If blocked, communicate immediately:

```
[BLOCKED] network: Need memory allocator from boot team
[BLOCKED] llm: Need HTTP client from network team
[UNBLOCKED] boot: Memory allocator merged, network can proceed
```

### Interface Changes

If interface contract changes:
1. Update `TECHNICAL_SPECIFICATIONS.md`
2. Notify dependent workstreams
3. Update shared types in `shared/` crate

---

## Success Metrics

### MVP Criteria

- [ ] Bootable ISO (x86_64 UEFI minimum)
- [ ] Network connectivity (Ethernet minimum)
- [ ] At least 2 LLM providers working
- [ ] Basic TUI functional
- [ ] End-to-end conversation working

### v0.1.0 Release Criteria

- [ ] All 8 workstreams complete
- [ ] All 4 cloud providers working
- [ ] Local inference working
- [ ] WiFi support (Tier 1 chipsets)
- [ ] Full TUI with themes
- [ ] Configuration persistence

---

## Next Steps

1. **Launch Vibe Kanban**: `npx vibe-kanban`
2. **Create Project**: From existing git repository
3. **Create Initial Tasks**: Start with Workstream 1 tasks
4. **Assign Agents**: Begin parallel development
5. **Monitor Progress**: Use VK dashboard

---

**Last Updated**: January 2026
