//! Toggle (switch) widget — on/off control.

use crate::core::Color;
use crate::render::DrawCommand;
use crate::theme::Theme;

/// Toggle interactive state.
#[derive(Debug, Clone, Copy, Default)]
pub enum State {
    #[default]
    Normal,
    Focused,
    Disabled,
}

/// Properties for the toggle widget.
pub struct Props {
    pub on: bool,
    pub state: State,
    pub x: f32,
    pub y: f32,
}

const WIDTH: f32 = 44.0;
const HEIGHT: f32 = 24.0;
const KNOB: f32 = 18.0;
const INSET: f32 = 3.0;

/// Draw a toggle switch.
pub fn draw(props: &Props, theme: &Theme) -> Vec<DrawCommand> {
    let c = &theme.colors;

    let track_color = if matches!(props.state, State::Disabled) {
        c.bg_subtle
    } else if props.on {
        c.accent
    } else {
        c.border_strong
    };

    let mut cmds = Vec::with_capacity(3);

    cmds.push(DrawCommand::Rect {
        position: [props.x, props.y],
        size: [WIDTH, HEIGHT],
        color: track_color,
        corner_radius: HEIGHT / 2.0,
        border_width: 0.0,
        border_color: Color::TRANSPARENT,
    });

    if matches!(props.state, State::Focused) {
        cmds.push(DrawCommand::Rect {
            position: [props.x - 2.0, props.y - 2.0],
            size: [WIDTH + 4.0, HEIGHT + 4.0],
            color: Color::TRANSPARENT,
            corner_radius: HEIGHT / 2.0 + 2.0,
            border_width: 2.0,
            border_color: c.focus_ring,
        });
    }

    let knob_x = if props.on {
        props.x + WIDTH - KNOB - INSET
    } else {
        props.x + INSET
    };
    let knob_y = props.y + INSET;

    cmds.push(DrawCommand::Rect {
        position: [knob_x, knob_y],
        size: [KNOB, KNOB],
        color: Color::WHITE,
        corner_radius: KNOB / 2.0,
        border_width: 0.0,
        border_color: Color::TRANSPARENT,
    });

    cmds
}
