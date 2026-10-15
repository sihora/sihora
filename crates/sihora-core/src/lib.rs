//! Core types for Sihora.

pub mod a11y;
pub mod animation;
mod color;
mod event;
pub mod focus;
pub mod reactive;
pub mod text_input;
mod tree;
mod view;

pub use color::Color;
pub use event::{Event, Key, Modifiers};
pub use tree::{LayoutRect, NodeId, NodeLayout, ViewTree};
pub use view::{Framed, Padded, View, ViewExt};
