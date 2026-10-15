//! Checkbox widget — checked, unchecked, indeterminate.

use crate::core::Color;
use crate::render::DrawCommand;
use crate::theme::Theme;

/// Check mark visual state.
#[derive(Debug, Clone, Copy, Default)]
pub enum Value {
    #[default]
    Unchecked,
    Checked,
    Indeterminate,
}

/// Interactive state.
#[derive(Debug, Clone, Copy, Default)]
pub enum State {
    #[default]
    Normal,
    Focused,
    Disabled,
}

/// Properties for the checkbox widget.
pub struct Props {
    pub value: Value,
    pub state: State,
    pub x: f32,
    pub y: f32,
}

const SIZE: f32 = 20.0;

/// Draw a checkbox.
pub fn draw(props: &Props, theme: &Theme) -> Vec<DrawCommand> {
    let c = &theme.colors;
    let checked = !matches!(props.value, Value::Unchecked);

    let (bg, border_c, border_w) = if matches!(props.state, State::Disabled) {
        (c.bg_subtle, c.border, 1.0)
    } else if checked {
        (c.accent, c.accent, 0.0)
    } else {
        (c.surface, c.border_strong, 1.5)
    };

    let mut cmds = Vec::with_capacity(3);

    cmds.push(DrawCommand::Rect {
        position: [props.x, props.y],
        size: [SIZE, SIZE],
        color: bg,
        corner_radius: theme.radius.sm,
        border_width: border_w,
        border_color: border_c,
    });

    if matches!(props.state, State::Focused) {
        cmds.push(DrawCommand::Rect {
            position: [props.x - 2.0, props.y - 2.0],
            size: [SIZE + 4.0, SIZE + 4.0],
            color: Color::TRANSPARENT,
            corner_radius: theme.radius.sm + 2.0,
            border_width: 2.0,
            border_color: c.focus_ring,
        });
    }

    if checked {
        let mark_size = match props.value {
            Value::Indeterminate => [10.0, 2.0],
            _ => [8.0, 8.0],
        };
        let mx = props.x + (SIZE - mark_size[0]) / 2.0;
        let my = props.y + (SIZE - mark_size[1]) / 2.0;
        cmds.push(DrawCommand::Rect {
            position: [mx, my],
            size: mark_size,
            color: c.text_on_accent,
            corner_radius: 1.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        });
    }

    cmds
}
