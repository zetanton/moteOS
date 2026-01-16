# moteOS Product Requirements Document

**Version:** 1.0.0-draft  
**Author:** Zachary & Claude  
**Date:** January 2026  
**Status:** Draft for Review

---

## Executive Summary

**moteOS** is an ultra-lightweight, AI-native unikernel operating system written in Rust. It strips computing to its absolute essence: boot, connect, converse. The entire operating system exists for a single purpose—to provide a minimal TUI interface for interacting with large language models.

The name "mote" refers to a tiny particle of dust floating in sunlight. moteOS embodies this philosophy: the smallest possible bridge between human thought and artificial intelligence.

### Core Philosophy

> *"Everything that is not the AI interface is bloat."*

- No shell
- No filesystem (beyond config)
- No package manager
- No multi-tasking
- No GUI
- Just you and the model

---

## Goals & Non-Goals

### Goals

1. **Minimal footprint** — Target <10MB bootable image
2. **Instant boot** — Cold boot to prompt in <3 seconds on modern hardware
3. **Universal hardware** — Run on virtually anything with a network connection
4. **Multi-provider** — Support major LLM APIs (OpenAI, Anthropic, xAI)
5. **Offline-capable** — Connect to local inference servers (Ollama, vLLM)
6. **Secure by default** — TLS-only connections, minimal attack surface
7. **Beautiful TUI** — Clean, responsive, distraction-free interface

### Non-Goals (v1)

1. Persistent storage / conversation history
2. File management or document editing
3. Multi-user support
4. Graphical user interface
5. Running local models on-device
6. Plugin or extension system
7. Web browsing beyond API calls

---

## Target Platforms

### Architecture Support

| Architecture | Priority | Boot Methods |
|--------------|----------|--------------|
| x86_64 | P0 (primary) | UEFI, Legacy BIOS |
| ARM64 (AArch64) | P0 (primary) | UEFI |

### Deployment Targets

| Target | Priority | Notes |
|--------|----------|-------|
| Physical hardware | P0 | Primary target |
| QEMU/KVM | P0 | Development & testing |
| VirtualBox | P1 | Common VM platform |
| VMware | P2 | Enterprise VMs |
| Cloud (EC2, GCP) | P3 | Future consideration |

### Network Hardware

| Type | Priority | Implementation |
|------|----------|----------------|
| Wired Ethernet | P0 | virtio-net, Intel e1000, Realtek RTL8139 |
| WiFi | P0 | Must support common chipsets (see Network section) |

---

## System Architecture

### High-Level Design

```
┌─────────────────────────────────────────────────────────────────┐
│                        moteOS Unikernel                         │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │   TUI App   │  │ LLM Client  │  │    Config Manager       │  │
│  │  (ratatui)  │  │  (reqwest)  │  │                         │  │
│  └──────┬──────┘  └──────┬──────┘  └────────────┬────────────┘  │
│         │                │                      │               │
│  ┌──────┴────────────────┴──────────────────────┴────────────┐  │
│  │                    Application Layer                       │  │
│  └──────┬────────────────┬──────────────────────┬────────────┘  │
│         │                │                      │               │
│  ┌──────┴──────┐  ┌──────┴──────┐  ┌───────────┴────────────┐  │
│  │  TLS/HTTPS  │  │   TCP/IP    │  │    Memory Allocator    │  │
│  │  (rustls)   │  │  (smoltcp)  │  │      (linked_list)     │  │
│  └──────┬──────┘  └──────┬──────┘  └───────────┬────────────┘  │
│         │                │                      │               │
│  ┌──────┴────────────────┴──────────────────────┴────────────┐  │
│  │                    Hardware Abstraction                    │  │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌──────────────┐  │  │
│  │  │ Console │  │ Network │  │  Timer  │  │   WiFi/WPA   │  │  │
│  │  │  (VGA/  │  │ Drivers │  │         │  │   Handler    │  │  │
│  │  │  UEFI)  │  │         │  │         │  │              │  │  │
│  │  └─────────┘  └─────────┘  └─────────┘  └──────────────┘  │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                     Boot Layer                             │  │
│  │            (UEFI / Multiboot2 / BIOS)                      │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### Component Breakdown

#### Boot Layer
- **UEFI Boot**: Primary method for modern systems using `uefi-rs` crate
- **Multiboot2**: For legacy BIOS via GRUB using `multiboot2` crate
- **Responsibilities**: Memory map acquisition, framebuffer setup, jump to kernel

#### Hardware Abstraction Layer (HAL)

| Component | Crate/Implementation | Purpose |
|-----------|---------------------|---------|
| Console | Custom framebuffer driver | Text output, TUI rendering |
| Network (Wired) | Custom drivers for virtio/e1000/rtl8139 | Ethernet connectivity |
| Network (WiFi) | Custom 802.11 + WPA2 supplicant | Wireless connectivity |
| Timer | HPET / APIC / ARM Generic Timer | Timeouts, delays |
| Interrupts | Custom IDT/GIC setup | Event handling |
| Memory | `linked_list_allocator` | Heap allocation |

#### Network Stack

| Layer | Implementation | Notes |
|-------|----------------|-------|
| WiFi MAC | Custom 802.11 driver | Targeting common chipsets |
| WPA2/WPA3 | Custom or `embedded-wpa` | PSK authentication |
| IP/TCP/UDP | `smoltcp` | Battle-tested embedded TCP/IP |
| DNS | `smoltcp` or custom | Simple resolver |
| DHCP | `smoltcp` | Automatic configuration |
| TLS 1.3 | `rustls` | Secure connections |
| HTTP/1.1 | Custom minimal client | API communication |

#### Application Layer

| Component | Implementation | Purpose |
|-----------|----------------|---------|
| TUI Framework | `ratatui` (no_std port or custom) | Interface rendering |
| LLM Client | Custom HTTP/JSON | API communication |
| Config | TOML parser (minimal) | Settings management |
| Input | PS/2 + USB HID | Keyboard handling |

---

## LLM Provider Support

### Supported Providers (v1)

| Provider | API Endpoint | Default Model | Priority |
|----------|--------------|---------------|----------|
| OpenAI | api.openai.com | gpt-4o | P0 (default) |
| Anthropic | api.anthropic.com | claude-sonnet-4-20250514 | P0 |
| Groq | api.groq.com | llama-3.3-70b-versatile | P0 |
| xAI (Grok) | api.x.ai | grok-2 | P0 |
| **Local (Bundled)** | localhost | SmolLM-360M | P0 |
| Local (Ollama) | configurable | configurable | P1 |
| Local (vLLM) | configurable | configurable | P1 |

### Bundled Local Model

moteOS includes a bundled small language model for fully offline operation. This makes moteOS a **true standalone AI OS** that requires no internet connection for basic functionality.

**Bundled Model: SmolLM-360M-Instruct (Q4_K_M quantization)**

| Property | Value |
|----------|-------|
| Parameters | 360M |
| Quantization | Q4_K_M (4-bit) |
| Size on disk | ~200MB |
| RAM required | ~400MB |
| Quality | Basic assistant tasks, coding help, Q&A |
| License | Apache 2.0 |

**Why SmolLM-360M:**
- Smallest model that's still genuinely useful
- Apache 2.0 licensed (no restrictions)
- Runs on virtually any hardware (even 1GB RAM systems)
- GGUF format with excellent tooling
- Trained by HuggingFace with good instruction following

**Alternative bundled models (user-selectable at build time):**
- SmolLM-135M (~75MB) — Ultra minimal, lower quality
- SmolLM-1.7B (~1GB) — Better quality, needs more RAM
- Qwen2.5-0.5B (~300MB) — Good multilingual support
- TinyLlama-1.1B (~600MB) — Popular, well-tested

**Local Inference Engine:**
The OS includes a minimal GGUF inference runtime written in Rust, based on `llama.cpp` architecture but compiled directly into the kernel. No external runtime needed.

### API Client Requirements

1. **Streaming support** — All providers must stream responses token-by-token
2. **Error handling** — Graceful handling of rate limits, network errors, auth failures
3. **Model selection** — User can switch models within provider
4. **System prompts** — Optional system prompt configuration
5. **Temperature/params** — Basic parameter adjustment

### Authentication Flow

```
┌─────────────────────────────────────────────────────────────┐
│                     moteOS Boot Sequence                     │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  1. Hardware init                                           │
│           ↓                                                 │
│  2. Check for saved config (in boot partition/CMOS)         │
│           ↓                                                 │
│  3. Config found? ──No──→ Prompt for WiFi/API keys          │
│           │                        ↓                        │
│          Yes                 Save to config                 │
│           │                        │                        │
│           ↓                        ↓                        │
│  4. Connect to network                                      │
│           ↓                                                 │
│  5. Validate API key (test request)                         │
│           ↓                                                 │
│  6. Launch TUI ←────────────────────                        │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## User Interface Specification

### Color Design System

moteOS uses a carefully designed color palette optimized for readability, aesthetic appeal, and semantic meaning. Colors are specified in both true color (24-bit) for modern terminals and fallback 256-color/16-color modes.

#### Dark Theme (Default)

```
┌─────────────────────────────────────────────────────────────────────────┐
│  DARK THEME COLOR PALETTE                                               │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  Background Layers:                                                     │
│  ┌─────────┐                                                            │
│  │ #0D1117 │  Base background (deep space blue-black)                   │
│  │ #161B22 │  Elevated surfaces (cards, input areas)                    │
│  │ #21262D │  Borders, separators                                       │
│  │ #30363D │  Hover states, subtle highlights                           │
│  └─────────┘                                                            │
│                                                                         │
│  Text Hierarchy:                                                        │
│  ┌─────────┐                                                            │
│  │ #F0F6FC │  Primary text (high contrast)                              │
│  │ #C9D1D9 │  Secondary text (messages)                                 │
│  │ #8B949E │  Tertiary text (timestamps, hints)                         │
│  │ #484F58 │  Disabled/placeholder text                                 │
│  └─────────┘                                                            │
│                                                                         │
│  Accent Colors (Semantic):                                              │
│  ┌─────────┐                                                            │
│  │ #58A6FF │  Primary accent (links, focus, user messages)              │
│  │ #7EE787 │  Success (connected, valid)                                │
│  │ #FFA657 │  Warning (rate limit, slow connection)                     │
│  │ #FF7B72 │  Error (disconnected, auth failed)                         │
│  │ #A371F7 │  AI/Assistant messages accent                              │
│  │ #79C0FF │  Code/monospace text                                       │
│  │ #FFA198 │  Highlight/search match                                    │
│  └─────────┘                                                            │
│                                                                         │
│  Provider Brand Colors:                                                 │
│  ┌─────────┐                                                            │
│  │ #10A37F │  OpenAI (green)                                            │
│  │ #D4A574 │  Anthropic (tan/clay)                                      │
│  │ #F55036 │  Groq (orange-red)                                         │
│  │ #FFFFFF │  xAI (white)                                               │
│  │ #7C3AED │  Local/Bundled (purple)                                    │
│  └─────────┘                                                            │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

#### Light Theme

```
┌─────────────────────────────────────────────────────────────────────────┐
│  LIGHT THEME COLOR PALETTE                                              │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  Background Layers:                                                     │
│  ┌─────────┐                                                            │
│  │ #FFFFFF │  Base background                                           │
│  │ #F6F8FA │  Elevated surfaces                                         │
│  │ #D0D7DE │  Borders, separators                                       │
│  │ #E6EBF1 │  Hover states                                              │
│  └─────────┘                                                            │
│                                                                         │
│  Text Hierarchy:                                                        │
│  ┌─────────┐                                                            │
│  │ #1F2328 │  Primary text                                              │
│  │ #424A53 │  Secondary text                                            │
│  │ #656D76 │  Tertiary text                                             │
│  │ #8C959F │  Disabled text                                             │
│  └─────────┘                                                            │
│                                                                         │
│  Accent Colors:                                                         │
│  ┌─────────┐                                                            │
│  │ #0969DA │  Primary accent                                            │
│  │ #1A7F37 │  Success                                                   │
│  │ #9A6700 │  Warning                                                   │
│  │ #CF222E │  Error                                                     │
│  │ #8250DF │  AI/Assistant accent                                       │
│  │ #0550AE │  Code text                                                 │
│  └─────────┘                                                            │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

#### UI Element Color Mapping

| Element | Dark Theme | Light Theme | Notes |
|---------|------------|-------------|-------|
| **Window background** | #0D1117 | #FFFFFF | Base layer |
| **Header bar** | #161B22 | #F6F8FA | Contains title, status |
| **Header text** | #F0F6FC | #1F2328 | "moteOS v0.1.0" |
| **Status indicator (connected)** | #7EE787 | #1A7F37 | ● symbol |
| **Status indicator (error)** | #FF7B72 | #CF222E | ● symbol |
| **Provider name** | Provider color | Provider color | Branded |
| **Chat area background** | #0D1117 | #FFFFFF | Main content |
| **User message bubble** | #161B22 | #F6F8FA | Subtle elevation |
| **User message border** | #58A6FF | #0969DA | Left accent line |
| **User message text** | #C9D1D9 | #424A53 | Readable |
| **User label ("You")** | #58A6FF | #0969DA | Matches border |
| **Assistant message bubble** | #161B22 | #F6F8FA | Subtle elevation |
| **Assistant message border** | #A371F7 | #8250DF | Left accent line |
| **Assistant message text** | #C9D1D9 | #424A53 | Readable |
| **Assistant label** | #A371F7 | #8250DF | "Assistant" or model name |
| **Code blocks background** | #21262D | #E6EBF1 | Distinct from text |
| **Code text** | #79C0FF | #0550AE | Monospace |
| **Input area background** | #161B22 | #F6F8FA | Elevated |
| **Input border (unfocused)** | #30363D | #D0D7DE | Subtle |
| **Input border (focused)** | #58A6FF | #0969DA | Highlighted |
| **Input text** | #F0F6FC | #1F2328 | High contrast |
| **Placeholder text** | #484F58 | #8C959F | "Type a message..." |
| **Footer/hotkey bar** | #161B22 | #F6F8FA | Elevated |
| **Hotkey brackets** | #8B949E | #656D76 | [F1] |
| **Hotkey label** | #C9D1D9 | #424A53 | "Help" |
| **Scrollbar track** | #21262D | #E6EBF1 | Subtle |
| **Scrollbar thumb** | #484F58 | #8C959F | Visible but unobtrusive |
| **Selection highlight** | #58A6FF40 | #0969DA30 | 25% opacity |
| **Search match** | #FFA19840 | #CF222E30 | Highlighted text |

#### Syntax Highlighting (Code Blocks)

| Token Type | Dark Theme | Light Theme |
|------------|------------|-------------|
| **Keyword** | #FF7B72 | #CF222E |
| **String** | #A5D6FF | #0A3069 |
| **Number** | #79C0FF | #0550AE |
| **Comment** | #8B949E | #656D76 |
| **Function** | #D2A8FF | #8250DF |
| **Type/Class** | #FFA657 | #953800 |
| **Variable** | #FFA657 | #953800 |
| **Operator** | #FF7B72 | #CF222E |
| **Punctuation** | #C9D1D9 | #424A53 |
| **Constant** | #79C0FF | #0550AE |

#### Modal/Popup Colors

| Element | Dark Theme | Light Theme |
|---------|------------|-------------|
| **Overlay background** | #0D111780 (50% opacity) | #1F232880 |
| **Modal background** | #161B22 | #FFFFFF |
| **Modal border** | #30363D | #D0D7DE |
| **Modal title** | #F0F6FC | #1F2328 |
| **Selected item** | #21262D | #E6EBF1 |
| **Selected item indicator** | #58A6FF | #0969DA |
| **Radio button (active)** | #58A6FF | #0969DA |
| **Radio button (inactive)** | #484F58 | #8C959F |
| **Checkmark** | #7EE787 | #1A7F37 |

### Visual Design Examples

### Main Chat Interface

```
┌──────────────────────────────────────────────────────────────────────────┐
│ moteOS v0.1.0                              │ OpenAI/gpt-4o │ ● Connected │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─ You ─────────────────────────────────────────────────────────────┐   │
│  │ Explain the concept of unikernels in simple terms.                │   │
│  └───────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│  ┌─ Assistant ───────────────────────────────────────────────────────┐   │
│  │ A unikernel is like a custom-built race car versus a general      │   │
│  │ purpose sedan. Traditional operating systems (like Linux or       │   │
│  │ Windows) are designed to do everything—run any program, support   │   │
│  │ any hardware, handle multiple users.                              │   │
│  │                                                                   │   │
│  │ A unikernel strips away everything except exactly what your       │   │
│  │ single application needs. It compiles your app directly with      │   │
│  │ just the OS components it requires into one tiny, fast package.   │   │
│  │                                                                   │   │
│  │ Benefits:                                                         │   │
│  │ • Tiny size (megabytes instead of gigabytes)                      │   │
│  │ • Boots in milliseconds                                           │   │
│  │ • Smaller attack surface (less code = fewer vulnerabilities)      │   │
│  │ • No unnecessary overhead                                         │   │
│  │                                                                   │   │
│  │ The tradeoff: you can only run that one application. But if       │   │
│  │ that's all you need (like an AI chat interface), it's perfect.    │   │
│  └───────────────────────────────────────────────────────────────────┘   │
│                                                                          │
├──────────────────────────────────────────────────────────────────────────┤
│ │                                                                        │
└──────────────────────────────────────────────────────────────────────────┘
 [F1 Help] [F2 Provider] [F3 Model] [F4 Config] [F9 New Chat] [F10 Shutdown]
```

### Configuration Screen (F4)

```
┌──────────────────────────────────────────────────────────────────────────┐
│                            moteOS Configuration                          │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─ Network ─────────────────────────────────────────────────────────┐   │
│  │  Connection:  WiFi                                                │   │
│  │  SSID:        MyNetwork                                           │   │
│  │  Status:      Connected (192.168.1.42)                            │   │
│  │  [Tab to edit]                                                    │   │
│  └───────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│  ┌─ API Keys ────────────────────────────────────────────────────────┐   │
│  │  OpenAI:      sk-...4f2k [set]                                    │   │
│  │  Anthropic:   sk-...8x2m [set]                                    │   │
│  │  xAI:         xai-...3j9 [set]                                    │   │
│  │  [Tab to edit]                                                    │   │
│  └───────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│  ┌─ Local Inference ─────────────────────────────────────────────────┐   │
│  │  Ollama URL:  http://192.168.1.100:11434                          │   │
│  │  vLLM URL:    [not set]                                           │   │
│  │  [Tab to edit]                                                    │   │
│  └───────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│  ┌─ Display ─────────────────────────────────────────────────────────┐   │
│  │  Theme:       [Dark] Light                                        │   │
│  │  Font Size:   Normal                                              │   │
│  └───────────────────────────────────────────────────────────────────┘   │
│                                                                          │
├──────────────────────────────────────────────────────────────────────────┤
│                    [Enter: Edit Field]  [Esc: Back to Chat]              │
└──────────────────────────────────────────────────────────────────────────┘
```

### Provider Selection (F2)

```
┌─────────────────────────────────────┐
│       Select LLM Provider           │
├─────────────────────────────────────┤
│                                     │
│   [●] OpenAI          ✓ configured  │
│   [ ] Anthropic       ✓ configured  │
│   [ ] Groq            ✓ configured  │
│   [ ] xAI (Grok)      ✓ configured  │
│   [ ] Local (SmolLM)  ✓ bundled     │
│   [ ] Ollama          ✗ not set     │
│   [ ] vLLM            ✗ not set     │
│                                     │
├─────────────────────────────────────┤
│  [Enter: Select]  [Esc: Cancel]     │
└─────────────────────────────────────┘
```

### Model Selection (F3)

```
┌─────────────────────────────────────┐
│     Select Model (OpenAI)           │
├─────────────────────────────────────┤
│                                     │
│   [●] gpt-4o                        │
│   [ ] gpt-4o-mini                   │
│   [ ] gpt-4-turbo                   │
│   [ ] o1                            │
│   [ ] o1-mini                       │
│   [ ] o3-mini                       │
│                                     │
├─────────────────────────────────────┤
│  [Enter: Select]  [Esc: Cancel]     │
└─────────────────────────────────────┘
```

```
┌─────────────────────────────────────┐
│     Select Model (Anthropic)        │
├─────────────────────────────────────┤
│                                     │
│   [●] claude-sonnet-4-20250514      │
│   [ ] claude-opus-4-20250514        │
│   [ ] claude-haiku-3-5-20241022     │
│                                     │
├─────────────────────────────────────┤
│  [Enter: Select]  [Esc: Cancel]     │
└─────────────────────────────────────┘
```

```
┌─────────────────────────────────────┐
│     Select Model (Groq)             │
├─────────────────────────────────────┤
│                                     │
│   [●] llama-3.3-70b-versatile       │
│   [ ] llama-3.1-8b-instant          │
│   [ ] mixtral-8x7b-32768            │
│   [ ] gemma2-9b-it                  │
│                                     │
├─────────────────────────────────────┤
│  [Enter: Select]  [Esc: Cancel]     │
└─────────────────────────────────────┘
```

```
┌─────────────────────────────────────┐
│     Select Model (xAI)              │
├─────────────────────────────────────┤
│                                     │
│   [●] grok-2                        │
│   [ ] grok-2-mini                   │
│                                     │
├─────────────────────────────────────┤
│  [Enter: Select]  [Esc: Cancel]     │
└─────────────────────────────────────┘
```

```
┌─────────────────────────────────────┐
│     Select Model (Local)            │
├─────────────────────────────────────┤
│                                     │
│   [●] SmolLM-360M (bundled)         │
│                                     │
├─────────────────────────────────────┤
│  [Enter: Select]  [Esc: Cancel]     │
└─────────────────────────────────────┘
```

### First-Boot Setup Wizard

```
┌──────────────────────────────────────────────────────────────────────────┐
│                                                                          │
│                                                                          │
│                              ·  moteOS  ·                                 │
│                                                                          │
│                     a speck of intelligence                              │
│                                                                          │
│                                                                          │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│                         Welcome to moteOS                                │
│                                                                          │
│          Let's get you connected to artificial intelligence.             │
│                                                                          │
│                                                                          │
│   Step 1 of 3: Network Configuration                                     │
│                                                                          │
│   ┌─ Available Networks ─────────────────────────────────────────────┐   │
│   │  > MyHomeWiFi          ████████░░  -42 dBm   WPA2                │   │
│   │    Neighbors5G         ██████░░░░  -58 dBm   WPA2                │   │
│   │    CoffeeShop          ████░░░░░░  -71 dBm   Open                │   │
│   │    [R] Refresh                                                   │   │
│   └──────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│   Password: ••••••••••••                                                 │
│                                                                          │
│                                                                          │
├──────────────────────────────────────────────────────────────────────────┤
│              [Enter: Connect]  [Tab: Next Field]  [Esc: Skip]            │
└──────────────────────────────────────────────────────────────────────────┘
```

```
┌──────────────────────────────────────────────────────────────────────────┐
│                                                                          │
│                              ·  moteOS  ·                                 │
│                                                                          │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   Step 2 of 3: API Configuration                                         │
│                                                                          │
│   Enter API keys for cloud providers (optional - local model included).  │
│                                                                          │
│   ┌──────────────────────────────────────────────────────────────────┐   │
│   │  OpenAI API Key:                                                 │   │
│   │  > sk-________________________________________________          │   │
│   │                                                                  │   │
│   │  Anthropic API Key:                                              │   │
│   │  > sk-ant-____________________________________________          │   │
│   │                                                                  │   │
│   │  Groq API Key:                                                   │   │
│   │  > gsk_________________________________________________          │   │
│   │                                                                  │   │
│   │  xAI API Key:                                                    │   │
│   │  > xai-_______________________________________________          │   │
│   └──────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│   Tip: Get API keys from:                                                │
│   • OpenAI: platform.openai.com/api-keys                                 │
│   • Anthropic: console.anthropic.com/settings/keys                       │
│   • Groq: console.groq.com/keys                                          │
│   • xAI: console.x.ai                                                    │
│                                                                          │
│   No keys? Press Enter to use bundled local model (SmolLM-360M)          │
│                                                                          │
├──────────────────────────────────────────────────────────────────────────┤
│                [Enter: Validate & Continue]  [Tab: Next Field]           │
└──────────────────────────────────────────────────────────────────────────┘
```

```
┌──────────────────────────────────────────────────────────────────────────┐
│                                                                          │
│                              ·  moteOS  ·                                 │
│                                                                          │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   Step 3 of 3: Ready                                                     │
│                                                                          │
│                                                                          │
│                          ✓ Network connected                             │
│                          ✓ OpenAI configured                             │
│                          ✓ Anthropic configured                          │
│                                                                          │
│                                                                          │
│   Default provider: OpenAI (gpt-4o)                                      │
│                                                                          │
│   Press Enter to begin.                                                  │
│                                                                          │
│                                                                          │
│                                                                          │
│                                                                          │
│                                                                          │
│                                                                          │
│                                                                          │
├──────────────────────────────────────────────────────────────────────────┤
│                              [Enter: Start]                              │
└──────────────────────────────────────────────────────────────────────────┘
```

---

## Keyboard Interface

### Global Shortcuts

| Key | Action |
|-----|--------|
| F1 | Help overlay |
| F2 | Provider selection |
| F3 | Model selection |
| F4 | Configuration |
| F9 | New conversation (clear chat) |
| F10 | Shutdown |
| Ctrl+C | Cancel current generation |
| Ctrl+L | Clear screen |

### Chat Input

| Key | Action |
|-----|--------|
| Enter | Send message (single-line mode) |
| Shift+Enter | Newline in message |
| Ctrl+Enter | Send message (multi-line mode) |
| Up/Down | Scroll chat history |
| PgUp/PgDn | Scroll chat (fast) |
| Home/End | Jump to top/bottom of chat |

---

## Configuration Storage

### Config Location Strategy

Since moteOS is ephemeral by design, configuration must persist across reboots via one of:

1. **EFI Variables** (UEFI systems) — Store in NVRAM, survives reboot
2. **Boot partition file** — Small FAT32 partition with `mote.toml`
3. **CMOS/RTC RAM** (Legacy BIOS) — Limited but persistent

### Config File Format

```toml
# mote.toml - moteOS Configuration

[network]
type = "wifi"  # "wifi" | "ethernet"
ssid = "MyNetwork"
# Password stored encrypted or in separate secure storage
wifi_password_hash = "..."

[network.static]
# Optional static IP config (DHCP if not present)
ip = "192.168.1.42"
gateway = "192.168.1.1"
dns = ["8.8.8.8", "1.1.1.1"]

[providers.openai]
api_key_encrypted = "..."
default_model = "gpt-4o"

[providers.anthropic]
api_key_encrypted = "..."
default_model = "claude-sonnet-4-20250514"

[providers.groq]
api_key_encrypted = "..."
default_model = "llama-3.3-70b-versatile"

[providers.xai]
api_key_encrypted = "..."
default_model = "grok-2"

[providers.ollama]
endpoint = "http://192.168.1.100:11434"
default_model = "llama3.3"

[providers.local]
enabled = true
model_path = "/bundled/smollm-360m.gguf"

[preferences]
default_provider = "openai"
theme = "dark"
temperature = 0.7
stream_responses = true
```

---

## Network Implementation Details

### WiFi Support Strategy

WiFi is the most complex component. moteOS targets chipsets common across both legacy and modern hardware.

#### Supported WiFi Chipsets

**Legacy Hardware (Pre-2015):**

| Chipset | Interface | Common In | Driver Complexity |
|---------|-----------|-----------|-------------------|
| RTL8187L | USB 2.0 | Early USB dongles | Medium (well-documented) |
| RTL8188CUS | USB 2.0 | Cheap dongles, RPi | Medium |
| RTL8192CU | USB 2.0 | Common dongles | Medium |
| AR9271 | USB 2.0 | Atheros dongles | Medium (open firmware) |
| RALink RT3070 | USB 2.0 | Many brands | Medium |
| RALink RT5370 | USB 2.0 | Very common, cheap | Medium |
| Atheros ATH9K | PCIe/Mini-PCIe | Laptops 2008-2014 | High (but open source) |

**Modern Hardware (2015+):**

| Chipset | Interface | Common In | Driver Complexity |
|---------|-----------|-----------|-------------------|
| RTL8811AU | USB 3.0 | AC dongles | Medium |
| RTL8812AU | USB 3.0 | Popular AC dongles | Medium |
| RTL8821CU | USB 2.0/3.0 | Modern cheap dongles | Medium |
| MT7601U | USB 2.0 | Very common, well-documented | Low |
| MT7612U | USB 3.0 | AC dongles | Medium |
| MT7921 | USB/PCIe | 2021+ laptops | High (newer) |
| Intel AX200/AX210 | PCIe | 2019+ laptops | High (complex firmware) |

#### v1 Implementation Priority

```
Tier 1 (Must Have - Widest Compatibility):
├── MT7601U      — Extremely common, simple, cheap dongles
├── RTL8188CUS   — Ubiquitous legacy dongle
├── RT5370       — Very common legacy dongle
└── RTL8812AU    — Popular modern AC dongle

Tier 2 (Should Have):
├── AR9271       — Open firmware, Atheros ecosystem
├── RTL8821CU    — Modern budget dongles
└── MT7612U      — Modern AC support

Tier 3 (Nice to Have):
├── ATH9K        — Many built-in laptop cards
├── Intel AX200  — Modern Intel laptops
└── MT7921       — Newest MediaTek
```

#### Recommended User Hardware

For guaranteed compatibility, moteOS documentation will recommend:
- **Budget:** Any MT7601U-based dongle (~$5-10)
- **Performance:** RTL8812AU-based dongle (~$15-25)
- **Legacy:** RT5370 or RTL8188CUS

### Network Driver Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     Network Driver Layer                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────────────────┐  │
│  │   Ethernet   │  │   USB WiFi   │  │   virtio-net (VMs)     │  │
│  │   (e1000,    │  │  (RTL8188,   │  │                        │  │
│  │   RTL8139)   │  │   MT7601U)   │  │                        │  │
│  └──────┬───────┘  └──────┬───────┘  └───────────┬────────────┘  │
│         │                 │                      │               │
│         │          ┌──────┴───────┐              │               │
│         │          │   802.11     │              │               │
│         │          │   + WPA2     │              │               │
│         │          └──────┬───────┘              │               │
│         │                 │                      │               │
│         └─────────────────┼──────────────────────┘               │
│                           │                                      │
│                    ┌──────┴───────┐                              │
│                    │   smoltcp    │                              │
│                    │  (TCP/IP)    │                              │
│                    └──────┬───────┘                              │
│                           │                                      │
│                    ┌──────┴───────┐                              │
│                    │   rustls     │                              │
│                    │   (TLS)      │                              │
│                    └──────────────┘                              │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Security Considerations

### Threat Model

moteOS has a minimal attack surface by design:

| Attack Vector | Mitigation |
|---------------|------------|
| Network attacks | TLS-only outbound, no listening ports |
| Malicious input | No shell, no code execution |
| Physical access | API keys encrypted at rest |
| Memory attacks | Rust memory safety, no unsafe unless required |
| Supply chain | Minimal dependencies, audited crates |

### Security Features

1. **TLS 1.3 only** — No fallback to older protocols
2. **Certificate validation** — Proper CA verification
3. **API key encryption** — Keys encrypted with hardware-derived key where possible
4. **No inbound connections** — Firewall everything by default
5. **Read-only execution** — Code runs from read-only memory

### Out of Scope (v1)

- Secure boot chain verification
- Full disk encryption
- TPM integration
- Attestation

---

## Build & Distribution

### Build Requirements

- Rust nightly toolchain
- `cargo-make` or custom build script
- QEMU for testing
- `xorriso` for ISO creation

### Build Targets

```bash
# Development (QEMU x86_64 UEFI)
make run-qemu-x64-uefi

# Development (QEMU x86_64 BIOS)
make run-qemu-x64-bios

# Development (QEMU ARM64)
make run-qemu-arm64

# Release ISO (x86_64, both UEFI and BIOS)
make iso-x64

# Release ISO (ARM64)
make iso-arm64

# Combined multi-arch ISO
make iso-all
```

### Output Artifacts

| Artifact | Description | Target Size |
|----------|-------------|-------------|
| `moteos-x64.iso` | Bootable ISO for x86_64 | <10MB |
| `moteos-arm64.iso` | Bootable ISO for ARM64 | <10MB |
| `moteos.img` | Raw disk image | <10MB |

### Distribution

1. **GitHub Releases** — Primary distribution
2. **Direct download** — moteos.org (future)
3. **Source build** — Full reproducible build instructions

---

## Licensing

### Recommended License: **MIT**

**Rationale:**
- Maximum permissiveness for adoption
- Compatible with all dependency licenses
- Simple and well-understood
- Allows commercial use and modification

### Dependency License Compatibility

All dependencies must be compatible with MIT:
- ✅ MIT
- ✅ Apache 2.0
- ✅ BSD-2/BSD-3
- ✅ ISC
- ⚠️ MPL 2.0 (file-level copyleft, acceptable)
- ❌ GPL (avoid)
- ❌ LGPL (avoid in static linking context)

---

## Development Architecture for Parallel Execution

### Agent Team Structure

moteOS development is structured for **parallel execution by multiple AI agents**. Each workstream is independent with clearly defined interfaces.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        moteOS AGENT ORCHESTRATION                           │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │   AGENT 1   │  │   AGENT 2   │  │   AGENT 3   │  │      AGENT 4        │ │
│  │    BOOT     │  │   NETWORK   │  │     TUI     │  │    LLM CLIENTS      │ │
│  │             │  │             │  │             │  │                     │ │
│  │ - UEFI boot │  │ - virtio    │  │ - ratatui   │  │ - OpenAI client     │ │
│  │ - BIOS boot │  │ - smoltcp   │  │ - Colors    │  │ - Anthropic client  │ │
│  │ - Memory    │  │ - DHCP/DNS  │  │ - Input     │  │ - Groq client       │ │
│  │ - Interrupts│  │ - TLS       │  │ - Widgets   │  │ - xAI client        │ │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘ │
│         │                │                │                    │            │
│  ┌──────┴────────────────┴────────────────┴────────────────────┴──────────┐ │
│  │                         SHARED INTERFACES                              │ │
│  │  - Trait definitions    - Error types    - Config structs              │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │   AGENT 5   │  │   AGENT 6   │  │   AGENT 7   │  │      AGENT 8        │ │
│  │    WIFI     │  │  INFERENCE  │  │   CONFIG    │  │    INTEGRATION      │ │
│  │             │  │             │  │             │  │                     │ │
│  │ - USB stack │  │ - GGUF load │  │ - TOML parse│  │ - Glue code         │ │
│  │ - 802.11    │  │ - Tokenizer │  │ - EFI vars  │  │ - Main loop         │ │
│  │ - WPA2      │  │ - Inference │  │ - Wizard UI │  │ - Testing           │ │
│  │ - Drivers   │  │ - Sampling  │  │ - Encryption│  │ - ISO build         │ │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────────────┘ │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Workstream Definitions

---

#### WORKSTREAM 1: Boot & Core (Agent 1)

**Owner:** Boot Agent  
**Dependencies:** None (foundation)  
**Deliverable:** Bootable kernel that reaches main()

**Tasks:**
```
□ Project scaffolding (Cargo workspace structure)
□ x86_64 UEFI boot entry using uefi-rs
□ x86_64 BIOS boot entry using bootloader crate
□ ARM64 UEFI boot entry
□ Memory allocator setup (linked_list_allocator)
□ Global allocator registration
□ Interrupt descriptor table (x86_64)
□ GIC setup (ARM64)
□ Basic panic handler
□ Framebuffer acquisition from bootloader
□ Basic text output to framebuffer
□ Timer setup (HPET/APIC/ARM Generic Timer)
```

**Interface Contract:**
```rust
// boot/src/lib.rs
pub struct BootInfo {
    pub framebuffer: FramebufferInfo,
    pub memory_map: MemoryMap,
    pub rsdp_addr: Option<PhysAddr>,  // For ACPI
}

pub struct FramebufferInfo {
    pub addr: *mut u8,
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub pixel_format: PixelFormat,
}

/// Entry point called by boot layer
pub fn kernel_main(boot_info: BootInfo) -> ! {
    // Hand off to integration agent's code
}
```

---

#### WORKSTREAM 2: Network Stack (Agent 2)

**Owner:** Network Agent  
**Dependencies:** Boot (memory allocator)  
**Deliverable:** Working TCP/TLS connections

**Tasks:**
```
□ virtio-net driver for QEMU
□ e1000 driver for VMs/older hardware
□ RTL8139 driver for legacy hardware
□ smoltcp integration
□ DHCP client
□ DNS resolver
□ TCP connection management
□ rustls integration (TLS 1.3)
□ Certificate verification
□ HTTP/1.1 client (minimal, for API calls)
□ Connection pooling
□ Timeout handling
```

**Interface Contract:**
```rust
// network/src/lib.rs
pub trait NetworkDriver: Send {
    fn send(&mut self, packet: &[u8]) -> Result<(), NetError>;
    fn receive(&mut self) -> Result<Option<Vec<u8>>, NetError>;
    fn mac_address(&self) -> [u8; 6];
}

pub struct NetworkStack {
    // Internal smoltcp state
}

impl NetworkStack {
    pub fn new(driver: Box<dyn NetworkDriver>) -> Self;
    pub fn poll(&mut self) -> Result<(), NetError>;
    pub fn dhcp_acquire(&mut self) -> Result<IpConfig, NetError>;
    pub fn dns_resolve(&mut self, hostname: &str) -> Result<IpAddr, NetError>;
    pub fn tcp_connect(&mut self, addr: SocketAddr) -> Result<TcpHandle, NetError>;
    pub fn tls_connect(&mut self, hostname: &str, port: u16) -> Result<TlsStream, NetError>;
}

pub struct HttpClient {
    network: NetworkStack,
}

impl HttpClient {
    pub fn post_json(&mut self, url: &str, body: &str, headers: &[(&str, &str)]) 
        -> Result<HttpResponse, HttpError>;
    pub fn post_streaming(&mut self, url: &str, body: &str, headers: &[(&str, &str)]) 
        -> Result<StreamingResponse, HttpError>;
}
```

---

#### WORKSTREAM 3: TUI Framework (Agent 3)

**Owner:** TUI Agent  
**Dependencies:** Boot (framebuffer)  
**Deliverable:** Complete UI rendering system

**Tasks:**
```
□ Framebuffer text renderer
□ Font embedding (PSF or custom bitmap font)
□ Color system implementation (24-bit + fallbacks)
□ Dark theme implementation
□ Light theme implementation
□ Text wrapping and scrolling
□ Input field widget
□ Message bubble widget
□ Modal/popup system
□ Status bar widget
□ Hotkey bar widget
□ Provider selector widget
□ Model selector widget
□ Configuration form widget
□ Setup wizard flow
□ Syntax highlighting engine
□ Markdown rendering (bold, italic, code, headers)
□ Scrollbar rendering
□ Focus management
□ Keyboard navigation
```

**Interface Contract:**
```rust
// tui/src/lib.rs
pub struct Theme {
    pub background: Color,
    pub surface: Color,
    pub border: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub accent_primary: Color,
    pub accent_success: Color,
    pub accent_warning: Color,
    pub accent_error: Color,
    pub accent_assistant: Color,
    // ... full color spec
}

pub static DARK_THEME: Theme = Theme { /* ... */ };
pub static LIGHT_THEME: Theme = Theme { /* ... */ };

pub struct Screen {
    framebuffer: FramebufferInfo,
    theme: &'static Theme,
}

impl Screen {
    pub fn clear(&mut self);
    pub fn draw_text(&mut self, x: usize, y: usize, text: &str, color: Color);
    pub fn draw_box(&mut self, rect: Rect, style: BoxStyle);
    pub fn draw_message(&mut self, msg: &Message, rect: Rect);
    pub fn draw_input(&mut self, input: &InputState, rect: Rect, focused: bool);
    pub fn draw_modal(&mut self, modal: &Modal);
    pub fn draw_code_block(&mut self, code: &str, lang: Option<&str>, rect: Rect);
    pub fn present(&mut self);  // Flush to screen
}

pub enum AppScreen {
    Chat(ChatState),
    Config(ConfigState),
    ProviderSelect(SelectState),
    ModelSelect(SelectState),
    SetupWizard(WizardState),
    Help,
}
```

---

#### WORKSTREAM 4: LLM API Clients (Agent 4)

**Owner:** LLM Agent  
**Dependencies:** Network (HTTP client)  
**Deliverable:** Working clients for all cloud providers

**Tasks:**
```
□ Common LLM trait definition
□ Message/conversation types
□ OpenAI API client
□ OpenAI streaming (SSE parsing)
□ Anthropic API client
□ Anthropic streaming
□ Groq API client
□ Groq streaming
□ xAI API client
□ xAI streaming
□ Error handling (rate limits, auth, network)
□ Retry logic with backoff
□ Model enumeration per provider
□ Token counting (approximate)
```

**Interface Contract:**
```rust
// llm/src/lib.rs
pub struct Message {
    pub role: Role,
    pub content: String,
}

pub enum Role {
    System,
    User,
    Assistant,
}

pub struct GenerationConfig {
    pub temperature: f32,
    pub max_tokens: Option<usize>,
    pub stop_sequences: Vec<String>,
}

#[async_trait]  // or callback-based for no_std
pub trait LlmProvider {
    fn name(&self) -> &str;
    fn models(&self) -> &[ModelInfo];
    fn complete(
        &mut self,
        messages: &[Message],
        model: &str,
        config: &GenerationConfig,
        on_token: impl FnMut(&str),  // Streaming callback
    ) -> Result<CompletionResult, LlmError>;
}

pub struct OpenAiClient { /* ... */ }
pub struct AnthropicClient { /* ... */ }
pub struct GroqClient { /* ... */ }
pub struct XaiClient { /* ... */ }

impl LlmProvider for OpenAiClient { /* ... */ }
impl LlmProvider for AnthropicClient { /* ... */ }
impl LlmProvider for GroqClient { /* ... */ }
impl LlmProvider for XaiClient { /* ... */ }
```

---

#### WORKSTREAM 5: WiFi Stack (Agent 5)

**Owner:** WiFi Agent  
**Dependencies:** Boot (USB stack needed), Network (smoltcp interface)  
**Deliverable:** WiFi connectivity with WPA2

**Tasks:**
```
□ USB host controller driver (xHCI for modern, UHCI/OHCI for legacy)
□ USB device enumeration
□ USB bulk/interrupt transfers
□ MT7601U driver
□ RTL8188CUS driver
□ RT5370 driver (stretch)
□ RTL8812AU driver (stretch)
□ 802.11 frame parsing
□ 802.11 scanning (probe request/response)
□ 802.11 authentication
□ 802.11 association
□ WPA2-PSK 4-way handshake
□ CCMP encryption/decryption
□ Integration with smoltcp as network driver
```

**Interface Contract:**
```rust
// wifi/src/lib.rs
pub struct WifiNetwork {
    pub ssid: String,
    pub bssid: [u8; 6],
    pub signal_strength: i8,  // dBm
    pub security: SecurityType,
    pub channel: u8,
}

pub enum SecurityType {
    Open,
    WPA2Personal,
    WPA3Personal,
}

pub struct WifiDriver {
    // Internal state
}

impl WifiDriver {
    pub fn scan(&mut self) -> Result<Vec<WifiNetwork>, WifiError>;
    pub fn connect(&mut self, ssid: &str, password: &str) -> Result<(), WifiError>;
    pub fn disconnect(&mut self) -> Result<(), WifiError>;
    pub fn status(&self) -> WifiStatus;
}

impl NetworkDriver for WifiDriver {
    // Implement network trait for smoltcp integration
}
```

---

#### WORKSTREAM 6: Local Inference Engine (Agent 6)

**Owner:** Inference Agent  
**Dependencies:** Boot (memory), (optional: SIMD intrinsics)  
**Deliverable:** GGUF model loading and inference

**Tasks:**
```
□ GGUF file format parser
□ Model weight loading
□ Tokenizer (BPE, from GGUF metadata)
□ Tensor operations (matmul, softmax, layernorm, etc.)
□ Transformer forward pass
□ KV cache management
□ Sampling (temperature, top-p, top-k)
□ Memory-efficient inference (streaming weights if needed)
□ SIMD optimization (SSE4.2/AVX2 for x86, NEON for ARM)
□ Quantized matmul (Q4_K_M support)
□ Integration as LlmProvider
□ Bundle SmolLM-360M weights
```

**Interface Contract:**
```rust
// inference/src/lib.rs
pub struct LocalModel {
    weights: ModelWeights,
    tokenizer: Tokenizer,
    kv_cache: KvCache,
}

impl LocalModel {
    pub fn load_gguf(data: &[u8]) -> Result<Self, ModelError>;
    pub fn generate(
        &mut self,
        prompt: &str,
        config: &GenerationConfig,
        on_token: impl FnMut(&str),
    ) -> Result<String, InferenceError>;
}

impl LlmProvider for LocalModel {
    // Implement common trait
}
```

---

#### WORKSTREAM 7: Configuration System (Agent 7)

**Owner:** Config Agent  
**Dependencies:** Boot (EFI services), TUI (wizard screens)  
**Deliverable:** Persistent configuration across reboots

**Tasks:**
```
□ TOML parser (minimal, no_std compatible)
□ Config struct definitions
□ EFI variable read/write (UEFI systems)
□ Boot partition config file (BIOS fallback)
□ API key encryption/decryption
□ First-boot detection
□ Setup wizard state machine
□ Network configuration UI
□ API key input UI
□ Preference editing UI
□ Config validation
□ Migration between config versions
```

**Interface Contract:**
```rust
// config/src/lib.rs
pub struct MoteConfig {
    pub network: NetworkConfig,
    pub providers: ProviderConfigs,
    pub preferences: Preferences,
}

pub struct NetworkConfig {
    pub connection_type: ConnectionType,
    pub wifi_ssid: Option<String>,
    pub wifi_password_encrypted: Option<Vec<u8>>,
    pub static_ip: Option<IpConfig>,
}

pub struct ProviderConfigs {
    pub openai: Option<ProviderConfig>,
    pub anthropic: Option<ProviderConfig>,
    pub groq: Option<ProviderConfig>,
    pub xai: Option<ProviderConfig>,
    pub ollama: Option<LocalProviderConfig>,
}

pub struct Preferences {
    pub default_provider: String,
    pub default_model: String,
    pub theme: ThemeChoice,
    pub temperature: f32,
}

pub trait ConfigStorage {
    fn load(&self) -> Result<Option<MoteConfig>, ConfigError>;
    fn save(&mut self, config: &MoteConfig) -> Result<(), ConfigError>;
}

pub struct EfiConfigStorage;
pub struct FileConfigStorage;

impl ConfigStorage for EfiConfigStorage { /* ... */ }
impl ConfigStorage for FileConfigStorage { /* ... */ }
```

---

#### WORKSTREAM 8: Integration & Build (Agent 8)

**Owner:** Integration Agent  
**Dependencies:** All other workstreams  
**Deliverable:** Working bootable ISO

**Tasks:**
```
□ Cargo workspace setup
□ Main kernel entry point
□ Component initialization sequence
□ Main event loop
□ Keyboard input handling (PS/2)
□ USB keyboard support
□ Screen update loop
□ Network polling integration
□ Provider switching logic
□ Conversation state management
□ New chat functionality
□ Shutdown sequence
□ ISO generation (xorriso)
□ QEMU test scripts
□ CI/CD pipeline
□ README and documentation
```

**Interface Contract:**
```rust
// kernel/src/main.rs
#![no_std]
#![no_main]

use boot::BootInfo;
use network::NetworkStack;
use tui::{Screen, AppScreen};
use llm::LlmProvider;
use config::MoteConfig;

static mut GLOBAL_STATE: Option<KernelState> = None;

struct KernelState {
    screen: Screen,
    network: Option<NetworkStack>,
    config: MoteConfig,
    current_provider: Box<dyn LlmProvider>,
    app_screen: AppScreen,
    conversation: Vec<Message>,
}

#[no_mangle]
pub extern "C" fn kernel_main(boot_info: BootInfo) -> ! {
    // Initialize all subsystems
    // Enter main loop
    loop {
        handle_input();
        poll_network();
        update_screen();
    }
}
```

---

### File Structure

```
moteos/
├── Cargo.toml                 # Workspace root
├── Makefile                   # Build orchestration
├── README.md
├── LICENSE                    # MIT
│
├── boot/                      # WORKSTREAM 1
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── uefi/
│       │   ├── mod.rs
│       │   └── x86_64.rs
│       │   └── aarch64.rs
│       ├── bios/
│       │   └── mod.rs
│       ├── memory.rs
│       └── interrupts.rs
│
├── network/                   # WORKSTREAM 2
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── drivers/
│       │   ├── mod.rs
│       │   ├── virtio.rs
│       │   ├── e1000.rs
│       │   └── rtl8139.rs
│       ├── stack.rs           # smoltcp wrapper
│       ├── dhcp.rs
│       ├── dns.rs
│       ├── tls.rs
│       └── http.rs
│
├── wifi/                      # WORKSTREAM 5
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── usb/
│       │   ├── mod.rs
│       │   ├── xhci.rs
│       │   └── device.rs
│       ├── drivers/
│       │   ├── mod.rs
│       │   ├── mt7601u.rs
│       │   └── rtl8188.rs
│       ├── ieee80211.rs
│       └── wpa2.rs
│
├── tui/                       # WORKSTREAM 3
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── framebuffer.rs
│       ├── font.rs
│       ├── colors.rs
│       ├── theme.rs
│       ├── widgets/
│       │   ├── mod.rs
│       │   ├── text.rs
│       │   ├── input.rs
│       │   ├── message.rs
│       │   ├── modal.rs
│       │   ├── status.rs
│       │   └── hotkeys.rs
│       ├── syntax.rs          # Code highlighting
│       ├── markdown.rs
│       └── screens/
│           ├── mod.rs
│           ├── chat.rs
│           ├── config.rs
│           ├── wizard.rs
│           └── help.rs
│
├── llm/                       # WORKSTREAM 4
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── types.rs
│       ├── streaming.rs       # SSE parser
│       ├── providers/
│       │   ├── mod.rs
│       │   ├── openai.rs
│       │   ├── anthropic.rs
│       │   ├── groq.rs
│       │   └── xai.rs
│       └── error.rs
│
├── inference/                 # WORKSTREAM 6
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── gguf.rs            # Format parser
│       ├── tokenizer.rs
│       ├── tensor.rs
│       ├── ops.rs             # Tensor operations
│       ├── transformer.rs
│       ├── sampling.rs
│       └── simd/
│           ├── mod.rs
│           ├── x86.rs
│           └── arm.rs
│
├── config/                    # WORKSTREAM 7
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── toml.rs            # Minimal parser
│       ├── types.rs
│       ├── storage/
│       │   ├── mod.rs
│       │   ├── efi.rs
│       │   └── file.rs
│       ├── crypto.rs          # Key encryption
│       └── wizard.rs
│
├── kernel/                    # WORKSTREAM 8
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── init.rs
│       ├── input.rs
│       ├── event_loop.rs
│       └── panic.rs
│
├── shared/                    # Common types (all agents)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── error.rs
│       └── types.rs
│
├── assets/
│   ├── fonts/
│   │   └── terminus.psf       # Bitmap font
│   └── models/
│       └── smollm-360m.gguf   # Bundled model (~200MB)
│
└── tools/
    ├── build-iso.sh
    ├── run-qemu-x64.sh
    ├── run-qemu-arm64.sh
    └── test-all.sh
```

---

### Parallel Execution Timeline (TONIGHT)

```
Hour 0-1: Project Setup
├── Agent 8: Cargo workspace, Makefile, CI setup
├── Agent 1: Begin UEFI boot scaffolding
├── Agent 3: Begin framebuffer/font code
└── All: Clone shared types crate

Hour 1-3: Foundation Sprint
├── Agent 1: Complete UEFI boot → framebuffer working
├── Agent 2: virtio-net driver + smoltcp skeleton
├── Agent 3: Text rendering + color system
├── Agent 4: LLM trait definitions + OpenAI client
├── Agent 6: GGUF parser + tensor ops skeleton
├── Agent 7: Config types + TOML parser
└── Agent 8: Integration scaffolding

Hour 3-5: Core Functionality
├── Agent 1: BIOS boot + memory allocator polish
├── Agent 2: DHCP + DNS + basic TCP working
├── Agent 3: Input widget + message bubbles
├── Agent 4: Anthropic + Groq + xAI clients
├── Agent 5: USB enumeration + MT7601U start
├── Agent 6: Tokenizer + transformer forward pass
├── Agent 7: EFI storage + wizard state machine
└── Agent 8: Main loop + keyboard handling

Hour 5-7: Integration Phase
├── Agent 2: TLS working → can hit APIs
├── Agent 3: Full chat screen + modals
├── Agent 4: Streaming working for all providers
├── Agent 5: WiFi scanning working
├── Agent 6: Basic inference working
├── Agent 7: Full wizard flow
└── Agent 8: Wire everything together

Hour 7-8: Polish & Ship
├── All: Bug fixes
├── Agent 8: ISO generation
├── Agent 8: QEMU testing
└── Agent 8: README/docs

DELIVERABLE: Bootable moteOS ISO by end of night
```

---

### Inter-Agent Communication Protocol

Agents should commit to the shared repository frequently with clear commit messages:

```
[boot] UEFI entry point working, framebuffer acquired
[network] virtio-net driver sending/receiving packets
[tui] Text rendering at 60fps, colors implemented
[llm] OpenAI streaming working end-to-end
[wifi] MT7601U recognized and initialized
[inference] GGUF loading, tokenizer working
[config] EFI variables read/write working
[integration] Main loop running, keyboard input working
```

**Blocking issues** should be communicated immediately:
```
[BLOCKED] network: Need memory allocator from boot team
[BLOCKED] llm: Need HTTP client from network team
[UNBLOCKED] boot: Memory allocator merged, network can proceed
```

---

## Success Metrics

### v0.1.0 Release Criteria

| Metric | Target |
|--------|--------|
| Boot image size (without model) | <10MB |
| Boot image size (with SmolLM-360M) | <250MB |
| Cold boot to prompt | <5 seconds |
| Cloud providers supported | 4 (OpenAI, Anthropic, Groq, xAI) |
| Local inference | Bundled SmolLM-360M |
| Architecture support | x86_64 + ARM64 |
| Boot method support | UEFI + Legacy BIOS |
| Network support | Ethernet + WiFi (Tier 1 chipsets) |
| TUI | Full color, syntax highlighting, markdown |

### MVP Criteria (Tonight's Build)

| Feature | Required | Nice-to-Have |
|---------|----------|--------------|
| UEFI boot (x86_64) | ✓ | |
| BIOS boot (x86_64) | | ✓ |
| ARM64 boot | | ✓ |
| Framebuffer text output | ✓ | |
| Color TUI with themes | ✓ | |
| virtio-net (QEMU) | ✓ | |
| e1000 driver | | ✓ |
| WiFi (MT7601U) | | ✓ |
| DHCP + DNS | ✓ | |
| TLS 1.3 | ✓ | |
| OpenAI API | ✓ | |
| Anthropic API | ✓ | |
| Groq API | ✓ | |
| xAI API | | ✓ |
| Streaming responses | ✓ | |
| Local inference (SmolLM) | | ✓ |
| Config persistence | | ✓ |
| Setup wizard | ✓ (basic) | Full wizard |
| Syntax highlighting | ✓ | |
| Bootable ISO | ✓ | |

### Stretch Goals

- [ ] Local Ollama/vLLM support
- [ ] Raspberry Pi official support
- [ ] Markdown rendering in TUI
- [ ] Syntax highlighting for code blocks
- [ ] Multi-conversation tabs
- [ ] Voice input (USB audio)

---

## Open Questions (Resolved)

| Question | Decision |
|----------|----------|
| WiFi chipset selection | Tier 1: MT7601U, RTL8188CUS, RT5370, RTL8812AU |
| ARM64 priority | Concurrent development, same code structure |
| Local inference | P0 - Bundled SmolLM-360M in image |
| Code rendering | Full syntax highlighting included |
| License | MIT |

## Decisions for Tonight's Build

1. **QEMU-first**: Get everything working on QEMU x86_64 UEFI first, then expand
2. **Streaming priority**: All cloud APIs must stream - no blocking requests
3. **Minimal viable inference**: Local model can be slow, but must work
4. **Config fallback**: If EFI vars fail, prompt on every boot (acceptable for MVP)
5. **Partial WiFi**: WiFi is nice-to-have for tonight; Ethernet is required

---

## Appendix A: Key Rust Crates

| Crate | Purpose | no_std? |
|-------|---------|---------|
| `bootloader` | x86_64 bootloader | N/A |
| `uefi` | UEFI services | Yes |
| `multiboot2` | BIOS multiboot | Yes |
| `x86_64` | CPU-specific abstractions | Yes |
| `aarch64` | ARM64 abstractions | Yes |
| `smoltcp` | TCP/IP stack | Yes |
| `rustls` | TLS implementation | Partial |
| `linked_list_allocator` | Heap allocator | Yes |
| `spin` | Spinlocks | Yes |
| `log` | Logging facade | Yes |
| `serde` | Serialization | Yes* |
| `toml` | Config parsing | No** |

*with `no_std` feature  
**may need custom minimal parser

---

## Appendix B: Reference Projects

| Project | Relevance |
|---------|-----------|
| [blog_os](https://os.phil-opp.com/) | Rust OS tutorial, excellent foundation |
| [Redox OS](https://redox-os.org/) | Rust microkernel, reference implementation |
| [Theseus OS](https://github.com/theseus-os/Theseus) | Research Rust OS |
| [smoltcp examples](https://github.com/smoltcp-rs/smoltcp) | Network stack usage |
| [MOROS](https://github.com/vinc/moros) | Minimal Rust OS |
| [hermit-os](https://github.com/hermitcore/hermit-os) | Rust unikernel |

---

## Document History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0-draft | Jan 16, 2026 | Zachary & Claude | Initial PRD |
| 1.1.0 | Jan 16, 2026 | Zachary & Claude | Added Groq, bundled model, color spec, parallel agent architecture |

---

## Critical Path for Tonight

```
BLOCKING CHAIN (must complete in order):
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[Boot Agent] UEFI boot + memory allocator
     │
     ▼
[Network Agent] virtio-net + smoltcp + DHCP
     │
     ▼
[Network Agent] TLS working
     │
     ▼
[LLM Agent] First successful API call
     │
     ▼
[TUI Agent] Can display streamed response
     │
     ▼
[Integration Agent] Full loop working
     │
     ▼
[Integration Agent] ISO boots in QEMU
     │
     ▼
                 🎉 MVP COMPLETE 🎉


PARALLEL WORK (can happen anytime):
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[TUI Agent] Colors, themes, widgets
[LLM Agent] Additional providers
[Inference Agent] Local model loading
[Config Agent] Persistence layer
[WiFi Agent] USB + drivers
```

---

*"A mote of dust, suspended in a sunbeam, connecting you to infinite intelligence."*
