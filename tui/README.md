# TUI Framework

The TUI (Terminal User Interface) framework for moteOS provides a complete color system, theming, and rendering capabilities for the framebuffer-based interface.

## Features

- **24-bit RGB Color System**: Full color support with hex parsing
- **Dark & Light Themes**: Pre-defined themes matching the PRD specifications
- **Provider Brand Colors**: Dedicated colors for each LLM provider
- **Color Blending**: Alpha blending support for transparency effects
- **No-std Compatible**: Works in kernel space without standard library

## Color System

The `Color` struct represents a 24-bit RGB color with alpha channel:

```rust
use tui::Color;

// Create colors directly
let red = Color::new(255, 0, 0);
let semi_transparent = Color::new_rgba(255, 0, 0, 128);

// Parse from hex strings
let blue = Color::from_hex("#0000FF").unwrap();
let green = Color::from_hex("#0F0").unwrap(); // Short form

// Convert to tuples
let (r, g, b) = blue.to_rgb();
let (r, g, b, a) = semi_transparent.to_rgba();

// Blend colors
let purple = red.blend(blue, 0.5);
```

## Dark Theme

The dark theme follows GitHub Dark color scheme:

### Background Colors
- **Background**: `#0D1117` - Main background
- **Surface**: `#161B22` - Elevated surfaces (cards, modals)
- **Border**: `#21262D` - Borders and separators

### Text Colors
- **Text Primary**: `#F0F6FC` - Main text
- **Text Secondary**: `#C9D1D9` - Secondary text (timestamps, hints)
- **Text Tertiary**: `#8B949E` - Tertiary text (disabled items)
- **Text Disabled**: `#484F58` - Disabled text

### Accent Colors
- **Accent Primary**: `#58A6FF` - Links, focused elements
- **Accent Success**: `#7EE787` - Success states, confirmations
- **Accent Warning**: `#FFA657` - Warnings, alerts
- **Accent Error**: `#FF7B72` - Errors, critical states
- **Accent Assistant**: `#A371F7` - Assistant messages
- **Accent Code**: `#79C0FF` - Code blocks, syntax highlighting

### Provider Brand Colors
- **OpenAI**: `#10A37F`
- **Anthropic**: `#D4A574`
- **Groq**: `#F55036`
- **xAI**: `#FFFFFF` (white for dark theme)
- **Local**: `#7C3AED`

## Light Theme

The light theme follows GitHub Light color scheme:

### Background Colors
- **Background**: `#FFFFFF` - Main background
- **Surface**: `#F6F8FA` - Elevated surfaces
- **Border**: `#D0D7DE` - Borders and separators

### Text Colors
- **Text Primary**: `#1F2328` - Main text
- **Text Secondary**: `#424A53` - Secondary text
- **Text Tertiary**: `#656D76` - Tertiary text
- **Text Disabled**: `#8C959F` - Disabled text

### Accent Colors
- **Accent Primary**: `#0969DA` - Links, focused elements
- **Accent Success**: `#1A7F37` - Success states
- **Accent Warning**: `#9A6700` - Warnings
- **Accent Error**: `#CF222E` - Errors
- **Accent Assistant**: `#8250DF` - Assistant messages
- **Accent Code**: `#0550AE` - Code blocks

### Provider Brand Colors
Same as dark theme, except:
- **xAI**: `#000000` (black for light theme)

## Usage Examples

### Basic Theme Usage

```rust
use tui::{DARK_THEME, LIGHT_THEME};

// Use the dark theme
let theme = &DARK_THEME;

// Access colors
let bg = theme.background;
let text = theme.text_primary;
let accent = theme.accent_assistant;
```

### Rendering a Chat Message

```rust
fn render_message(theme: &Theme, is_assistant: bool) {
    let text_color = if is_assistant {
        theme.accent_assistant
    } else {
        theme.text_primary
    };

    let bg_color = theme.surface;
    let border_color = theme.border;

    // Render background rectangle with bg_color
    // Render text with text_color
    // Draw border with border_color
}
```

### Provider Badge

```rust
fn render_provider(theme: &Theme, provider: &str) {
    let badge_color = match provider {
        "openai" => theme.provider_openai,
        "anthropic" => theme.provider_anthropic,
        "groq" => theme.provider_groq,
        "xai" => theme.provider_xai,
        "local" => theme.provider_local,
        _ => theme.accent_primary,
    };

    // Render badge with badge_color
}
```

### Status Indicators

```rust
fn render_status(theme: &Theme, status: &str) {
    let color = match status {
        "success" => theme.accent_success,
        "warning" => theme.accent_warning,
        "error" => theme.accent_error,
        _ => theme.text_secondary,
    };

    // Render status indicator with color
}
```

## Testing

The TUI framework includes comprehensive tests to verify color parsing and theme correctness:

```bash
cargo test -p tui
```

Tests cover:
- Color hex parsing (short and long form)
- Theme color verification against PRD
- RGB component extraction
- Color blending
- Contrast verification

## Implementation Details

### Color Parsing

The `Color::from_hex` method supports:
- Long form: `#RRGGBB` (e.g., `#FF0000`)
- Short form: `#RGB` (e.g., `#F00`)
- With or without `#` prefix
- Case-insensitive hex digits

All parsing is done at compile time for the theme constants using the `hex_color!` macro.

### No-std Compatibility

The entire TUI framework is `#![no_std]` compatible:
- No heap allocations for color parsing
- All theme constants are compile-time computed
- No dependencies on standard library

### Performance

- Color parsing is zero-cost for theme constants (compile-time)
- Runtime color parsing is fast (simple byte operations)
- Blending uses basic floating-point math (can be optimized with SIMD)

## Future Enhancements

Planned additions to the TUI framework:
- [ ] Framebuffer rendering primitives
- [ ] Font system (PSF2 support)
- [ ] Widget system
- [ ] Layout engine
- [ ] Syntax highlighting
- [ ] Markdown rendering
- [ ] Animation support

## References

- Technical Specifications: `docs/TECHNICAL_SPECIFICATIONS.md` Section 3.3-3.5
- Product Requirements: Color palette definitions
- GitHub Dark/Light themes (inspiration)
