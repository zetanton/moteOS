//! Example demonstrating the InputWidget
//!
//! This shows how to create and use an InputWidget for text input with cursor management.

#![no_std]
#![no_main]

extern crate alloc;

use tui::{
    CursorDirection, InputWidget, Key, Widget, WidgetEvent,
};

/// Example showing basic InputWidget usage
///
/// In a real application, this would be integrated with the event loop
/// and rendering system.
fn example_input_widget() {
    // Create a new input widget with a placeholder
    let mut input = InputWidget::new("Enter your message...".into());

    // Set focus to enable editing
    input.set_focused(true);

    // Simulate user typing "Hello, World!"
    for ch in "Hello, World!".chars() {
        input.insert_char(ch);
    }

    // Current text should be "Hello, World!"
    assert!(input.get_text() == "Hello, World!");

    // Move cursor to the middle
    for _ in 0..7 {
        input.move_cursor(CursorDirection::Left);
    }

    // Delete a character
    input.delete_char();

    // Insert a character at cursor position
    input.insert_char('w');

    // Handle keyboard input events
    let event = input.handle_input(Key::Backspace);
    assert!(event == WidgetEvent::Changed);

    let event = input.handle_input(Key::Enter);
    assert!(event == WidgetEvent::Submit);

    // Clear the input
    input.clear();
    assert!(input.get_text().is_empty());
}

/// Example showing focus management
fn example_focus_management() {
    let mut input = InputWidget::new("Type here...".into());

    // Initially not focused
    assert!(!input.is_focused());

    // Set focus
    input.set_focused(true);
    assert!(input.is_focused());

    // Remove focus
    input.set_focused(false);
    assert!(!input.is_focused());
}

/// Example showing cursor movement
fn example_cursor_movement() {
    let mut input = InputWidget::new("".into());

    // Type some text
    input.set_text("abcdef".into());

    // Cursor should be at the end
    assert!(input.cursor_position() == 6);

    // Move to start
    input.move_cursor(CursorDirection::Start);
    assert!(input.cursor_position() == 0);

    // Move to end
    input.move_cursor(CursorDirection::End);
    assert!(input.cursor_position() == 6);

    // Move left and right
    input.move_cursor(CursorDirection::Left);
    input.move_cursor(CursorDirection::Left);
    assert!(input.cursor_position() == 4);

    input.move_cursor(CursorDirection::Right);
    assert!(input.cursor_position() == 5);
}

/// Example showing insertion and deletion
fn example_editing() {
    let mut input = InputWidget::new("".into());

    // Insert characters
    input.insert_char('a');
    input.insert_char('b');
    input.insert_char('c');
    assert!(input.get_text() == "abc");

    // Delete with backspace
    input.delete_char();
    assert!(input.get_text() == "ab");

    // Move cursor and insert in the middle
    input.move_cursor(CursorDirection::Left);
    input.insert_char('x');
    assert!(input.get_text() == "axb");

    // Delete forward
    input.delete_char_forward();
    assert!(input.get_text() == "ax");
}

// Note: In a no_std environment, we can't run these as normal tests.
// These would be called from the kernel's test framework or as part
// of integration tests.
