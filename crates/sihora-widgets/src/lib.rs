//! Styled widget draw functions for Sihora.
//!
//! Each function consumes a [`Theme`] reference and produces
//! [`Vec<DrawCommand>`] using semantic tokens only — no raw colors.
//!
//! # Example
//! ```ignore
//! use sihora_theme::Theme;
//! use sihora_widgets::button;
//!
//! let theme = Theme::light();
//! let cmds = button::draw(
//!     &button::Props { label: "OK", variant: button::Variant::Primary, ..Default::default() },
//!     &theme,
//!     &mut text_ctx,
//! );
//! ```

pub mod button;
pub mod card;
pub mod checkbox;
pub mod dialog;
pub mod divider;
pub mod icon;
pub mod list_row;
pub mod text;
pub mod text_field;
pub mod toggle;

pub use sihora_render::DrawCommand;
pub use sihora_theme::Theme;
