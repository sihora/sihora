//! Divider widget — horizontal or vertical thin line.

use crate::core::Color;
use crate::render::DrawCommand;
use crate::theme::Theme;

/// Divider orientation.
#[derive(Debug, Clone, Copy, Default)]
pub enum Direction {
    #[default]
    Horizontal,
    Vertical,
}

/// Properties for the divider widget.
pub struct Props {
    pub direction: Direction,
    pub x: f32,
    pub y: f32,
    pub length: f32,
}

/// Draw a divider line.
pub fn draw(props: &Props, theme: &Theme) -> Vec<DrawCommand> {
    let (w, h) = match props.direction {
        Direction::Horizontal => (props.length, 1.0),
        Direction::Vertical => (1.0, props.length),
    };

    vec![DrawCommand::Rect {
        position: [props.x, props.y],
        size: [w, h],
        color: theme.colors.border,
        corner_radius: 0.0,
        border_width: 0.0,
        border_color: Color::TRANSPARENT,
    }]
}
