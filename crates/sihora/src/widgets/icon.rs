//! Icon widget — placeholder square using theme tokens.
//!
//! Real icon rendering (SVG / icon font) is post-v0.1. For now, a
//! tinted square of the right size shows icon placement in layouts.

use crate::render::DrawCommand;
use crate::theme::Theme;

/// Icon sizes in logical pixels.
#[derive(Debug, Clone, Copy, Default)]
pub enum Size {
    Sm,
    #[default]
    Md,
    Lg,
}

impl Size {
    fn pixels(self) -> f32 {
        match self {
            Self::Sm => 16.0,
            Self::Md => 20.0,
            Self::Lg => 24.0,
        }
    }
}

/// Properties for the icon widget.
pub struct Props {
    pub size: Size,
    pub x: f32,
    pub y: f32,
}

/// Draw an icon placeholder.
pub fn draw(props: &Props, theme: &Theme) -> Vec<DrawCommand> {
    let s = props.size.pixels();
    vec![DrawCommand::Rect {
        position: [props.x, props.y],
        size: [s, s],
        color: theme.colors.text_secondary,
        corner_radius: theme.radius.sm,
        border_width: 0.0,
        border_color: crate::core::Color::TRANSPARENT,
    }]
}
