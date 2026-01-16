#!/usr/bin/env node
/**
 * Vibe Kanban Task Creator
 * 
 * Creates tasks in Vibe Kanban without starting them.
 * Uses the MCP server API (requires Vibe Kanban to be running).
 * 
 * Usage:
 *   node tools/create-vk-tasks.js
 * 
 * Or via MCP client:
 *   Use the MCP tools: create_task, list_projects, list_tasks
 */

const tasks = [
  // Workstream 1: Boot & Core
  {
    workstream: 1,
    title: "[1] Project scaffolding (Cargo workspace structure)",
    description: `Create workspace Cargo.toml
- Set up crate structure: boot/, shared/
- Configure no_std for all crates
- See docs/TECHNICAL_SPECIFICATIONS.md Section 3.1`,
    tags: ["workstream-1", "priority-p0", "foundation", "blocking"],
    priority: "P0"
  },
  {
    workstream: 1,
    title: "[1] x86_64 UEFI boot entry point",
    description: `Implement UEFI entry point using uefi-rs
- Acquire framebuffer via Graphics Output Protocol
- Get memory map and exit boot services
- Jump to kernel_main()
- See docs/TECHNICAL_SPECIFICATIONS.md Section 3.1.2`,
    tags: ["workstream-1", "priority-p0", "foundation", "blocking"],
    priority: "P0"
  },
  {
    workstream: 1,
    title: "[1] Memory allocator setup",
    description: `Integrate linked_list_allocator
- Initialize heap from memory map
- Register global allocator
- Test allocation/deallocation
- See docs/TECHNICAL_SPECIFICATIONS.md Section 3.1.5`,
    tags: ["workstream-1", "priority-p0", "foundation", "blocking"],
    priority: "P0"
  },
  {
    workstream: 1,
    title: "[1] Interrupt setup (x86_64)",
    description: `Create Interrupt Descriptor Table (IDT)
- Implement timer interrupt handler
- Implement keyboard interrupt handler
- Implement panic handler
- See docs/TECHNICAL_SPECIFICATIONS.md Section 3.1.6`,
    tags: ["workstream-1", "priority-p0", "foundation"],
    priority: "P0"
  },
  {
    workstream: 1,
    title: "[1] Framebuffer interface",
    description: `Create FramebufferInfo struct
- Implement safe wrapper functions
- Basic pixel writing functions
- Test text rendering
- See docs/TECHNICAL_SPECIFICATIONS.md Section 3.1.7`,
    tags: ["workstream-1", "priority-p0", "foundation"],
    priority: "P0"
  },
  {
    workstream: 1,
    title: "[1] Timer setup",
    description: `Configure HPET or APIC timer
- Implement sleep_ms() function
- Test timer interrupts
- See docs/TECHNICAL_SPECIFICATIONS.md Section 3.1.8`,
    tags: ["workstream-1", "priority-p0", "foundation"],
    priority: "P0"
  },

  // Workstream 2: Network Stack
  {
    workstream: 2,
    title: "[2] virtio-net driver",
    description: `PCI device discovery
- Virtio queue setup
- Packet transmission/reception
- Interrupt handling
- See docs/TECHNICAL_SPECIFICATIONS.md Section 3.2.3`,
    tags: ["workstream-2", "priority-p0", "feature"],
    priority: "P0",
    depends_on: "[1] Memory allocator setup"
  },
  {
    workstream: 2,
    title: "[2] smoltcp integration",
    description: `Create NetworkStack struct
- Implement NetworkDriver trait
- Set up interface, neighbor cache, routes
- Implement polling mechanism
- See docs/TECHNICAL_SPECIFICATIONS.md Section 3.2.6`,
    tags: ["workstream-2", "priority-p0", "feature"],
    priority: "P0",
    depends_on: "[2] virtio-net driver"
  },
  {
    workstream: 2,
    title: "[2] DHCP client",
    description: `Create DHCP socket
- Implement Discover/Offer/Request/Ack flow
- Configure interface with IP, gateway, DNS
- See docs/TECHNICAL_SPECIFICATIONS.md Section 3.2.7`,
    tags: ["workstream-2", "priority-p0", "feature"],
    priority: "P0",
    depends_on: "[2] smoltcp integration"
  },
  {
    workstream: 2,
    title: "[2] DNS resolver",
    description: `Create UDP socket
- Send DNS queries (A records)
- Parse DNS responses
- See docs/TECHNICAL_SPECIFICATIONS.md Section 3.2.8`,
    tags: ["workstream-2", "priority-p0", "feature"],
    priority: "P0",
    depends_on: "[2] DHCP client"
  },
  {
    workstream: 2,
    title: "[2] TLS 1.3 support",
    description: `Integrate rustls or implement minimal TLS
- TLS handshake
- Certificate verification
- Record layer encryption/decryption
- See docs/TECHNICAL_SPECIFICATIONS.md Section 3.2.10`,
    tags: ["workstream-2", "priority-p0", "feature"],
    priority: "P0",
    depends_on: "[2] TCP connection management"
  },
  {
    workstream: 2,
    title: "[2] HTTP/1.1 client",
    description: `Parse URLs
- Build HTTP requests
- Parse HTTP responses
- Handle headers and body
- See docs/TECHNICAL_SPECIFICATIONS.md Section 3.2.11`,
    tags: ["workstream-2", "priority-p0", "feature"],
    priority: "P0",
    depends_on: "[2] TLS 1.3 support"
  },

  // Workstream 3: TUI Framework
  {
    workstream: 3,
    title: "[3] Framebuffer rendering",
    description: `Implement pixel writing functions
- Implement rectangle filling
- Implement line drawing
- Safe wrapper functions
- See docs/TECHNICAL_SPECIFICATIONS.md Section 3.3.2`,
    tags: ["workstream-3", "priority-p0", "feature"],
    priority: "P0",
    depends_on: "[1] Framebuffer interface"
  },
  {
    workstream: 3,
    title: "[3] Font system",
    description: `Load PSF font format
- Implement glyph rendering
- Text rendering function
- See docs/TECHNICAL_SPECIFICATIONS.md Section 3.3.3`,
    tags: ["workstream-3", "priority-p0", "feature"],
    priority: "P0",
    depends_on: "[3] Framebuffer rendering"
  },
  {
    workstream: 3,
    title: "[3] Dark theme implementation",
    description: `Define all colors from PRD
- Create DARK_THEME constant
- Test color rendering
- See docs/TECHNICAL_SPECIFICATIONS.md Section 3.3.5`,
    tags: ["workstream-3", "priority-p0", "feature"],
    priority: "P0",
    depends_on: "[3] Color system"
  },
  {
    workstream: 3,
    title: "[3] Input widget",
    description: `Text input with cursor
- Character insertion/deletion
- Focus management
- See docs/TECHNICAL_SPECIFICATIONS.md Section 3.3.7`,
    tags: ["workstream-3", "priority-p0", "feature"],
    priority: "P0",
    depends_on: "[3] Base widget system"
  },
  {
    workstream: 3,
    title: "[3] Chat screen",
    description: `Message list with scrolling
- Input area
- Status bar
- Hotkey bar
- See docs/TECHNICAL_SPECIFICATIONS.md Section 3.3.12`,
    tags: ["workstream-3", "priority-p0", "feature"],
    priority: "P0",
    depends_on: "[3] Message widget"
  },

  // Workstream 4: LLM API Clients
  {
    workstream: 4,
    title: "[4] Common types and LLM provider trait",
    description: `Define Message struct, Role enum
- GenerationConfig struct
- ModelInfo struct
- LlmProvider trait
- See docs/TECHNICAL_SPECIFICATIONS.md Section 3.4.2-3`,
    tags: ["workstream-4", "priority-p0", "feature"],
    priority: "P0"
  },
  {
    workstream: 4,
    title: "[4] OpenAI client with streaming",
    description: `API endpoint configuration
- Request/response handling
- SSE streaming parser
- Error handling
- See docs/TECHNICAL_SPECIFICATIONS.md Section 3.4.4`,
    tags: ["workstream-4", "priority-p0", "feature"],
    priority: "P0",
    depends_on: "[2] HTTP/1.1 client"
  },
  {
    workstream: 4,
    title: "[4] Anthropic client with streaming",
    description: `API endpoint configuration
- Request/response handling
- SSE streaming parser
- Error handling
- See docs/TECHNICAL_SPECIFICATIONS.md Section 3.4.5`,
    tags: ["workstream-4", "priority-p0", "feature"],
    priority: "P0",
    depends_on: "[2] HTTP/1.1 client"
  },
  {
    workstream: 4,
    title: "[4] Groq client with streaming",
    description: `API endpoint configuration
- Request/response handling
- SSE streaming parser
- See docs/TECHNICAL_SPECIFICATIONS.md Section 3.4.6`,
    tags: ["workstream-4", "priority-p0", "feature"],
    priority: "P0",
    depends_on: "[2] HTTP/1.1 client"
  },
  {
    workstream: 4,
    title: "[4] xAI client with streaming",
    description: `API endpoint configuration
- Request/response handling
- SSE streaming parser
- See docs/TECHNICAL_SPECIFICATIONS.md Section 3.4.7`,
    tags: ["workstream-4", "priority-p0", "feature"],
    priority: "P0",
    depends_on: "[2] HTTP/1.1 client"
  },

  // Workstream 6: Local Inference
  {
    workstream: 6,
    title: "[6] GGUF file format parser",
    description: `Parse GGUF header
- Extract metadata
- Load tensor data
- See docs/TECHNICAL_SPECIFICATIONS.md Section 3.6.2`,
    tags: ["workstream-6", "priority-p0", "feature"],
    priority: "P0"
  },
  {
    workstream: 6,
    title: "[6] BPE tokenizer",
    description: `Load vocab from GGUF
- Encode/decode functions
- Special token handling
- See docs/TECHNICAL_SPECIFICATIONS.md Section 3.6.4`,
    tags: ["workstream-6", "priority-p0", "feature"],
    priority: "P0",
    depends_on: "[6] GGUF file format parser"
  },
  {
    workstream: 6,
    title: "[6] Transformer forward pass",
    description: `Attention layer
- FFN layer
- Layer-by-layer processing
- See docs/TECHNICAL_SPECIFICATIONS.md Section 3.6.6`,
    tags: ["workstream-6", "priority-p0", "feature"],
    priority: "P0",
    depends_on: "[6] Tensor operations"
  },
  {
    workstream: 6,
    title: "[6] Generation loop with streaming",
    description: `Prefill phase
- Generation loop
- Streaming tokens
- Stop conditions
- See docs/TECHNICAL_SPECIFICATIONS.md Section 3.6.9`,
    tags: ["workstream-6", "priority-p0", "feature"],
    priority: "P0",
    depends_on: "[6] Transformer forward pass"
  },

  // Workstream 7: Config System
  {
    workstream: 7,
    title: "[7] TOML parser (no_std)",
    description: `Minimal TOML parser
- Key-value pairs
- Nested tables
- Arrays
- See docs/TECHNICAL_SPECIFICATIONS.md Section 3.7.3`,
    tags: ["workstream-7", "priority-p1", "feature"],
    priority: "P1"
  },
  {
    workstream: 7,
    title: "[7] EFI variable storage",
    description: `Read/write EFI variables
- Config serialization
- Config deserialization
- See docs/TECHNICAL_SPECIFICATIONS.md Section 3.7.5`,
    tags: ["workstream-7", "priority-p1", "feature"],
    priority: "P1",
    depends_on: "[7] TOML parser (no_std)"
  },
  {
    workstream: 7,
    title: "[7] Setup wizard",
    description: `State machine
- Network configuration UI
- API key input UI
- Validation
- See docs/TECHNICAL_SPECIFICATIONS.md Section 3.7.8`,
    tags: ["workstream-7", "priority-p1", "feature"],
    priority: "P1",
    depends_on: "[7] EFI variable storage"
  },

  // Workstream 8: Integration
  {
    workstream: 8,
    title: "[8] Cargo workspace setup",
    description: `Root Cargo.toml
- Workspace members
- Shared dependencies
- See docs/TECHNICAL_SPECIFICATIONS.md Section 3.8.1`,
    tags: ["workstream-8", "priority-p0", "foundation"],
    priority: "P0"
  },
  {
    workstream: 8,
    title: "[8] Kernel entry point and main loop",
    description: `kernel_main() function
- Component initialization
- Input polling
- Network polling
- Screen updates
- See docs/TECHNICAL_SPECIFICATIONS.md Section 3.8.2-5`,
    tags: ["workstream-8", "priority-p0", "feature"],
    priority: "P0",
    depends_on: "[1] Memory allocator setup"
  },
  {
    workstream: 8,
    title: "[8] Component integration",
    description: `Network + LLM clients
- TUI + LLM streaming
- Config + providers
- Provider switching
- See docs/TECHNICAL_SPECIFICATIONS.md Section 3.8.6`,
    tags: ["workstream-8", "priority-p0", "feature"],
    priority: "P0",
    depends_on: "[4] OpenAI client with streaming"
  },
  {
    workstream: 8,
    title: "[8] ISO generation and QEMU testing",
    description: `UEFI boot ISO
- BIOS boot ISO
- Boot test
- Network test
- API test
- See docs/TECHNICAL_SPECIFICATIONS.md Section 3.8.8-9`,
    tags: ["workstream-8", "priority-p0", "polish"],
    priority: "P0",
    depends_on: "[8] Component integration"
  }
];

// MCP Client Example
// This shows how to use the MCP API to create tasks
// You'll need to connect to the Vibe Kanban MCP server

console.log(`
Vibe Kanban Task Creator
========================

This script contains ${tasks.length} tasks ready to be created in Vibe Kanban.

To use the MCP API:

1. Start Vibe Kanban: npx vibe-kanban
2. The MCP server will be available locally
3. Use an MCP client to call these tools:

   - list_projects: Get project_id for moteOS
   - create_task: Create each task (does NOT start it)
   - start_task_attempt: Later, start work on a task

Example MCP call:
{
  "tool": "create_task",
  "parameters": {
    "project_id": "proj-abc123",
    "title": "[1] Project scaffolding",
    "description": "Create workspace Cargo.toml...",
    "tags": ["workstream-1", "priority-p0"]
  }
}

Tasks are created in "To Do" status and will NOT start automatically.
Call start_task_attempt later to begin work.

Tasks to create:
`);

tasks.forEach((task, index) => {
  console.log(`${index + 1}. ${task.title} (${task.tags.join(', ')})`);
  if (task.depends_on) {
    console.log(`   Depends on: ${task.depends_on}`);
  }
});

console.log(`
Export format (JSON):
====================
`);

console.log(JSON.stringify(tasks, null, 2));
