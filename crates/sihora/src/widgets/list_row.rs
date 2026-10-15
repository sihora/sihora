//! ListRow widget — a horizontal row with label and optional trailing text.

use crate::core::Color;
use crate::render::{DrawCommand, TextDraw};
use crate::theme::Theme;

/// Row interactive state.
#[derive(Debug, Clone, Copy, Default)]
pub enum State {
    #[default]
    Normal,
    Hover,
    Selected,
    Pressed,
}

/// Properties for the list row widget.
pub struct Props<'a> {
    pub label: &'a str,
    pub trailing: &'a str,
    pub state: State,
    pub x: f32,
    pub y: f32,
    pub width: f32,
}

impl<'a> Default for Props<'a> {
    fn default() -> Self {
        Self {
            label: "",
            trailing: "",
            state: State::Normal,
            x: 0.0,
            y: 0.0,
            width: 300.0,
        }
    }
}

const HEIGHT: f32 = 44.0;

/// Draw a list row.
pub fn draw(props: &Props<'_>, theme: &Theme, td: &mut dyn TextDraw) -> Vec<DrawCommand> {
    let c = &theme.colors;
    let pad = theme.spacing.md;
    let font = theme.typography.body;

    let bg = match props.state {
        State::Normal => Color::TRANSPARENT,
        State::Hover => c.bg_subtle,
        State::Selected => c.accent.with_alpha(0.12),
        State::Pressed => c.bg_subtle,
    };

    let mut cmds = Vec::with_capacity(3);

    cmds.push(DrawCommand::Rect {
        position: [props.x, props.y],
        size: [props.width, HEIGHT],
        color: bg,
        corner_radius: theme.radius.sm,
        border_width: 0.0,
        border_color: Color::TRANSPARENT,
    });

    let text_y = props.y + (HEIGHT - font) / 2.0;
    cmds.push(td.draw_label(props.label, font, props.x + pad, text_y, c.text_primary));

    if !props.trailing.is_empty() {
        let trail_x = props.x + props.width - pad - 60.0;
        cmds.push(td.draw_label(
            props.trailing,
            theme.typography.body_sm,
            trail_x,
            text_y,
            c.text_secondary,
        ));
    }

    cmds
}
