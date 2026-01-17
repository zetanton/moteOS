# moteOS Kernel

The kernel module provides the main entry point and event loop for moteOS.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        kernel_main()                         │
│                    (Entry Point)                             │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
              ┌──────────────────────┐
              │  Initialize System   │
              │  - Heap allocator    │
              │  - Load config       │
              │  - Setup state       │
              └──────────┬───────────┘
                         │
                         ▼
              ┌──────────────────────┐
              │    main_loop()       │
              │   (Event Loop)       │
              └──────────┬───────────┘
                         │
                         │ Loop forever
                         ▼
        ┌────────────────┼────────────────┐
        │                │                │
        ▼                ▼                ▼
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│ Input        │  │ Network      │  │ Screen       │
│ Handling     │  │ Polling      │  │ Update       │
├──────────────┤  ├──────────────┤  ├──────────────┤
│ - Read kbd   │  │ - Poll stack │  │ - Render UI  │
│ - Process    │  │ - Handle RX  │  │ - Display    │
│ - Dispatch   │  │ - Send TX    │  │ - Present    │
└──────────────┘  └──────────────┘  └──────────────┘
        │                │                │
        └────────────────┴────────────────┘
                         │
                         ▼
                  Sleep ~16ms
                         │
                         │
                    (repeat)
```

## Module Structure

- **lib.rs** - Main entry point, kernel_main(), panic handler, global state
- **init.rs** - System initialization (heap, network, LLM)
- **event_loop.rs** - Main event loop implementation
- **input.rs** - Keyboard input handling and dispatch
- **screen.rs** - Screen rendering and updates

## Key Features

### 1. No-Std Operation
All code is `#![no_std]` compatible for bare-metal operation.

### 2. Global State Management
Uses `spin::Mutex` for thread-safe access to kernel state.

### 3. Event-Driven Architecture
Non-blocking polling design with clear separation of concerns.

### 4. Platform Support
- x86_64 (UEFI, BIOS)
- aarch64 (UEFI)

## Integration Points

### Completed
- ✅ Boot module (BootInfo)
- ✅ Config module (MoteConfig, storage)
- ✅ LLM module (traits)

### Pending
- ⏳ TUI framework (rendering)
- ⏳ Network stack (polling)
- ⏳ Keyboard drivers (PS/2, USB)
- ⏳ Timer (interrupts)

## Usage

The kernel is designed to be linked with the bootloader and other system components.

### Build
```bash
cargo build --release -p kernel
```

### Test in QEMU
```bash
qemu-system-x86_64 \
    -machine q35 \
    -cpu qemu64 \
    -m 1G \
    -drive if=pflash,format=raw,file=OVMF.fd \
    -drive format=raw,file=moteos-x64.iso \
    -netdev user,id=net0 \
    -device virtio-net,netdev=net0
```

## Configuration

The kernel loads configuration from EFI variables or file storage:
- Provider settings (API keys)
- Network configuration
- User preferences (theme, temperature, etc.)

If no configuration exists, it launches the setup wizard.

## Event Loop

The main event loop runs at ~60 FPS (16ms sleep) and:

1. **Handles Input** - Processes keyboard events
2. **Polls Network** - Services network stack
3. **Updates Screen** - Renders current state
4. **Sleeps** - Maintains frame rate

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| F1  | Help |
| F2  | Provider Select |
| F3  | Model Select |
| F4  | Configuration |
| F9  | New Chat |
| F10 | Shutdown |
| ESC | Return to Chat |

## State Management

Global state includes:
- Current configuration
- Conversation history
- Setup completion status

Access is synchronized via Mutex for safety.

## Error Handling

The kernel includes a panic handler that:
- Halts the CPU safely
- Will render panic info to screen (TODO)
- Prevents undefined behavior

## Next Steps

1. Integrate TUI framework for rendering
2. Add keyboard driver support
3. Complete network polling integration
4. Implement timer interrupts
5. Add LLM provider instantiation

## License

MIT License - See LICENSE file for details
