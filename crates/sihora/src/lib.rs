//! Sihora — declarative cross-platform UI framework in pure Rust.
//!
//! Facade crate that re-exports the public API from internal crates.

pub use sihora_core::a11y;
pub use sihora_core::animation;
pub use sihora_core::focus;
pub use sihora_core::reactive;
pub use sihora_core::text_input;
pub use sihora_core::{
    Color, Event, Framed, Key, LayoutRect, Modifiers, NodeId, Padded, View, ViewExt, ViewTree,
};
pub use sihora_data as data;
pub use sihora_layout::{Alignment, Constraints, EdgeInsets, HAlign, Point, Size, VAlign};
pub use sihora_platform::{run, run_app, run_reactive, run_with_text, AppFrame, TextContext};
pub use sihora_primitives::{hit_test, HStack, SizedBox, VStack, ZStack};
pub use sihora_render::{DrawCommand, TextDraw};
#[cfg(not(target_arch = "wasm32"))]
pub use sihora_text::TextBuffer;
pub use sihora_theme::{self as theme, ColorScheme, Theme};
pub use sihora_widgets as widgets;

#[cfg(target_os = "android")]
pub use sihora_platform::{set_android_app, AndroidApp};
