# moteOS

> *"A mote of dust, suspended in a sunbeam, connecting you to infinite intelligence."*

**moteOS** is an ultra-lightweight, AI-native unikernel operating system written in Rust. It strips computing to its absolute essence: boot, connect, converse. The entire operating system exists for a single purpose—to provide a minimal TUI interface for interacting with large language models.

## Philosophy

> *"Everything that is not the AI interface is bloat."*

- No shell
- No filesystem (beyond config)
- No package manager
- No multi-tasking
- No GUI
- Just you and the model

## Features

- **Ultra-minimal**: <10MB bootable image (without bundled model)
- **Instant boot**: Cold boot to prompt in <3 seconds
- **Universal hardware**: Run on virtually anything with a network connection
- **Multi-provider**: Support for OpenAI, Anthropic, Groq, xAI
- **Offline-capable**: Bundled SmolLM-360M for fully offline operation
- **Secure by default**: TLS-only connections, minimal attack surface
- **Beautiful TUI**: Clean, responsive, distraction-free interface

## Status

🚧 **Early Development** - This project is currently in active development.

## Testing

Ready to test moteOS? Start here:

```bash
# Quick start - run all tests
./tools/run-all-tests.sh

# Or use Make targets
make test-all
```

**Testing Resources:**
- [Quick Start Guide](TESTING_QUICKSTART.md) - Get testing in 5 minutes
- [Comprehensive Testing Plan](docs/TESTING_PLAN.md) - Full test strategy

## Documentation

- [Product Requirements Document](docs/moteOS-PRD.md)
- [Technical Specifications](docs/TECHNICAL_SPECIFICATIONS.md)
- [Testing Plan](docs/TESTING_PLAN.md)

## License

MIT License - see [LICENSE](LICENSE) for details.
