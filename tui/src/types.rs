//! Common types used throughout the TUI framework

/// A point in 2D space
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

impl Point {
    pub const fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

/// A rectangular region
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

impl Rect {
    pub const fn new(x: usize, y: usize, width: usize, height: usize) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub const fn from_point_size(point: Point, width: usize, height: usize) -> Self {
        Self {
            x: point.x,
            y: point.y,
            width,
            height,
        }
    }

    pub const fn top_left(&self) -> Point {
        Point {
            x: self.x,
            y: self.y,
        }
    }

    pub const fn bottom_right(&self) -> Point {
        Point {
            x: self.x + self.width,
            y: self.y + self.height,
        }
    }

    pub const fn contains(&self, point: Point) -> bool {
        point.x >= self.x
            && point.x < self.x + self.width
            && point.y >= self.y
            && point.y < self.y + self.height
    }

    pub const fn area(&self) -> usize {
        self.width * self.height
    }
}

/// Keyboard key representation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    /// Regular character input
    Char(char),
    /// Enter/Return key
    Enter,
    /// Backspace key
    Backspace,
    /// Delete key
    Delete,
    /// Tab key
    Tab,
    /// Escape key
    Escape,
    /// Arrow keys
    Up,
    Down,
    Left,
    Right,
    /// Function keys
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    /// Page navigation
    PageUp,
    PageDown,
    Home,
    End,
}

/// Cursor movement direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorDirection {
    Left,
    Right,
    Up,
    Down,
    Start,
    End,
}

/// Events that widgets can emit
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WidgetEvent {
    /// No event
    None,
    /// Widget wants to close/exit
    Close,
    /// Widget state changed
    Changed,
    /// Widget submitted data
    Submit,
    /// Custom event with string data
    Custom(&'static str),
}
