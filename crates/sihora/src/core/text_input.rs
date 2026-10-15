//! Text editing state for input fields.
//!
//! [`TextEditState`] manages a text buffer, cursor position, and selection
//! range, with methods for all standard editing operations.
//!
//! # Example
//! ```
//! # sihora::core::reactive::reset();
//! use sihora::core::text_input::TextEditState;
//!
//! let state = TextEditState::new();
//! state.insert('h');
//! state.insert('i');
//! assert_eq!(state.text(), "hi");
//! assert_eq!(state.cursor(), 2);
//! ```

use crate::core::reactive::{create_signal, get, set, Signal};

/// Manages the editing state of a text input field.
///
/// Stores text content, cursor position, and selection as reactive signals.
/// All operations are char-boundary-aware for correct UTF-8 handling.
pub struct TextEditState {
    text: Signal<String>,
    cursor: Signal<usize>,
    selection_anchor: Signal<Option<usize>>,
}

impl Default for TextEditState {
    fn default() -> Self {
        Self::new()
    }
}

impl TextEditState {
    /// Create an empty text edit state.
    pub fn new() -> Self {
        Self {
            text: create_signal(String::new()),
            cursor: create_signal(0),
            selection_anchor: create_signal(None),
        }
    }

    /// Create a text edit state with initial content.
    pub fn with_text(s: &str) -> Self {
        let len = s.len();
        Self {
            text: create_signal(s.to_string()),
            cursor: create_signal(len),
            selection_anchor: create_signal(None),
        }
    }

    /// The current text content.
    pub fn text(&self) -> String {
        get(self.text)
    }

    /// The cursor byte offset within the text.
    pub fn cursor(&self) -> usize {
        get(self.cursor)
    }

    /// Selection range `(start, end)` in byte offsets, or `None` if no selection.
    pub fn selection(&self) -> Option<(usize, usize)> {
        get(self.selection_anchor).map(|anchor| {
            let cursor = get(self.cursor);
            if anchor <= cursor {
                (anchor, cursor)
            } else {
                (cursor, anchor)
            }
        })
    }

    /// Insert a character at the cursor position, replacing any selection.
    pub fn insert(&self, ch: char) {
        self.delete_selection_inner();
        let mut text = get(self.text);
        let cursor = get(self.cursor);
        text.insert(cursor, ch);
        let new_cursor = cursor + ch.len_utf8();
        set(self.text, text);
        set(self.cursor, new_cursor);
    }

    /// Insert a string at the cursor position, replacing any selection.
    pub fn insert_str(&self, s: &str) {
        self.delete_selection_inner();
        let mut text = get(self.text);
        let cursor = get(self.cursor);
        text.insert_str(cursor, s);
        let new_cursor = cursor + s.len();
        set(self.text, text);
        set(self.cursor, new_cursor);
    }

    /// Delete the character before the cursor (backspace).
    pub fn delete_backward(&self) {
        if self.delete_selection_inner() {
            return;
        }
        let mut text = get(self.text);
        let cursor = get(self.cursor);
        if cursor > 0 {
            let prev = prev_char_boundary(&text, cursor);
            text.drain(prev..cursor);
            set(self.text, text);
            set(self.cursor, prev);
        }
    }

    /// Delete the character after the cursor (forward delete).
    pub fn delete_forward(&self) {
        if self.delete_selection_inner() {
            return;
        }
        let mut text = get(self.text);
        let cursor = get(self.cursor);
        if cursor < text.len() {
            let next = next_char_boundary(&text, cursor);
            text.drain(cursor..next);
            set(self.text, text);
        }
    }

    /// Move cursor one character left.
    pub fn move_left(&self, extend_selection: bool) {
        let text = get(self.text);
        let cursor = get(self.cursor);
        if cursor > 0 {
            self.update_selection_anchor(extend_selection);
            let prev = prev_char_boundary(&text, cursor);
            set(self.cursor, prev);
            if !extend_selection {
                set(self.selection_anchor, None);
            }
        }
    }

    /// Move cursor one character right.
    pub fn move_right(&self, extend_selection: bool) {
        let text = get(self.text);
        let cursor = get(self.cursor);
        if cursor < text.len() {
            self.update_selection_anchor(extend_selection);
            let next = next_char_boundary(&text, cursor);
            set(self.cursor, next);
            if !extend_selection {
                set(self.selection_anchor, None);
            }
        }
    }

    /// Move cursor to the beginning of the text.
    pub fn move_home(&self, extend_selection: bool) {
        self.update_selection_anchor(extend_selection);
        set(self.cursor, 0);
        if !extend_selection {
            set(self.selection_anchor, None);
        }
    }

    /// Move cursor to the end of the text.
    pub fn move_end(&self, extend_selection: bool) {
        let text = get(self.text);
        self.update_selection_anchor(extend_selection);
        set(self.cursor, text.len());
        if !extend_selection {
            set(self.selection_anchor, None);
        }
    }

    /// Select all text.
    pub fn select_all(&self) {
        let text = get(self.text);
        set(self.selection_anchor, Some(0));
        set(self.cursor, text.len());
    }

    /// Replace all text and move cursor to end.
    pub fn set_text(&self, s: &str) {
        set(self.text, s.to_string());
        set(self.cursor, s.len());
        set(self.selection_anchor, None);
    }

    /// Clear text, cursor, and selection.
    pub fn clear(&self) {
        set(self.text, String::new());
        set(self.cursor, 0);
        set(self.selection_anchor, None);
    }

    /// The number of characters (not bytes) in the text.
    pub fn char_count(&self) -> usize {
        get(self.text).chars().count()
    }

    /// Return the cursor position as a character index (for rendering).
    pub fn cursor_char_index(&self) -> usize {
        let text = get(self.text);
        let cursor = get(self.cursor);
        text[..cursor].chars().count()
    }

    fn update_selection_anchor(&self, extend: bool) {
        if extend && get(self.selection_anchor).is_none() {
            set(self.selection_anchor, Some(get(self.cursor)));
        }
    }

    fn delete_selection_inner(&self) -> bool {
        if let Some((start, end)) = self.selection() {
            let mut text = get(self.text);
            text.drain(start..end);
            set(self.text, text);
            set(self.cursor, start);
            set(self.selection_anchor, None);
            true
        } else {
            false
        }
    }
}

fn prev_char_boundary(text: &str, pos: usize) -> usize {
    let mut i = pos.saturating_sub(1);
    while i > 0 && !text.is_char_boundary(i) {
        i -= 1;
    }
    i
}

fn next_char_boundary(text: &str, pos: usize) -> usize {
    let mut i = pos + 1;
    while i < text.len() && !text.is_char_boundary(i) {
        i += 1;
    }
    i
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::reactive::reset;

    #[test]
    fn insert_and_read() {
        reset();
        let s = TextEditState::new();
        s.insert('a');
        s.insert('b');
        s.insert('c');
        assert_eq!(s.text(), "abc");
        assert_eq!(s.cursor(), 3);
    }

    #[test]
    fn backspace() {
        reset();
        let s = TextEditState::new();
        s.insert('a');
        s.insert('b');
        s.delete_backward();
        assert_eq!(s.text(), "a");
        assert_eq!(s.cursor(), 1);
    }

    #[test]
    fn forward_delete() {
        reset();
        let s = TextEditState::with_text("abc");
        s.move_home(false);
        s.delete_forward();
        assert_eq!(s.text(), "bc");
    }

    #[test]
    fn cursor_movement() {
        reset();
        let s = TextEditState::with_text("hello");
        assert_eq!(s.cursor(), 5);
        s.move_left(false);
        assert_eq!(s.cursor(), 4);
        s.move_right(false);
        assert_eq!(s.cursor(), 5);
        s.move_home(false);
        assert_eq!(s.cursor(), 0);
        s.move_end(false);
        assert_eq!(s.cursor(), 5);
    }

    #[test]
    fn selection() {
        reset();
        let s = TextEditState::with_text("hello");
        s.move_home(false);
        s.move_right(true);
        s.move_right(true);
        assert_eq!(s.selection(), Some((0, 2)));
    }

    #[test]
    fn delete_selection() {
        reset();
        let s = TextEditState::with_text("hello");
        s.select_all();
        s.delete_backward();
        assert_eq!(s.text(), "");
        assert_eq!(s.cursor(), 0);
    }

    #[test]
    fn insert_replaces_selection() {
        reset();
        let s = TextEditState::with_text("hello");
        s.select_all();
        s.insert('x');
        assert_eq!(s.text(), "x");
    }

    #[test]
    fn utf8_multibyte() {
        reset();
        let s = TextEditState::new();
        s.insert('ä');
        s.insert('ö');
        assert_eq!(s.text(), "äö");
        s.delete_backward();
        assert_eq!(s.text(), "ä");
        assert_eq!(s.cursor(), 2); // ä is 2 bytes
    }

    #[test]
    fn clear() {
        reset();
        let s = TextEditState::with_text("test");
        s.clear();
        assert_eq!(s.text(), "");
        assert_eq!(s.cursor(), 0);
        assert_eq!(s.selection(), None);
    }
}
