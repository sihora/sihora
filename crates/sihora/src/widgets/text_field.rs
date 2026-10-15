//! TextField widget — bordered input area with states.

use crate::core::Color;
use crate::render::{DrawCommand, TextDraw};
use crate::theme::Theme;

/// Field visual state.
#[derive(Debug, Clone, Copy, Default)]
pub enum State {
    #[default]
    Normal,
    Focused,
    Error,
    Disabled,
}

/// Properties for the text field widget.
pub struct Props<'a> {
    pub value: &'a str,
    pub placeholder: &'a str,
    pub state: State,
    pub x: f32,
    pub y: f32,
    pub width: f32,
}

impl<'a> Default for Props<'a> {
    fn default() -> Self {
        Self {
            value: "",
            placeholder: "",
            state: State::Normal,
            x: 0.0,
            y: 0.0,
            width: 200.0,
        }
    }
}

const HEIGHT: f32 = 36.0;

/// Draw a text field.
pub fn draw(props: &Props<'_>, theme: &Theme, td: &mut dyn TextDraw) -> Vec<DrawCommand> {
    let c = &theme.colors;
    let radius = theme.radius.md;

    let (bg, border_c, border_w) = match props.state {
        State::Normal => (c.surface, c.border, 1.0),
        State::Focused => (c.surface, c.accent, 2.0),
        State::Error => (c.surface, c.danger, 2.0),
        State::Disabled => (c.bg_subtle, c.border, 1.0),
    };

    let mut cmds = Vec::with_capacity(3);

    cmds.push(DrawCommand::Rect {
        position: [props.x, props.y],
        size: [props.width, HEIGHT],
        color: bg,
        corner_radius: radius,
        border_width: border_w,
        border_color: border_c,
    });

    if matches!(props.state, State::Focused) {
        cmds.push(DrawCommand::Rect {
            position: [props.x - 2.0, props.y - 2.0],
            size: [props.width + 4.0, HEIGHT + 4.0],
            color: Color::TRANSPARENT,
            corner_radius: radius + 2.0,
            border_width: 2.0,
            border_color: c.focus_ring,
        });
    }

    let pad = theme.spacing.sm;
    let font = theme.typography.body;
    let text_y = props.y + (HEIGHT - font) / 2.0;

    let (display, color) = if props.value.is_empty() {
        (props.placeholder, c.text_disabled)
    } else {
        let tc = if matches!(props.state, State::Disabled) {
            c.text_disabled
        } else {
            c.text_primary
        };
        (props.value, tc)
    };

    if !display.is_empty() {
        cmds.push(td.draw_label(display, font, props.x + pad, text_y, color));
    }

    cmds
}
