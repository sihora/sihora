//! Sihora — declarative cross-platform UI framework in pure Rust.
//!
//! # Features
//!
//! - **`full`** *(default)* — enables everything below.
//! - **`render`** — wgpu GPU renderer.
//! - **`text`** — text shaping and rasterization (cosmic-text).
//! - **`widgets`** — styled components (buttons, text fields, checkboxes, …).
//! - **`primitives`** — headless layout views (VStack, HStack, ZStack, …).
//! - **`platform`** — platform shells (desktop, web, mobile).
//! - **`data`** — reactive data layer with SQLite persistence.
//! - **`android`** — Android native activity support.

pub mod core;
pub mod layout;

#[cfg(feature = "primitives")]
pub mod primitives;

#[cfg(feature = "render")]
pub mod render;

#[cfg(feature = "text")]
pub mod text;

pub mod theme;

#[cfg(feature = "widgets")]
pub mod widgets;

#[cfg(feature = "platform")]
pub mod platform;

#[cfg(feature = "data")]
pub mod data;

// ── Core re-exports (always available) ──────────────────────────────────

pub use core::a11y;
pub use core::animation;
pub use core::focus;
pub use core::reactive;
pub use core::text_input;
pub use core::{
    Color, Event, Framed, Key, LayoutRect, Modifiers, NodeId, Padded, View, ViewExt, ViewTree,
};

pub use layout::{Alignment, Constraints, EdgeInsets, HAlign, Point, Size, VAlign};

pub use theme::{ColorScheme, Theme};

// ── Feature-gated re-exports ────────────────────────────────────────────

#[cfg(feature = "primitives")]
pub use primitives::{hit_test, HStack, SizedBox, VStack, ZStack};

#[cfg(feature = "render")]
pub use render::{DrawCommand, TextDraw};

#[cfg(all(feature = "text", not(target_arch = "wasm32")))]
pub use text::TextBuffer;

#[cfg(feature = "platform")]
pub use platform::{run, run_app, run_reactive, run_with_text, AppFrame, TextContext};

#[cfg(all(feature = "platform", target_os = "android"))]
pub use platform::{set_android_app, AndroidApp};
