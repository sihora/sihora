//! Card widget — flat or raised container.

use crate::core::Color;
use crate::render::DrawCommand;
use crate::theme::Theme;

/// Card elevation variant.
#[derive(Debug, Clone, Copy, Default)]
pub enum Variant {
    #[default]
    Flat,
    Raised,
}

/// Properties for the card widget.
pub struct Props {
    pub variant: Variant,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Draw a card container.
pub fn draw(props: &Props, theme: &Theme) -> Vec<DrawCommand> {
    let c = &theme.colors;
    let radius = theme.radius.lg;

    let (bg, border_w, border_c) = match props.variant {
        Variant::Flat => (c.surface, 1.0, c.border),
        Variant::Raised => (c.surface_raised, 0.0, Color::TRANSPARENT),
    };

    let mut cmds = vec![DrawCommand::Rect {
        position: [props.x, props.y],
        size: [props.width, props.height],
        color: bg,
        corner_radius: radius,
        border_width: border_w,
        border_color: border_c,
    }];

    if matches!(props.variant, Variant::Raised) {
        cmds.insert(
            0,
            DrawCommand::Rect {
                position: [props.x + 1.0, props.y + 2.0],
                size: [props.width, props.height],
                color: Color::BLACK.with_alpha(0.08),
                corner_radius: radius,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
        );
    }

    cmds
}
