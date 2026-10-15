//! Input event types delivered by the platform.

/// Keyboard modifier state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Modifiers {
    /// Shift key is held.
    pub shift: bool,
    /// Control key is held.
    pub ctrl: bool,
    /// Alt / Option key is held.
    pub alt: bool,
    /// Super / Cmd / Windows key is held.
    pub meta: bool,
}

/// An input event delivered by the platform to the application.
#[derive(Debug, Clone)]
pub enum Event {
    /// Primary pointer button pressed at logical coordinates.
    PointerPressed {
        /// X in logical pixels.
        x: f32,
        /// Y in logical pixels.
        y: f32,
    },
    /// Primary pointer button released at logical coordinates.
    PointerReleased {
        /// X in logical pixels.
        x: f32,
        /// Y in logical pixels.
        y: f32,
    },
    /// Pointer moved to logical coordinates.
    PointerMoved {
        /// X in logical pixels.
        x: f32,
        /// Y in logical pixels.
        y: f32,
    },
    /// Scroll wheel or trackpad scroll gesture.
    Scroll {
        /// Cursor X in logical pixels.
        x: f32,
        /// Cursor Y in logical pixels.
        y: f32,
        /// Horizontal scroll delta (positive = right).
        dx: f32,
        /// Vertical scroll delta (positive = content moves up / scroll down).
        dy: f32,
    },
    /// Keyboard key pressed.
    KeyPressed {
        /// The key that was pressed.
        key: Key,
        /// Active modifier keys.
        modifiers: Modifiers,
    },
    /// Keyboard key released.
    KeyReleased {
        /// The key that was released.
        key: Key,
        /// Active modifier keys.
        modifiers: Modifiers,
    },
}

/// A keyboard key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    /// Spacebar.
    Space,
    /// Enter / Return.
    Enter,
    /// Escape.
    Escape,
    /// Tab.
    Tab,
    /// Backspace / Delete-backward.
    Backspace,
    /// Forward delete.
    Delete,
    /// Home.
    Home,
    /// End.
    End,
    /// Up arrow.
    ArrowUp,
    /// Down arrow.
    ArrowDown,
    /// Left arrow.
    ArrowLeft,
    /// Right arrow.
    ArrowRight,
    /// A character key.
    Char(char),
}
