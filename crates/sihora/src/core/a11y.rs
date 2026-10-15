//! Accessibility tree types (platform-agnostic).
//!
//! These types describe the accessibility tree that the platform layer
//! translates into AccessKit (or other backend) nodes. Kept in `sihora-core`
//! so they compile on `wasm32-unknown-unknown`.
//!
//! # Example
//! ```
//! use sihora::core::a11y::{A11yNode, A11yRole, A11yTree};
//!
//! let root = A11yNode::new(1, A11yRole::Window).name("My App");
//! let button = A11yNode::new(2, A11yRole::Button).name("OK");
//! let tree = A11yTree {
//!     root: 1,
//!     nodes: vec![root.children(vec![2]), button],
//!     focus: Some(2),
//! };
//! assert_eq!(tree.nodes.len(), 2);
//! ```

/// Accessibility node identifier. Must be nonzero.
pub type A11yId = u64;

/// The semantic role of an accessibility node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum A11yRole {
    /// Top-level window container.
    Window,
    /// Generic grouping container.
    Group,
    /// Clickable button.
    Button,
    /// Toggle checkbox.
    CheckBox,
    /// Single-line text input field.
    TextField,
    /// Static text label.
    Label,
    /// Ordered or unordered list container.
    List,
    /// An item within a list.
    ListItem,
}

/// A single node in the accessibility tree.
#[derive(Debug, Clone)]
pub struct A11yNode {
    /// Unique identifier (must be nonzero).
    pub id: A11yId,
    /// Semantic role.
    pub role: A11yRole,
    /// Human-readable label for the node.
    pub name: Option<String>,
    /// Current value (e.g. text field content).
    pub value: Option<String>,
    /// Checked state for checkboxes.
    pub checked: Option<bool>,
    /// IDs of child nodes.
    pub child_ids: Vec<A11yId>,
    /// Whether this node can receive keyboard focus.
    pub focusable: bool,
}

impl A11yNode {
    /// Create a new accessibility node with the given id and role.
    pub fn new(id: A11yId, role: A11yRole) -> Self {
        Self {
            id,
            role,
            name: None,
            value: None,
            checked: None,
            child_ids: Vec::new(),
            focusable: false,
        }
    }

    /// Set the human-readable name (builder pattern).
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    /// Set the current value (builder pattern).
    pub fn value(mut self, value: &str) -> Self {
        self.value = Some(value.to_string());
        self
    }

    /// Set the checked state (builder pattern).
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }

    /// Set child node IDs (builder pattern).
    pub fn children(mut self, ids: Vec<A11yId>) -> Self {
        self.child_ids = ids;
        self
    }

    /// Mark this node as focusable (builder pattern).
    pub fn focusable(mut self) -> Self {
        self.focusable = true;
        self
    }
}

/// The complete accessibility tree for one frame.
#[derive(Debug, Clone)]
pub struct A11yTree {
    /// ID of the root node.
    pub root: A11yId,
    /// All nodes in the tree.
    pub nodes: Vec<A11yNode>,
    /// ID of the currently focused node, if any.
    pub focus: Option<A11yId>,
}

impl A11yTree {
    /// Create an empty tree with just a window root.
    pub fn empty(title: &str) -> Self {
        Self {
            root: 1,
            nodes: vec![A11yNode::new(1, A11yRole::Window).name(title)],
            focus: None,
        }
    }
}
