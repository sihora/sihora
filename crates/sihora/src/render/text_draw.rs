//! Text rendering trait for widget composition.

use super::DrawCommand;
use crate::core::Color;

/// Renders text into a [`DrawCommand`].
///
/// Widgets accept `&mut dyn TextDraw` to produce text commands without
/// depending on the platform's text shaping implementation.
pub trait TextDraw {
    /// Render `text` at the given position and return the draw command.
    fn draw_label(
        &mut self,
        text: &str,
        font_size: f32,
        x: f32,
        y: f32,
        color: Color,
    ) -> DrawCommand;
}
