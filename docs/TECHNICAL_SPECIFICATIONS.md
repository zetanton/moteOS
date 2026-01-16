# moteOS Technical Specifications

**Version:** 1.0.0  
**Date:** January 2026  
**Purpose:** Complete technical specifications for AI agent implementation

---

## Table of Contents

1. [System Overview](#system-overview)
2. [Architecture Specifications](#architecture-specifications)
3. [Workstream 1: Boot & Core](#workstream-1-boot--core)
4. [Workstream 2: Network Stack](#workstream-2-network-stack)
5. [Workstream 3: TUI Framework](#workstream-3-tui-framework)
6. [Workstream 4: LLM API Clients](#workstream-4-llm-api-clients)
7. [Workstream 5: WiFi Stack](#workstream-5-wifi-stack)
8. [Workstream 6: Local Inference Engine](#workstream-6-local-inference-engine)
9. [Workstream 7: Configuration System](#workstream-7-configuration-system)
10. [Workstream 8: Integration & Build](#workstream-8-integration--build)
11. [Build System Specifications](#build-system-specifications)
12. [Testing Requirements](#testing-requirements)
13. [Performance Targets](#performance-targets)

---

## System Overview

### Core Philosophy
- **Ultra-minimal**: Target <10MB bootable image (without bundled model)
- **Single-purpose**: Only AI chat interface, nothing else
- **No_std Rust**: All code must compile with `#![no_std]`
- **Unikernel**: Single application compiled with minimal OS components

### Target Specifications

| Metric | Target |
|--------|--------|
| Boot image size (base) | <10MB |
| Boot image size (with SmolLM-360M) | <250MB |
| Cold boot to prompt | <3 seconds |
| Memory footprint (runtime) | <512MB (without local model) |
| Memory footprint (with local model) | <1GB |

### Architecture Support

| Architecture | Boot Methods | Priority |
|--------------|--------------|----------|
| x86_64 | UEFI, Legacy BIOS | P0 |
| ARM64 (AArch64) | UEFI | P0 |

---

## Architecture Specifications

### High-Level Component Stack

```
┌─────────────────────────────────────────────────────────┐
│ Application Layer                                        │
│  - TUI (ratatui/custom)                                 │
│  - LLM Clients                                          │
│  - Config Manager                                       │
├─────────────────────────────────────────────────────────┤
│ Network Layer                                           │
│  - TLS/HTTPS (rustls)                                   │
│  - TCP/IP (smoltcp)                                     │
│  - Network Drivers (virtio, e1000, RTL8139, WiFi)      │
├─────────────────────────────────────────────────────────┤
│ Hardware Abstraction Layer                              │
│  - Console (Framebuffer)                                │
│  - Network Hardware                                     │
│  - Timer (HPET/APIC/ARM Generic Timer)                 │
│  - Interrupts (IDT/GIC)                                 │
│  - Memory Allocator (linked_list_allocator)            │
├─────────────────────────────────────────────────────────┤
│ Boot Layer                                              │
│  - UEFI (uefi-rs)                                       │
│  - Multiboot2 (BIOS)                                    │
└─────────────────────────────────────────────────────────┘
```

### Memory Layout

- **Kernel code**: Read-only, loaded by bootloader
- **Heap**: Dynamic allocation via `linked_list_allocator`
- **Stack**: Per-task stacks (single task in v1)
- **Framebuffer**: Mapped from bootloader-provided address
- **Model weights**: Loaded into heap (bundled model)

### Interrupt Handling

- **x86_64**: IDT (Interrupt Descriptor Table) with handlers for:
  - Timer interrupts (HPET/APIC)
  - Keyboard interrupts (PS/2, USB)
  - Network interrupts (device-specific)
- **ARM64**: GIC (Generic Interrupt Controller) with handlers for:
  - Timer interrupts (ARM Generic Timer)
  - Keyboard interrupts
  - Network interrupts

---

## Workstream 1: Boot & Core

### Objectives
- Bootable kernel that reaches `kernel_main()`
- Memory allocator functional
- Framebuffer accessible
- Interrupts configured
- Timer operational

### Implementation Requirements

#### 1.1 Project Structure

```
boot/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── uefi/
    │   ├── mod.rs
    │   ├── x86_64.rs
    │   └── aarch64.rs
    ├── bios/
    │   └── mod.rs
    ├── memory.rs
    ├── interrupts.rs
    ├── framebuffer.rs
    └── timer.rs
```

#### 1.2 UEFI Boot (x86_64)

**Crate**: `uefi` (with `no_std` feature)

**Entry Point**:
```rust
#[no_mangle]
pub extern "efiapi" fn efi_main(
    image_handle: uefi::Handle,
    system_table: *mut uefi::table::SystemTable<uefi::table::Runtime>,
) -> uefi::Status {
    // Initialize UEFI services
    // Acquire framebuffer
    // Get memory map
    // Exit boot services
    // Call kernel_main()
}
```

**Required Operations**:
1. Initialize UEFI boot services
2. Locate Graphics Output Protocol (GOP) for framebuffer
3. Get memory map via `BootServices::get_memory_map()`
4. Exit boot services (required before using memory allocator)
5. Set up page tables (identity mapping for initial memory)
6. Jump to `kernel_main()`

**Framebuffer Acquisition**:
- Query GOP for available modes
- Select highest resolution mode (or 1024x768 minimum)
- Get framebuffer base address, width, height, stride, pixel format
- Store in `FramebufferInfo` struct

#### 1.3 BIOS Boot (x86_64)

**Crate**: `bootloader` or `multiboot2`

**Multiboot2 Header** (in separate bootloader binary):
```rust
#[repr(C, packed)]
struct Multiboot2Header {
    magic: u32,      // 0xE85250D6
    architecture: u32, // 0 (i386)
    header_length: u32,
    checksum: u32,
    // tags...
}
```

**Entry Point**:
```rust
#[no_mangle]
pub extern "C" fn _start(multiboot_info: *const Multiboot2Info) -> ! {
    // Parse multiboot info
    // Get framebuffer info
    // Get memory map
    // Set up page tables
    // Call kernel_main()
}
```

#### 1.4 ARM64 UEFI Boot

**Entry Point**:
```rust
#[no_mangle]
pub extern "efiapi" fn efi_main(
    image_handle: uefi::Handle,
    system_table: *mut uefi::table::SystemTable<uefi::table::Runtime>,
) -> uefi::Status {
    // Similar to x86_64 but with ARM64-specific setup
    // Configure MMU
    // Get framebuffer
    // Exit boot services
    // Call kernel_main()
}
```

#### 1.5 Memory Allocator

**Crate**: `linked_list_allocator`

**Setup**:
```rust
use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init_heap(heap_start: usize, heap_size: usize) {
    unsafe {
        ALLOCATOR.lock().init(heap_start, heap_size);
    }
}
```

**Memory Map Processing**:
- Parse memory map from bootloader
- Identify usable RAM regions
- Reserve kernel code region
- Reserve framebuffer region
- Allocate heap from largest contiguous region
- Minimum heap size: 64MB

#### 1.6 Interrupt Setup

**x86_64 IDT**:
```rust
use x86_64::structures::idt::InterruptDescriptorTable;

static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

pub fn init_idt() {
    IDT.breakpoint.set_handler_fn(breakpoint_handler);
    IDT.double_fault.set_handler_fn(double_fault_handler);
    // Timer, keyboard handlers
    IDT.load();
}
```

**ARM64 GIC**:
- Configure GICv2 or GICv3
- Set up timer interrupt
- Set up keyboard interrupt
- Enable interrupts

#### 1.7 Framebuffer Interface

**Struct Definition**:
```rust
#[derive(Debug, Clone, Copy)]
pub struct FramebufferInfo {
    pub base: *mut u8,
    pub width: usize,
    pub height: usize,
    pub stride: usize,  // Bytes per row
    pub pixel_format: PixelFormat,
}

#[derive(Debug, Clone, Copy)]
pub enum PixelFormat {
    Rgb,      // 24-bit RGB
    Bgr,      // 24-bit BGR
    Rgba,     // 32-bit RGBA
    Bgra,     // 32-bit BGRA
}
```

**Safety**: Framebuffer must be marked as `unsafe` to write to. Provide safe wrapper functions.

#### 1.8 Timer Setup

**x86_64**:
- HPET (High Precision Event Timer) preferred
- APIC timer fallback
- Configure periodic interrupts (e.g., 100Hz)

**ARM64**:
- ARM Generic Timer
- Configure CNTP_CTL_EL0 for interrupts

**Timer Interface**:
```rust
pub fn init_timer(frequency_hz: u64);
pub fn get_ticks() -> u64;
pub fn sleep_ms(ms: u64);
```

#### 1.9 Panic Handler

```rust
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Print panic message to framebuffer
    // Halt CPU
    loop {
        x86_64::instructions::hlt();
    }
}
```

#### 1.10 BootInfo Structure

**Interface Contract**:
```rust
pub struct BootInfo {
    pub framebuffer: FramebufferInfo,
    pub memory_map: MemoryMap,
    pub rsdp_addr: Option<PhysAddr>,  // ACPI RSDP for power management
    pub heap_start: usize,
    pub heap_size: usize,
}

pub struct MemoryMap {
    pub regions: &'static [MemoryRegion],
}

pub struct MemoryRegion {
    pub start: PhysAddr,
    pub len: usize,
    pub kind: MemoryKind,
}

pub enum MemoryKind {
    Usable,
    Reserved,
    AcpiReclaimable,
    AcpiNvs,
}
```

**Entry Point Signature**:
```rust
#[no_mangle]
pub extern "C" fn kernel_main(boot_info: BootInfo) -> ! {
    // Hand off to integration agent
}
```

---

## Workstream 2: Network Stack

### Objectives
- TCP/IP connectivity via smoltcp
- TLS 1.3 support via rustls
- HTTP/1.1 client for API calls
- DHCP and DNS resolution
- Network driver abstraction

### Implementation Requirements

#### 2.1 Project Structure

```
network/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── drivers/
    │   ├── mod.rs
    │   ├── virtio.rs
    │   ├── e1000.rs
    │   └── rtl8139.rs
    ├── stack.rs
    ├── dhcp.rs
    ├── dns.rs
    ├── tls.rs
    └── http.rs
```

#### 2.2 Network Driver Trait

**Interface Contract**:
```rust
pub trait NetworkDriver: Send {
    /// Send a raw Ethernet frame
    fn send(&mut self, packet: &[u8]) -> Result<(), NetError>;
    
    /// Receive a raw Ethernet frame (non-blocking)
    fn receive(&mut self) -> Result<Option<Vec<u8>>, NetError>;
    
    /// Get MAC address
    fn mac_address(&self) -> [u8; 6];
    
    /// Check if link is up
    fn is_link_up(&self) -> bool;
    
    /// Poll for new packets (must be called regularly)
    fn poll(&mut self) -> Result<(), NetError>;
}
```

#### 2.3 virtio-net Driver

**Purpose**: Primary driver for QEMU/KVM VMs

**Implementation**:
- Detect virtio-net device via PCI
- Initialize virtio queues (RX and TX)
- Handle virtio interrupts
- Implement `NetworkDriver` trait

**Key Operations**:
1. PCI device discovery (vendor ID 0x1AF4, device ID 0x1000)
2. Virtio feature negotiation
3. Queue setup (virtqueue)
4. Interrupt handling
5. Packet transmission/reception

#### 2.4 e1000 Driver

**Purpose**: Intel Gigabit Ethernet (common in VMs and older hardware)

**Implementation**:
- PCI device discovery (vendor ID 0x8086)
- Register access (MMIO)
- TX/RX descriptor rings
- Interrupt handling

**Key Registers**:
- `CTRL`: Control register
- `TCTL`: Transmit control
- `RCTL`: Receive control
- `TDBAL/TDBAH`: TX descriptor base address
- `RDBAL/RDBAH`: RX descriptor base address

#### 2.5 RTL8139 Driver

**Purpose**: Realtek 10/100 Ethernet (legacy hardware)

**Implementation**:
- PCI device discovery (vendor ID 0x10EC, device ID 0x8139)
- I/O port access
- TX/RX buffer management
- Interrupt handling

#### 2.6 smoltcp Integration

**Crate**: `smoltcp` (with `no_std` feature)

**Network Stack Setup**:
```rust
use smoltcp::{
    iface::{Interface, InterfaceBuilder, NeighborCache, Routes},
    socket::{SocketSet, TcpSocket, TcpSocketBuffer},
    wire::{EthernetAddress, IpAddress, IpCidr, Ipv4Address},
    time::Instant,
};

pub struct NetworkStack {
    interface: Interface,
    sockets: SocketSet<'static>,
    neighbor_cache: NeighborCache<'static>,
    routes: Routes<'static>,
    driver: Box<dyn NetworkDriver>,
}

impl NetworkStack {
    pub fn new(
        driver: Box<dyn NetworkDriver>,
        mac: EthernetAddress,
    ) -> Result<Self, NetError> {
        // Create interface
        // Set up neighbor cache
        // Set up routes
        // Create socket set
    }
    
    pub fn poll(&mut self, timestamp: Instant) -> Result<(), NetError> {
        // Receive packets from driver
        // Feed to smoltcp
        // Process sockets
        // Send packets from smoltcp to driver
    }
}
```

**Polling Requirements**:
- Must be called regularly (e.g., every 10ms or in main loop)
- Handles packet reception, TCP state machine, timeouts

#### 2.7 DHCP Client

**Implementation**:
```rust
impl NetworkStack {
    pub fn dhcp_acquire(&mut self) -> Result<IpConfig, NetError> {
        // Create DHCP socket
        // Send DHCP Discover
        // Wait for DHCP Offer
        // Send DHCP Request
        // Wait for DHCP Ack
        // Configure interface with IP, gateway, DNS
    }
}

pub struct IpConfig {
    pub ip: Ipv4Address,
    pub gateway: Ipv4Address,
    pub dns: Vec<Ipv4Address>,
    pub subnet_mask: Ipv4Address,
}
```

#### 2.8 DNS Resolver

**Implementation**:
```rust
impl NetworkStack {
    pub fn dns_resolve(
        &mut self,
        hostname: &str,
        dns_server: Ipv4Address,
    ) -> Result<Ipv4Address, NetError> {
        // Create UDP socket
        // Send DNS query (A record)
        // Parse DNS response
        // Return IP address
    }
}
```

**DNS Query Format**:
- Standard DNS packet format (RFC 1035)
- Query type: A (IPv4 address)
- Recursion desired flag set

#### 2.9 TCP Connection Management

**Interface**:
```rust
pub struct TcpHandle {
    socket_handle: smoltcp::socket::SocketHandle,
}

impl NetworkStack {
    pub fn tcp_connect(
        &mut self,
        addr: SocketAddr,
    ) -> Result<TcpHandle, NetError> {
        // Create TCP socket
        // Bind to local port
        // Connect to remote address
        // Wait for connection established
    }
    
    pub fn tcp_send(
        &mut self,
        handle: &TcpHandle,
        data: &[u8],
    ) -> Result<usize, NetError>;
    
    pub fn tcp_receive(
        &mut self,
        handle: &TcpHandle,
        buffer: &mut [u8],
    ) -> Result<usize, NetError>;
    
    pub fn tcp_close(&mut self, handle: TcpHandle) -> Result<(), NetError>;
}
```

#### 2.10 TLS 1.3 Support

**Crate**: `rustls` (with `no_std` feature if available, or custom TLS)

**Note**: `rustls` may require `std`. If so, implement minimal TLS 1.3 client:
- TLS 1.3 handshake
- Certificate verification (CA chain)
- Record layer encryption/decryption
- AES-GCM for encryption

**Interface**:
```rust
pub struct TlsStream {
    tcp: TcpHandle,
    // TLS state
}

impl NetworkStack {
    pub fn tls_connect(
        &mut self,
        hostname: &str,
        port: u16,
    ) -> Result<TlsStream, NetError> {
        // Resolve hostname
        // TCP connect
        // TLS handshake
        // Verify certificate
    }
}

impl TlsStream {
    pub fn send(&mut self, data: &[u8]) -> Result<usize, NetError>;
    pub fn receive(&mut self, buffer: &mut [u8]) -> Result<usize, NetError>;
}
```

#### 2.11 HTTP/1.1 Client

**Implementation**:
```rust
pub struct HttpClient {
    network: NetworkStack,
}

impl HttpClient {
    pub fn post_json(
        &mut self,
        url: &str,
        body: &str,
        headers: &[(&str, &str)],
    ) -> Result<HttpResponse, HttpError> {
        // Parse URL
        // TLS connect
        // Send HTTP POST request
        // Parse HTTP response
    }
    
    pub fn post_streaming(
        &mut self,
        url: &str,
        body: &str,
        headers: &[(&str, &str)],
    ) -> Result<StreamingResponse, HttpError> {
        // Similar to post_json but returns streaming iterator
    }
}

pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

pub struct StreamingResponse {
    // Iterator over response chunks
}
```

**HTTP Request Format**:
```
POST /v1/chat/completions HTTP/1.1\r\n
Host: api.openai.com\r\n
Content-Type: application/json\r\n
Authorization: Bearer sk-...\r\n
Content-Length: 123\r\n
\r\n
{...}
```

**HTTP Response Parsing**:
- Parse status line
- Parse headers (until `\r\n\r\n`)
- Read body (Content-Length or chunked encoding)

#### 2.12 Error Types

```rust
#[derive(Debug)]
pub enum NetError {
    DriverError(String),
    SmoltcpError(smoltcp::Error),
    DnsError(String),
    TlsError(String),
    HttpError(String),
    Timeout,
    ConnectionRefused,
    InvalidAddress,
}
```

---

## Workstream 3: TUI Framework

### Objectives
- Framebuffer text rendering
- Color system (24-bit + fallbacks)
- Dark and light themes
- Widget system (input, messages, modals)
- Syntax highlighting
- Markdown rendering

### Implementation Requirements

#### 3.1 Project Structure

```
tui/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── framebuffer.rs
    ├── font.rs
    ├── colors.rs
    ├── theme.rs
    ├── widgets/
    │   ├── mod.rs
    │   ├── text.rs
    │   ├── input.rs
    │   ├── message.rs
    │   ├── modal.rs
    │   ├── status.rs
    │   └── hotkeys.rs
    ├── syntax.rs
    ├── markdown.rs
    └── screens/
        ├── mod.rs
        ├── chat.rs
        ├── config.rs
        ├── wizard.rs
        └── help.rs
```

#### 3.2 Framebuffer Rendering

**Pixel Writing**:
```rust
pub struct Framebuffer {
    base: *mut u8,
    width: usize,
    height: usize,
    stride: usize,
    format: PixelFormat,
}

impl Framebuffer {
    pub unsafe fn set_pixel(&mut self, x: usize, y: usize, color: Color);
    pub unsafe fn fill_rect(&mut self, rect: Rect, color: Color);
    pub unsafe fn draw_line(&mut self, start: Point, end: Point, color: Color);
}
```

**Safety**: All framebuffer operations are `unsafe`. Provide safe wrapper functions that validate bounds.

#### 3.3 Font System

**Format**: PSF (PC Screen Font) v2

**Font Loading**:
```rust
pub struct Font {
    glyphs: &'static [u8],
    width: usize,
    height: usize,
    glyph_count: usize,
}

impl Font {
    pub fn load_psf(data: &[u8]) -> Result<Self, FontError>;
    pub fn render_glyph(&self, ch: char, buffer: &mut [u8]) -> Option<&[u8]>;
}
```

**Default Font**: Terminus or similar monospace bitmap font (embedded in binary)

**Text Rendering**:
```rust
pub fn draw_text(
    fb: &mut Framebuffer,
    font: &Font,
    x: usize,
    y: usize,
    text: &str,
    color: Color,
);
```

#### 3.4 Color System

**Color Definition**:
```rust
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,  // Alpha (for compositing)
}

impl Color {
    pub fn from_hex(hex: &str) -> Result<Self, ColorError>;
    pub fn to_rgb(&self) -> (u8, u8, u8);
    pub fn blend(&self, other: Color, alpha: f32) -> Color;
}
```

**Color Palette** (from PRD):

**Dark Theme**:
- Background: `#0D1117`
- Surface: `#161B22`
- Border: `#21262D`
- Text Primary: `#F0F6FC`
- Text Secondary: `#C9D1D9`
- Accent Primary: `#58A6FF`
- Accent Success: `#7EE787`
- Accent Warning: `#FFA657`
- Accent Error: `#FF7B72`
- Accent Assistant: `#A371F7`

**Light Theme**:
- Background: `#FFFFFF`
- Surface: `#F6F8FA`
- Border: `#D0D7DE`
- Text Primary: `#1F2328`
- Text Secondary: `#424A53`
- Accent Primary: `#0969DA`
- Accent Success: `#1A7F37`
- Accent Warning: `#9A6700`
- Accent Error: `#CF222E`
- Accent Assistant: `#8250DF`

**Provider Brand Colors**:
- OpenAI: `#10A37F`
- Anthropic: `#D4A574`
- Groq: `#F55036`
- xAI: `#FFFFFF`
- Local: `#7C3AED`

#### 3.5 Theme System

**Theme Structure**:
```rust
pub struct Theme {
    pub background: Color,
    pub surface: Color,
    pub border: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_tertiary: Color,
    pub text_disabled: Color,
    pub accent_primary: Color,
    pub accent_success: Color,
    pub accent_warning: Color,
    pub accent_error: Color,
    pub accent_assistant: Color,
    pub accent_code: Color,
    pub provider_openai: Color,
    pub provider_anthropic: Color,
    pub provider_groq: Color,
    pub provider_xai: Color,
    pub provider_local: Color,
}

pub static DARK_THEME: Theme = Theme {
    background: Color::from_hex("#0D1117").unwrap(),
    // ... all colors from PRD
};

pub static LIGHT_THEME: Theme = Theme {
    background: Color::from_hex("#FFFFFF").unwrap(),
    // ... all colors from PRD
};
```

#### 3.6 Screen Interface

**Main Screen Structure**:
```rust
pub struct Screen {
    framebuffer: Framebuffer,
    font: Font,
    theme: &'static Theme,
    dirty: bool,  // Track if redraw needed
}

impl Screen {
    pub fn new(fb_info: FramebufferInfo, theme: &'static Theme) -> Self;
    pub fn clear(&mut self);
    pub fn present(&mut self);  // Flush to screen
    pub fn draw_text(&mut self, x: usize, y: usize, text: &str, color: Color);
    pub fn draw_box(&mut self, rect: Rect, style: BoxStyle);
    pub fn draw_message(&mut self, msg: &Message, rect: Rect);
    pub fn draw_input(&mut self, input: &InputState, rect: Rect, focused: bool);
    pub fn draw_modal(&mut self, modal: &Modal);
    pub fn draw_code_block(&mut self, code: &str, lang: Option<&str>, rect: Rect);
}
```

#### 3.7 Widget System

**Base Widget Trait**:
```rust
pub trait Widget {
    fn render(&self, screen: &mut Screen, rect: Rect);
    fn handle_input(&mut self, key: Key) -> WidgetEvent;
    fn size_hint(&self) -> (usize, usize);
}
```

**Input Widget**:
```rust
pub struct InputWidget {
    text: String,
    cursor_pos: usize,
    placeholder: String,
    focused: bool,
}

impl InputWidget {
    pub fn new(placeholder: String) -> Self;
    pub fn insert_char(&mut self, ch: char);
    pub fn delete_char(&mut self);
    pub fn move_cursor(&mut self, direction: CursorDirection);
    pub fn get_text(&self) -> &str;
    pub fn clear(&mut self);
}
```

**Message Widget**:
```rust
pub struct MessageWidget {
    role: MessageRole,
    content: String,
    timestamp: Option<u64>,
}

pub enum MessageRole {
    User,
    Assistant,
}

impl MessageWidget {
    pub fn render(&self, screen: &mut Screen, rect: Rect, theme: &Theme);
}
```

**Modal Widget**:
```rust
pub struct Modal {
    title: String,
    content: Box<dyn Widget>,
    buttons: Vec<Button>,
}

impl Modal {
    pub fn render(&self, screen: &mut Screen, rect: Rect);
    pub fn handle_input(&mut self, key: Key) -> ModalEvent;
}
```

#### 3.8 Chat Screen

**Chat State**:
```rust
pub struct ChatScreen {
    messages: Vec<MessageWidget>,
    input: InputWidget,
    scroll_offset: usize,
    status: ConnectionStatus,
    provider: String,
    model: String,
}

pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Error(String),
}

impl ChatScreen {
    pub fn render(&mut self, screen: &mut Screen);
    pub fn add_message(&mut self, role: MessageRole, content: String);
    pub fn scroll_up(&mut self);
    pub fn scroll_down(&mut self);
    pub fn handle_input(&mut self, key: Key) -> ChatEvent;
}
```

**Layout**:
- Header bar: 1 line (title, provider, status)
- Chat area: Remaining height - input height - footer height
- Input area: 3 lines
- Footer/hotkeys: 1 line

#### 3.9 Syntax Highlighting

**Token Types**:
```rust
pub enum TokenType {
    Keyword,
    String,
    Number,
    Comment,
    Function,
    Type,
    Variable,
    Operator,
    Punctuation,
    Constant,
}
```

**Syntax Highlighter**:
```rust
pub struct SyntaxHighlighter {
    language: Option<String>,
}

impl SyntaxHighlighter {
    pub fn highlight(&self, code: &str) -> Vec<(TokenType, &str)>;
    pub fn get_color(&self, token_type: TokenType, theme: &Theme) -> Color;
}
```

**Supported Languages** (minimum):
- Rust
- Python
- JavaScript
- JSON
- Markdown

**Implementation**: Simple regex-based tokenizer (no full parser needed)

#### 3.10 Markdown Rendering

**Supported Markdown**:
- **Bold** (`**text**`)
- *Italic* (`*text*`)
- `Code` (`` `code` ``)
- Code blocks (```language\ncode\n```)
- Headers (`# Header`)

**Markdown Parser**:
```rust
pub struct MarkdownParser;

impl MarkdownParser {
    pub fn parse(&self, text: &str) -> Vec<MarkdownElement>;
}

pub enum MarkdownElement {
    Text(String),
    Bold(String),
    Italic(String),
    Code(String),
    CodeBlock { language: Option<String>, code: String },
    Header { level: usize, text: String },
}
```

**Rendering**:
```rust
pub fn render_markdown(
    screen: &mut Screen,
    elements: &[MarkdownElement],
    rect: Rect,
    theme: &Theme,
);
```

#### 3.11 Screen Management

**App Screen Enum**:
```rust
pub enum AppScreen {
    Chat(ChatScreen),
    Config(ConfigScreen),
    ProviderSelect(SelectScreen),
    ModelSelect(SelectScreen),
    SetupWizard(WizardScreen),
    Help(HelpScreen),
}
```

**Screen Transitions**:
- F1: Help
- F2: Provider Select
- F3: Model Select
- F4: Config
- F9: New Chat (clear messages)
- F10: Shutdown
- Esc: Back to Chat

---

## Workstream 4: LLM API Clients

### Objectives
- Common LLM provider trait
- OpenAI API client with streaming
- Anthropic API client with streaming
- Groq API client with streaming
- xAI API client with streaming
- Error handling and retries

### Implementation Requirements

#### 4.1 Project Structure

```
llm/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── types.rs
    ├── streaming.rs
    ├── providers/
    │   ├── mod.rs
    │   ├── openai.rs
    │   ├── anthropic.rs
    │   ├── groq.rs
    │   └── xai.rs
    └── error.rs
```

#### 4.2 Common Types

**Message Structure**:
```rust
pub struct Message {
    pub role: Role,
    pub content: String,
}

pub enum Role {
    System,
    User,
    Assistant,
}
```

**Generation Config**:
```rust
pub struct GenerationConfig {
    pub temperature: f32,        // 0.0-2.0
    pub max_tokens: Option<usize>,
    pub stop_sequences: Vec<String>,
    pub top_p: Option<f32>,      // 0.0-1.0
    pub top_k: Option<usize>,
}
```

**Model Info**:
```rust
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub context_length: usize,
    pub supports_streaming: bool,
}
```

**Completion Result**:
```rust
pub struct CompletionResult {
    pub text: String,
    pub tokens_used: Option<usize>,
    pub finish_reason: FinishReason,
}

pub enum FinishReason {
    Stop,
    Length,
    ContentFilter,
    Other(String),
}
```

#### 4.3 LLM Provider Trait

**Interface**:
```rust
pub trait LlmProvider: Send {
    fn name(&self) -> &str;
    fn models(&self) -> &[ModelInfo];
    fn default_model(&self) -> &str;
    
    fn complete(
        &mut self,
        messages: &[Message],
        model: &str,
        config: &GenerationConfig,
        on_token: impl FnMut(&str),
    ) -> Result<CompletionResult, LlmError>;
    
    fn validate_api_key(&self) -> Result<(), LlmError>;
}
```

**Note**: For `no_std`, use callback-based approach instead of async/await.

#### 4.4 OpenAI Client

**API Endpoint**: `https://api.openai.com/v1/chat/completions`

**Request Format**:
```json
{
  "model": "gpt-4o",
  "messages": [
    {"role": "system", "content": "..."},
    {"role": "user", "content": "..."}
  ],
  "temperature": 0.7,
  "max_tokens": 2048,
  "stream": true
}
```

**Streaming Response** (Server-Sent Events):
```
data: {"id":"...","object":"chat.completion.chunk","choices":[{"delta":{"content":"Hello"}}]}

data: {"id":"...","object":"chat.completion.chunk","choices":[{"delta":{"content":" there"}}]}

data: [DONE]
```

**Implementation**:
```rust
pub struct OpenAiClient {
    api_key: String,
    http_client: HttpClient,
    base_url: String,
}

impl OpenAiClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            http_client: HttpClient::new(),
            base_url: "https://api.openai.com".to_string(),
        }
    }
}

impl LlmProvider for OpenAiClient {
    fn name(&self) -> &str { "OpenAI" }
    
    fn models(&self) -> &[ModelInfo] {
        &[
            ModelInfo { id: "gpt-4o", name: "GPT-4o", context_length: 128000, supports_streaming: true },
            ModelInfo { id: "gpt-4o-mini", name: "GPT-4o Mini", context_length: 128000, supports_streaming: true },
            ModelInfo { id: "gpt-4-turbo", name: "GPT-4 Turbo", context_length: 128000, supports_streaming: true },
            ModelInfo { id: "o1", name: "O1", context_length: 200000, supports_streaming: false },
            ModelInfo { id: "o1-mini", name: "O1 Mini", context_length: 128000, supports_streaming: false },
            ModelInfo { id: "o3-mini", name: "O3 Mini", context_length: 128000, supports_streaming: false },
        ]
    }
    
    fn complete(
        &mut self,
        messages: &[Message],
        model: &str,
        config: &GenerationConfig,
        mut on_token: impl FnMut(&str),
    ) -> Result<CompletionResult, LlmError> {
        // Build request JSON
        // Make streaming HTTP POST
        // Parse SSE stream
        // Call on_token for each chunk
        // Return completion result
    }
}
```

**SSE Parser**:
```rust
pub struct SseParser;

impl SseParser {
    pub fn parse_line(&self, line: &str) -> Option<serde_json::Value>;
    pub fn extract_content(&self, json: &serde_json::Value) -> Option<String>;
}
```

#### 4.5 Anthropic Client

**API Endpoint**: `https://api.anthropic.com/v1/messages`

**Request Format**:
```json
{
  "model": "claude-sonnet-4-20250514",
  "max_tokens": 4096,
  "messages": [
    {"role": "user", "content": "..."}
  ],
  "system": "...",
  "temperature": 0.7,
  "stream": true
}
```

**Streaming Response** (SSE):
```
event: message_start
data: {"type":"message_start","message":{"id":"...","type":"message","role":"assistant","content":[]}}

event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hello"}}

event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":" there"}}

event: message_stop
data: {"type":"message_stop"}
```

**Models**:
- `claude-sonnet-4-20250514`
- `claude-opus-4-20250514`
- `claude-haiku-3-5-20241022`

#### 4.6 Groq Client

**API Endpoint**: `https://api.groq.com/openai/v1/chat/completions`

**Request Format**: Same as OpenAI (OpenAI-compatible API)

**Models**:
- `llama-3.3-70b-versatile`
- `llama-3.1-8b-instant`
- `mixtral-8x7b-32768`
- `gemma2-9b-it`

#### 4.7 xAI Client

**API Endpoint**: `https://api.x.ai/v1/chat/completions`

**Request Format**: OpenAI-compatible

**Models**:
- `grok-2`
- `grok-2-mini`

#### 4.8 Error Handling

**Error Types**:
```rust
#[derive(Debug)]
pub enum LlmError {
    NetworkError(String),
    HttpError { status: u16, body: String },
    AuthError(String),
    RateLimitError { retry_after: Option<u64> },
    InvalidModel(String),
    ParseError(String),
    Timeout,
    Other(String),
}
```

**Retry Logic**:
```rust
pub fn with_retry<F, T>(
    mut f: F,
    max_retries: usize,
) -> Result<T, LlmError>
where
    F: FnMut() -> Result<T, LlmError>,
{
    let mut last_error = None;
    for attempt in 0..max_retries {
        match f() {
            Ok(result) => return Ok(result),
            Err(e @ LlmError::RateLimitError { .. }) => {
                // Wait before retry
                last_error = Some(e);
                sleep_ms(1000 * (attempt + 1));
            }
            Err(e) => return Err(e),
        }
    }
    Err(last_error.unwrap_or(LlmError::Other("Max retries exceeded".to_string())))
}
```

---

## Workstream 5: WiFi Stack

### Objectives
- USB host controller support
- WiFi driver for MT7601U (Tier 1)
- 802.11 frame handling
- WPA2-PSK authentication
- Integration with network stack

### Implementation Requirements

#### 5.1 Project Structure

```
wifi/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── usb/
    │   ├── mod.rs
    │   ├── xhci.rs
    │   ├── ehci.rs
    │   └── device.rs
    ├── drivers/
    │   ├── mod.rs
    │   ├── mt7601u.rs
    │   └── rtl8188.rs
    ├── ieee80211.rs
    └── wpa2.rs
```

#### 5.2 USB Host Controller

**xHCI (USB 3.0)**:
- PCI device discovery
- Register setup
- Device enumeration
- Bulk/interrupt transfers

**EHCI (USB 2.0)**:
- Fallback for older hardware
- Similar interface to xHCI

**USB Device Interface**:
```rust
pub trait UsbDevice {
    fn vendor_id(&self) -> u16;
    fn product_id(&self) -> u16;
    fn control_transfer(&mut self, request: UsbRequest) -> Result<Vec<u8>, UsbError>;
    fn bulk_transfer(&mut self, endpoint: u8, data: &[u8]) -> Result<(), UsbError>;
    fn interrupt_transfer(&mut self, endpoint: u8) -> Result<Vec<u8>, UsbError>;
}
```

#### 5.3 MT7601U Driver

**Chipset**: MediaTek MT7601U (USB 2.0, 802.11n)

**Key Operations**:
1. USB device detection (vendor ID 0x0E8D, product ID 0x7601)
2. Firmware loading (if required)
3. Register initialization
4. 802.11 frame transmission/reception
5. WPA2 handshake support

**Driver Interface**:
```rust
pub struct Mt7601uDriver {
    usb_device: Box<dyn UsbDevice>,
    // Internal state
}

impl Mt7601uDriver {
    pub fn new(usb_device: Box<dyn UsbDevice>) -> Result<Self, WifiError>;
    pub fn init(&mut self) -> Result<(), WifiError>;
    pub fn set_channel(&mut self, channel: u8) -> Result<(), WifiError>;
    pub fn send_frame(&mut self, frame: &[u8]) -> Result<(), WifiError>;
    pub fn receive_frame(&mut self) -> Result<Option<Vec<u8>>, WifiError>;
}
```

#### 5.4 802.11 Frame Handling

**Frame Types**:
- Management frames (beacon, probe, auth, assoc)
- Control frames (ACK, RTS, CTS)
- Data frames (encrypted with WPA2)

**Frame Parser**:
```rust
pub struct Ieee80211Frame {
    pub frame_control: u16,
    pub duration: u16,
    pub addr1: [u8; 6],  // Destination
    pub addr2: [u8; 6],  // Source
    pub addr3: [u8; 6],  // BSSID
    pub sequence: u16,
    pub body: Vec<u8>,
}

impl Ieee80211Frame {
    pub fn parse(data: &[u8]) -> Result<Self, FrameError>;
    pub fn frame_type(&self) -> FrameType;
}

pub enum FrameType {
    Management(ManagementSubtype),
    Control(ControlSubtype),
    Data(DataSubtype),
}
```

#### 5.5 WiFi Network Scanning

**Implementation**:
```rust
pub struct WifiNetwork {
    pub ssid: String,
    pub bssid: [u8; 6],
    pub signal_strength: i8,  // dBm
    pub security: SecurityType,
    pub channel: u8,
    pub frequency: u16,  // MHz
}

pub enum SecurityType {
    Open,
    WPA2Personal,
    WPA3Personal,
}

pub struct WifiDriver {
    // Driver implementation
}

impl WifiDriver {
    pub fn scan(&mut self) -> Result<Vec<WifiNetwork>, WifiError> {
        // Send probe requests
        // Listen for beacons
        // Parse and return networks
    }
}
```

#### 5.6 WPA2-PSK Authentication

**4-Way Handshake**:
1. AP sends ANonce
2. Station sends SNonce + MIC
3. AP sends GTK + MIC
4. Station sends ACK

**Implementation**:
```rust
pub struct Wpa2Supplicant {
    ssid: String,
    password: String,
    pmk: [u8; 32],  // Pairwise Master Key
    ptk: Option<PairwiseTransientKey>,
}

impl Wpa2Supplicant {
    pub fn new(ssid: String, password: String) -> Self {
        // Derive PMK from password
        let pmk = Self::derive_pmk(&ssid, &password);
        Self { ssid, password, pmk, ptk: None }
    }
    
    fn derive_pmk(ssid: &str, password: &str) -> [u8; 32] {
        // PBKDF2 with SHA-1, 4096 iterations
    }
    
    pub fn perform_handshake(
        &mut self,
        driver: &mut dyn WifiDriver,
    ) -> Result<(), WpaError> {
        // Execute 4-way handshake
    }
}
```

**CCMP Encryption**:
- AES-128 in CCM mode
- Encrypt/decrypt data frames

#### 5.7 Network Driver Integration

**Implement NetworkDriver Trait**:
```rust
impl NetworkDriver for WifiDriver {
    fn send(&mut self, packet: &[u8]) -> Result<(), NetError> {
        // Wrap in 802.11 data frame
        // Encrypt with CCMP
        // Send via driver
    }
    
    fn receive(&mut self) -> Result<Option<Vec<u8>>, NetError> {
        // Receive 802.11 frame
        // Decrypt with CCMP
        // Extract Ethernet frame
    }
    
    fn mac_address(&self) -> [u8; 6] {
        // Return WiFi MAC address
    }
}
```

---

## Workstream 6: Local Inference Engine

### Objectives
- GGUF file format parser
- Model weight loading
- Tokenizer (BPE)
- Transformer inference
- Quantized operations (Q4_K_M)

### Implementation Requirements

#### 6.1 Project Structure

```
inference/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── gguf.rs
    ├── tokenizer.rs
    ├── tensor.rs
    ├── ops.rs
    ├── transformer.rs
    ├── sampling.rs
    └── simd/
        ├── mod.rs
        ├── x86.rs
        └── arm.rs
```

#### 6.2 GGUF Parser

**GGUF Format**:
- Magic: "GGUF"
- Version: u32
- Tensor count: u64
- Metadata (key-value pairs)
- Tensor data

**Parser**:
```rust
pub struct GgufFile {
    metadata: HashMap<String, MetadataValue>,
    tensors: Vec<TensorInfo>,
    data: Vec<u8>,
}

pub enum MetadataValue {
    String(String),
    UInt32(u32),
    Float32(f32),
    Array(Vec<MetadataValue>),
}

impl GgufFile {
    pub fn parse(data: &[u8]) -> Result<Self, ModelError>;
    pub fn get_tensor(&self, name: &str) -> Result<&[u8], ModelError>;
    pub fn get_metadata(&self, key: &str) -> Option<&MetadataValue>;
}
```

#### 6.3 Model Weights

**Weight Structure**:
```rust
pub struct ModelWeights {
    pub embedding: EmbeddingWeights,
    pub layers: Vec<TransformerLayerWeights>,
    pub output: OutputWeights,
}

pub struct TransformerLayerWeights {
    pub attention_norm: Vec<f32>,
    pub attention_qkv: QuantizedTensor,
    pub attention_output: QuantizedTensor,
    pub ffn_norm: Vec<f32>,
    pub ffn_gate: QuantizedTensor,
    pub ffn_up: QuantizedTensor,
    pub ffn_down: QuantizedTensor,
}

pub enum QuantizedTensor {
    F32(Vec<f32>),
    Q4K(Vec<u8>),  // Q4_K_M quantization
}
```

#### 6.4 Tokenizer

**BPE Tokenizer**:
```rust
pub struct Tokenizer {
    vocab: HashMap<String, u32>,
    merges: Vec<(String, String)>,
    special_tokens: HashMap<String, u32>,
}

impl Tokenizer {
    pub fn from_gguf(gguf: &GgufFile) -> Result<Self, ModelError>;
    pub fn encode(&self, text: &str) -> Vec<u32>;
    pub fn decode(&self, tokens: &[u32]) -> String;
}
```

#### 6.5 Tensor Operations

**Operations**:
```rust
pub mod ops {
    pub fn matmul_f32(a: &[f32], b: &[f32], m: usize, n: usize, k: usize) -> Vec<f32>;
    pub fn matmul_q4k(a: &[u8], b: &[f32], m: usize, n: usize, k: usize) -> Vec<f32>;
    pub fn add(a: &[f32], b: &[f32]) -> Vec<f32>;
    pub fn mul(a: &[f32], b: &[f32]) -> Vec<f32>;
    pub fn softmax(x: &mut [f32]);
    pub fn layer_norm(x: &[f32], weight: &[f32], bias: &[f32], eps: f32) -> Vec<f32>;
    pub fn rms_norm(x: &[f32], weight: &[f32], eps: f32) -> Vec<f32>;
    pub fn silu(x: &[f32]) -> Vec<f32>;
    pub fn rope(x: &mut [f32], pos: usize, head_dim: usize, freq_base: f32);
}
```

**SIMD Optimizations**:
- x86_64: SSE4.2, AVX2
- ARM64: NEON

#### 6.6 Transformer Forward Pass

**Implementation**:
```rust
pub struct Transformer {
    weights: ModelWeights,
    tokenizer: Tokenizer,
    config: ModelConfig,
}

pub struct ModelConfig {
    pub vocab_size: usize,
    pub hidden_size: usize,
    pub num_layers: usize,
    pub num_heads: usize,
    pub head_dim: usize,
    pub intermediate_size: usize,
    pub max_seq_len: usize,
}

impl Transformer {
    pub fn forward(
        &self,
        tokens: &[u32],
        kv_cache: &mut KvCache,
    ) -> Vec<f32> {
        // Embedding
        // Layer-by-layer processing
        // Output projection
    }
    
    fn attention_layer(
        &self,
        x: &[f32],
        layer_idx: usize,
        kv_cache: &mut KvCache,
    ) -> Vec<f32> {
        // QKV projection
        // RoPE
        // Attention
        // Output projection
    }
    
    fn ffn_layer(
        &self,
        x: &[f32],
        layer_idx: usize,
    ) -> Vec<f32> {
        // Gate projection
        // Up projection
        // SiLU activation
        // Down projection
    }
}
```

#### 6.7 KV Cache

**Implementation**:
```rust
pub struct KvCache {
    k_cache: Vec<Vec<f32>>,  // [layer][seq_len * head_dim]
    v_cache: Vec<Vec<f32>>,
    current_pos: usize,
}

impl KvCache {
    pub fn new(num_layers: usize, max_seq_len: usize, head_dim: usize) -> Self;
    pub fn append(&mut self, layer: usize, k: &[f32], v: &[f32]);
    pub fn get_k(&self, layer: usize, pos: usize) -> &[f32];
    pub fn get_v(&self, layer: usize, pos: usize) -> &[f32];
}
```

#### 6.8 Sampling

**Sampling Methods**:
```rust
pub fn sample(
    logits: &[f32],
    temperature: f32,
    top_p: Option<f32>,
    top_k: Option<usize>,
) -> u32 {
    // Apply temperature
    // Top-k filtering
    // Top-p (nucleus) sampling
    // Sample from distribution
}
```

#### 6.9 Generation Loop

**Implementation**:
```rust
impl LocalModel {
    pub fn generate(
        &mut self,
        prompt: &str,
        config: &GenerationConfig,
        mut on_token: impl FnMut(&str),
    ) -> Result<String, InferenceError> {
        let tokens = self.tokenizer.encode(prompt);
        let mut kv_cache = KvCache::new(/* ... */);
        let mut output_tokens = Vec::new();
        
        // Prefill
        for token in &tokens {
            let logits = self.transformer.forward(&[*token], &mut kv_cache);
        }
        
        // Generation loop
        loop {
            let next_token = sample(&logits, config.temperature, config.top_p, config.top_k);
            output_tokens.push(next_token);
            
            let text = self.tokenizer.decode(&[next_token]);
            on_token(&text);
            
            if next_token == self.tokenizer.eos_token() {
                break;
            }
            
            if output_tokens.len() >= config.max_tokens.unwrap_or(2048) {
                break;
            }
            
            // Next forward pass
            let logits = self.transformer.forward(&[next_token], &mut kv_cache);
        }
        
        Ok(self.tokenizer.decode(&output_tokens))
    }
}
```

#### 6.10 LlmProvider Implementation

```rust
impl LlmProvider for LocalModel {
    fn name(&self) -> &str { "Local (SmolLM)" }
    
    fn models(&self) -> &[ModelInfo] {
        &[ModelInfo {
            id: "smollm-360m",
            name: "SmolLM-360M",
            context_length: 2048,
            supports_streaming: true,
        }]
    }
    
    fn complete(
        &mut self,
        messages: &[Message],
        model: &str,
        config: &GenerationConfig,
        on_token: impl FnMut(&str),
    ) -> Result<CompletionResult, LlmError> {
        // Format messages as prompt
        // Call generate()
    }
}
```

---

## Workstream 7: Configuration System

### Objectives
- TOML parser (no_std)
- Config storage (EFI variables, file)
- API key encryption
- Setup wizard
- Config validation

### Implementation Requirements

#### 7.1 Project Structure

```
config/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── toml.rs
    ├── types.rs
    ├── storage/
    │   ├── mod.rs
    │   ├── efi.rs
    │   └── file.rs
    ├── crypto.rs
    └── wizard.rs
```

#### 7.2 Config Types

**Config Structure**:
```rust
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

pub enum ConnectionType {
    Ethernet,
    Wifi,
}

pub struct ProviderConfigs {
    pub openai: Option<ProviderConfig>,
    pub anthropic: Option<ProviderConfig>,
    pub groq: Option<ProviderConfig>,
    pub xai: Option<ProviderConfig>,
    pub ollama: Option<LocalProviderConfig>,
    pub local: Option<LocalProviderConfig>,
}

pub struct ProviderConfig {
    pub api_key_encrypted: Vec<u8>,
    pub default_model: String,
}

pub struct LocalProviderConfig {
    pub endpoint: String,
    pub default_model: String,
}

pub struct Preferences {
    pub default_provider: String,
    pub default_model: String,
    pub theme: ThemeChoice,
    pub temperature: f32,
    pub stream_responses: bool,
}

pub enum ThemeChoice {
    Dark,
    Light,
}
```

#### 7.3 TOML Parser

**Minimal Parser** (no_std compatible):
```rust
pub struct TomlParser;

impl TomlParser {
    pub fn parse(data: &str) -> Result<MoteConfig, ConfigError>;
    pub fn serialize(config: &MoteConfig) -> Result<String, ConfigError>;
}
```

**Supported TOML Features** (minimum):
- Key-value pairs
- Nested tables
- Arrays
- Strings (basic, no multiline)
- Numbers (integers, floats)
- Booleans

#### 7.4 Config Storage Trait

**Interface**:
```rust
pub trait ConfigStorage {
    fn load(&self) -> Result<Option<MoteConfig>, ConfigError>;
    fn save(&mut self, config: &MoteConfig) -> Result<(), ConfigError>;
    fn exists(&self) -> bool;
}
```

#### 7.5 EFI Variable Storage

**Implementation**:
```rust
use uefi::Variable;

pub struct EfiConfigStorage;

impl ConfigStorage for EfiConfigStorage {
    fn load(&self) -> Result<Option<MoteConfig>, ConfigError> {
        // Read EFI variable "MoteOS-Config"
        // Deserialize TOML
    }
    
    fn save(&mut self, config: &MoteConfig) -> Result<(), ConfigError> {
        // Serialize to TOML
        // Write EFI variable
    }
    
    fn exists(&self) -> bool {
        // Check if variable exists
    }
}
```

**EFI Variable**:
- Name: `MoteOS-Config`
- GUID: Custom GUID
- Size: Up to 64KB (EFI variable limit)

#### 7.6 File Storage (Fallback)

**Implementation**:
```rust
pub struct FileConfigStorage {
    path: &'static str,  // "/boot/mote.toml" or similar
}

impl ConfigStorage for FileConfigStorage {
    fn load(&self) -> Result<Option<MoteConfig>, ConfigError> {
        // Read from boot partition
        // Parse TOML
    }
    
    fn save(&mut self, config: &MoteConfig) -> Result<(), ConfigError> {
        // Serialize to TOML
        // Write to boot partition
    }
}
```

#### 7.7 API Key Encryption

**Encryption**:
```rust
pub struct KeyEncryption;

impl KeyEncryption {
    pub fn encrypt(plaintext: &str) -> Result<Vec<u8>, CryptoError> {
        // Use hardware-derived key if available
        // Otherwise use static key (less secure but functional)
        // AES-256-GCM encryption
    }
    
    pub fn decrypt(ciphertext: &[u8]) -> Result<String, CryptoError> {
        // Decrypt with same key
    }
}

fn derive_key() -> [u8; 32] {
    // Try to get hardware key (TPM, CPUID, etc.)
    // Fallback to static key
}
```

#### 7.8 Setup Wizard

**Wizard State Machine**:
```rust
pub enum WizardState {
    Welcome,
    NetworkScan { networks: Vec<WifiNetwork> },
    NetworkPassword { ssid: String },
    ApiKeys,
    Ready { config: MoteConfig },
}

pub struct SetupWizard {
    state: WizardState,
    config: MoteConfig,
}

impl SetupWizard {
    pub fn new() -> Self;
    pub fn handle_input(&mut self, key: Key) -> WizardEvent;
    pub fn render(&self, screen: &mut Screen);
    pub fn get_config(&self) -> Option<&MoteConfig>;
}
```

**Wizard Flow**:
1. Welcome screen
2. Network selection (scan WiFi or skip for Ethernet)
3. WiFi password (if WiFi selected)
4. API key input (optional, can skip to use local model)
5. Ready screen (summary)

---

## Workstream 8: Integration & Build

### Objectives
- Main kernel entry point
- Component initialization
- Event loop
- Keyboard input
- ISO generation

### Implementation Requirements

#### 8.1 Project Structure

```
kernel/
├── Cargo.toml
└── src/
    ├── main.rs
    ├── init.rs
    ├── input.rs
    ├── event_loop.rs
    └── panic.rs
```

#### 8.2 Main Entry Point

**Kernel Main**:
```rust
#![no_std]
#![no_main]

use boot::BootInfo;
use network::NetworkStack;
use tui::{Screen, AppScreen, DARK_THEME};
use llm::LlmProvider;
use config::{MoteConfig, ConfigStorage, EfiConfigStorage};

static mut GLOBAL_STATE: Option<KernelState> = None;

struct KernelState {
    screen: Screen,
    network: Option<NetworkStack>,
    config: MoteConfig,
    current_provider: Box<dyn LlmProvider>,
    app_screen: AppScreen,
    conversation: Vec<Message>,
    input_buffer: String,
}

#[no_mangle]
pub extern "C" fn kernel_main(boot_info: BootInfo) -> ! {
    // Initialize heap
    boot::init_heap(boot_info.heap_start, boot_info.heap_size);
    
    // Initialize screen
    let screen = Screen::new(boot_info.framebuffer, &DARK_THEME);
    
    // Load config
    let config_storage = EfiConfigStorage;
    let config = config_storage.load()
        .unwrap_or_else(|_| MoteConfig::default());
    
    // Initialize network (if configured)
    let network = init_network(&config).ok();
    
    // Initialize LLM provider
    let provider = init_provider(&config);
    
    // Initialize app screen
    let app_screen = if config_storage.exists() {
        AppScreen::Chat(ChatScreen::new())
    } else {
        AppScreen::SetupWizard(SetupWizard::new())
    };
    
    // Set global state
    unsafe {
        GLOBAL_STATE = Some(KernelState {
            screen,
            network,
            config,
            current_provider: provider,
            app_screen,
            conversation: Vec::new(),
            input_buffer: String::new(),
        });
    }
    
    // Enter main loop
    main_loop();
}

fn main_loop() -> ! {
    loop {
        // Handle input
        handle_input();
        
        // Poll network
        poll_network();
        
        // Update screen
        update_screen();
        
        // Small delay (or use timer interrupt)
        sleep_ms(16);  // ~60 FPS
    }
}
```

#### 8.3 Input Handling

**Keyboard Support**:
- PS/2 keyboard (legacy)
- USB HID keyboard (modern)

**Input Handler**:
```rust
pub fn handle_input() {
    let state = unsafe { GLOBAL_STATE.as_mut().unwrap() };
    
    if let Some(key) = read_keyboard() {
        match &mut state.app_screen {
            AppScreen::Chat(chat) => {
                match key {
                    Key::Char(ch) => chat.input.insert_char(ch),
                    Key::Enter => {
                        let message = chat.input.get_text().to_string();
                        send_message(message);
                        chat.input.clear();
                    }
                    Key::F1 => state.app_screen = AppScreen::Help(HelpScreen::new()),
                    Key::F2 => state.app_screen = AppScreen::ProviderSelect(/* ... */),
                    // ... other keys
                    _ => {}
                }
            }
            AppScreen::SetupWizard(wizard) => {
                wizard.handle_input(key);
            }
            // ... other screens
        }
    }
}
```

#### 8.4 Network Polling

**Polling**:
```rust
pub fn poll_network() {
    let state = unsafe { GLOBAL_STATE.as_mut().unwrap() };
    
    if let Some(ref mut network) = state.network {
        let timestamp = Instant::now();
        if let Err(e) = network.poll(timestamp) {
            // Handle error
        }
    }
}
```

#### 8.5 Screen Updates

**Update Loop**:
```rust
pub fn update_screen() {
    let state = unsafe { GLOBAL_STATE.as_mut().unwrap() };
    
    state.screen.clear();
    
    match &mut state.app_screen {
        AppScreen::Chat(chat) => {
            chat.render(&mut state.screen);
        }
        AppScreen::SetupWizard(wizard) => {
            wizard.render(&mut state.screen);
        }
        // ... other screens
    }
    
    state.screen.present();
}
```

#### 8.6 Message Sending

**LLM Integration**:
```rust
pub fn send_message(text: String) {
    let state = unsafe { GLOBAL_STATE.as_mut().unwrap() };
    
    // Add user message to conversation
    state.conversation.push(Message {
        role: Role::User,
        content: text.clone(),
    });
    
    // Update chat screen
    if let AppScreen::Chat(ref mut chat) = state.app_screen {
        chat.add_message(MessageRole::User, text);
    }
    
    // Generate response
    let mut response_text = String::new();
    let result = state.current_provider.complete(
        &state.conversation,
        &state.config.preferences.default_model,
        &GenerationConfig {
            temperature: state.config.preferences.temperature,
            max_tokens: None,
            stop_sequences: Vec::new(),
        },
        |token| {
            response_text.push_str(token);
            // Update chat screen with streaming token
            if let AppScreen::Chat(ref mut chat) = state.app_screen {
                chat.update_last_message(&response_text);
            }
        },
    );
    
    // Add assistant message
    if let Ok(result) = result {
        state.conversation.push(Message {
            role: Role::Assistant,
            content: result.text,
        });
    }
}
```

#### 8.7 ISO Generation

**Build Script**:
```bash
#!/bin/bash
# build-iso.sh

# Build kernel
cargo build --release --target x86_64-unknown-uefi

# Create ISO structure
mkdir -p iso/EFI/BOOT
cp target/x86_64-unknown-uefi/release/moteos.efi iso/EFI/BOOT/BOOTX64.EFI

# Create ISO
xorriso -as mkisofs \
    -R -J \
    -b isolinux/isolinux.bin \
    -no-emul-boot \
    -boot-load-size 4 \
    -boot-info-table \
    -o moteos-x64.iso \
    iso/
```

**Makefile Targets**:
```makefile
.PHONY: run-qemu-x64-uefi
run-qemu-x64-uefi:
	qemu-system-x86_64 \
		-machine q35 \
		-cpu qemu64 \
		-m 1G \
		-drive if=pflash,format=raw,file=OVMF.fd \
		-drive format=raw,file=moteos-x64.iso \
		-netdev user,id=net0 \
		-device virtio-net,netdev=net0

.PHONY: iso-x64
iso-x64:
	./tools/build-iso.sh
```

---

## Build System Specifications

### Cargo Workspace

**Root Cargo.toml**:
```toml
[workspace]
members = [
    "boot",
    "network",
    "wifi",
    "tui",
    "llm",
    "inference",
    "config",
    "kernel",
    "shared",
]

[workspace.package]
version = "0.1.0"
edition = "2021"

[workspace.dependencies]
# Common dependencies
```

### Target Specifications

**x86_64 UEFI**:
```toml
[target.'cfg(target_arch = "x86_64")']
rustflags = [
    "-C", "link-arg=-Tlinker.ld",
    "-C", "link-arg=-nostdlib",
]
```

**ARM64 UEFI**:
```toml
[target.'cfg(target_arch = "aarch64")']
rustflags = [
    "-C", "link-arg=-Tlinker.ld",
    "-C", "link-arg=-nostdlib",
]
```

### Linker Scripts

**x86_64 Linker Script** (`linker-x86_64.ld`):
```
ENTRY(_start)

SECTIONS {
    . = 0x100000;
    
    .text : {
        *(.text .text.*)
    }
    
    .rodata : {
        *(.rodata .rodata.*)
    }
    
    .data : {
        *(.data .data.*)
    }
    
    .bss : {
        *(.bss .bss.*)
    }
}
```

### Build Dependencies

**Required Tools**:
- Rust nightly toolchain
- `cargo-make` or custom build scripts
- `xorriso` for ISO creation
- QEMU for testing
- OVMF (UEFI firmware for QEMU)

**Crate Dependencies** (key ones):
- `uefi` - UEFI boot services
- `x86_64` - x86_64 CPU abstractions
- `smoltcp` - TCP/IP stack
- `linked_list_allocator` - Heap allocator
- `spin` - Spinlocks

---

## Testing Requirements

### Unit Tests

**Test Structure**:
- Each crate should have unit tests where possible
- Use `#[cfg(test)]` modules
- Mock dependencies for network/LLM tests

### Integration Tests

**QEMU Testing**:
- Boot test (verify kernel_main reached)
- Network test (DHCP, DNS, HTTP)
- LLM API test (mock or real API)
- TUI rendering test (screenshot comparison)

### Test Scripts

**run-tests.sh**:
```bash
#!/bin/bash
# Run all tests in QEMU

# Boot test
qemu-system-x86_64 -kernel target/release/moteos.efi -serial stdio

# Network test
# ... automated network connectivity test

# API test
# ... automated LLM API test
```

---

## Performance Targets

### Boot Performance

| Stage | Target Time |
|-------|-------------|
| Bootloader → kernel_main | <500ms |
| Kernel init | <1s |
| Network connect | <2s |
| Total boot to prompt | <3s |

### Runtime Performance

| Operation | Target |
|-----------|--------|
| Screen refresh | 60 FPS |
| Network polling | <10ms overhead |
| LLM token streaming | Real-time (no buffering) |
| Local inference | >10 tokens/sec (SmolLM-360M) |

### Memory Usage

| Component | Max Memory |
|-----------|------------|
| Kernel code | <5MB |
| Heap (base) | <64MB |
| Network buffers | <16MB |
| TUI buffers | <8MB |
| Local model weights | ~200MB (SmolLM-360M) |
| KV cache | ~100MB (max context) |
| **Total (with model)** | **<400MB** |

---

## Error Handling Strategy

### Error Types

Each workstream defines its own error types:
- `BootError` - Boot-related errors
- `NetError` - Network errors
- `LlmError` - LLM API errors
- `WifiError` - WiFi errors
- `ModelError` - Inference errors
- `ConfigError` - Config errors

### Error Propagation

- Use `Result<T, E>` for fallible operations
- Propagate errors up to main loop
- Display user-friendly error messages in TUI
- Log errors to serial/debug output

### Panic Strategy

- Panic only on unrecoverable errors
- Panic handler prints to framebuffer
- Halt CPU on panic (don't reboot)

---

## Security Specifications

### TLS Requirements

- **TLS 1.3 only** (no fallback to older versions)
- **Certificate validation** (proper CA chain verification)
- **No self-signed certificates** (unless explicitly configured for local)

### API Key Security

- **Encryption at rest** (EFI variables or encrypted file)
- **No plaintext storage** (except during input)
- **Memory clearing** (zero out keys after use where possible)

### Network Security

- **No listening ports** (outbound connections only)
- **Firewall everything** (default deny inbound)
- **Input validation** (sanitize all user input)

---

## Documentation Requirements

### Code Documentation

- All public APIs must have doc comments
- Use `///` for public items
- Include examples where helpful

### README

**Required Sections**:
1. Overview
2. Building
3. Running (QEMU)
4. Hardware Requirements
5. Configuration
6. Troubleshooting

### API Documentation

Generate with `cargo doc --no-deps` (if possible in no_std context)

---

## Deliverables Checklist

### v0.1.0 Release

- [ ] Bootable ISO for x86_64 UEFI
- [ ] Bootable ISO for x86_64 BIOS
- [ ] Bootable ISO for ARM64 UEFI (stretch)
- [ ] Network connectivity (Ethernet)
- [ ] WiFi connectivity (MT7601U minimum)
- [ ] OpenAI API client
- [ ] Anthropic API client
- [ ] Groq API client
- [ ] xAI API client
- [ ] Local inference (SmolLM-360M)
- [ ] TUI with dark/light themes
- [ ] Syntax highlighting
- [ ] Markdown rendering
- [ ] Configuration persistence
- [ ] Setup wizard
- [ ] Documentation (README)

---

## Implementation Notes

### no_std Considerations

- **No standard library**: All code must be `#![no_std]`
- **No heap by default**: Use `alloc` crate for heap allocation
- **No async/await**: Use callbacks or polling instead
- **No file I/O**: Use EFI variables or boot partition only
- **No threading**: Single-threaded with interrupt handlers

### Memory Management

- **Static allocation preferred**: Use `const` and `static` where possible
- **Heap for dynamic data**: Conversation history, network buffers
- **Stack for small data**: Function-local variables
- **No memory leaks**: Ensure all allocations are freed

### Interrupt Safety

- **Disable interrupts during critical sections**: Use `cli`/`sti` or spinlocks
- **Minimal work in handlers**: Defer to main loop where possible
- **No allocation in handlers**: Pre-allocate buffers

---

**End of Technical Specifications**
