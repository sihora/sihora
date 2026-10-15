//! Focus management for keyboard navigation.
//!
//! [`FocusManager`] tracks which element has focus and handles Tab cycling.
//!
//! # Example
//! ```
//! # sihora::core::reactive::reset();
//! use sihora::core::focus::FocusManager;
//!
//! let focus = FocusManager::new();
//! assert_eq!(focus.focused(), None);
//!
//! focus.handle_tab(3, false);
//! assert_eq!(focus.focused(), Some(0));
//! assert!(focus.is_focused(0));
//!
//! focus.handle_tab(3, false);
//! assert_eq!(focus.focused(), Some(1));
//! ```

use crate::core::reactive::{create_signal, get, set, Signal};

/// Manages keyboard focus across a set of focusable elements.
///
/// Elements are identified by a `usize` index. The total count of focusable
/// elements is passed to [`handle_tab`](Self::handle_tab) so the list can
/// grow or shrink dynamically (e.g. when list items are added/removed).
pub struct FocusManager {
    focused: Signal<Option<usize>>,
}

impl Default for FocusManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FocusManager {
    /// Create a new focus manager with no element focused.
    pub fn new() -> Self {
        Self {
            focused: create_signal(None),
        }
    }

    /// The currently focused element index, or `None`.
    pub fn focused(&self) -> Option<usize> {
        get(self.focused)
    }

    /// Whether the element at `index` currently has focus.
    pub fn is_focused(&self, index: usize) -> bool {
        get(self.focused) == Some(index)
    }

    /// Set focus to the element at `index`.
    pub fn focus(&self, index: usize) {
        set(self.focused, Some(index));
    }

    /// Clear focus so no element is focused.
    pub fn blur(&self) {
        set(self.focused, None);
    }

    /// Move focus to the next (or previous if `reverse`) element.
    ///
    /// `count` is the total number of focusable elements. If nothing is
    /// focused, focuses the first (or last) element.
    pub fn handle_tab(&self, count: usize, reverse: bool) {
        if count == 0 {
            return;
        }
        let next = match get(self.focused) {
            None => {
                if reverse {
                    count - 1
                } else {
                    0
                }
            }
            Some(i) => {
                if reverse {
                    if i == 0 {
                        count - 1
                    } else {
                        i - 1
                    }
                } else if i + 1 >= count {
                    0
                } else {
                    i + 1
                }
            }
        };
        set(self.focused, Some(next));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::reactive::reset;

    #[test]
    fn initial_state() {
        reset();
        let fm = FocusManager::new();
        assert_eq!(fm.focused(), None);
        assert!(!fm.is_focused(0));
    }

    #[test]
    fn tab_forward() {
        reset();
        let fm = FocusManager::new();
        fm.handle_tab(3, false);
        assert_eq!(fm.focused(), Some(0));
        fm.handle_tab(3, false);
        assert_eq!(fm.focused(), Some(1));
        fm.handle_tab(3, false);
        assert_eq!(fm.focused(), Some(2));
        fm.handle_tab(3, false);
        assert_eq!(fm.focused(), Some(0));
    }

    #[test]
    fn tab_reverse() {
        reset();
        let fm = FocusManager::new();
        fm.handle_tab(3, true);
        assert_eq!(fm.focused(), Some(2));
        fm.handle_tab(3, true);
        assert_eq!(fm.focused(), Some(1));
        fm.handle_tab(3, true);
        assert_eq!(fm.focused(), Some(0));
        fm.handle_tab(3, true);
        assert_eq!(fm.focused(), Some(2));
    }

    #[test]
    fn focus_and_blur() {
        reset();
        let fm = FocusManager::new();
        fm.focus(2);
        assert!(fm.is_focused(2));
        fm.blur();
        assert_eq!(fm.focused(), None);
    }

    #[test]
    fn zero_count_is_noop() {
        reset();
        let fm = FocusManager::new();
        fm.handle_tab(0, false);
        assert_eq!(fm.focused(), None);
    }
}
