//! Dialog widget — overlay scrim + centered card with title, body, actions.

use crate::core::Color;
use crate::render::{DrawCommand, TextDraw};
use crate::theme::Theme;

/// Properties for the dialog widget.
pub struct Props<'a> {
    pub title: &'a str,
    pub body: &'a str,
    pub confirm_label: &'a str,
    pub cancel_label: &'a str,
    pub viewport_width: f32,
    pub viewport_height: f32,
}

const DIALOG_W: f32 = 320.0;
const DIALOG_H: f32 = 180.0;

/// Draw a dialog with scrim overlay.
pub fn draw(props: &Props<'_>, theme: &Theme, td: &mut dyn TextDraw) -> Vec<DrawCommand> {
    let c = &theme.colors;
    let sp = &theme.spacing;

    let mut cmds = Vec::with_capacity(8);

    cmds.push(DrawCommand::Rect {
        position: [0.0, 0.0],
        size: [props.viewport_width, props.viewport_height],
        color: c.surface_overlay,
        corner_radius: 0.0,
        border_width: 0.0,
        border_color: Color::TRANSPARENT,
    });

    let dx = (props.viewport_width - DIALOG_W) / 2.0;
    let dy = (props.viewport_height - DIALOG_H) / 2.0;

    cmds.push(DrawCommand::Rect {
        position: [dx, dy],
        size: [DIALOG_W, DIALOG_H],
        color: c.surface_raised,
        corner_radius: theme.radius.lg,
        border_width: 0.0,
        border_color: Color::TRANSPARENT,
    });

    let pad = sp.lg;
    let title_size = theme.typography.title_sm;
    cmds.push(td.draw_label(props.title, title_size, dx + pad, dy + pad, c.text_primary));

    let body_y = dy + pad + title_size + sp.sm;
    cmds.push(td.draw_label(
        props.body,
        theme.typography.body,
        dx + pad,
        body_y,
        c.text_secondary,
    ));

    let btn_y = dy + DIALOG_H - 36.0 - sp.md;
    let btn_w = 80.0;

    if !props.cancel_label.is_empty() {
        let cancel_x = dx + DIALOG_W - pad - btn_w * 2.0 - sp.sm;
        cmds.push(DrawCommand::Rect {
            position: [cancel_x, btn_y],
            size: [btn_w, 36.0],
            color: Color::TRANSPARENT,
            corner_radius: theme.radius.md,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        });
        cmds.push(td.draw_label(
            props.cancel_label,
            theme.typography.label,
            cancel_x + sp.sm,
            btn_y + 10.0,
            c.text_secondary,
        ));
    }

    if !props.confirm_label.is_empty() {
        let confirm_x = dx + DIALOG_W - pad - btn_w;
        cmds.push(DrawCommand::Rect {
            position: [confirm_x, btn_y],
            size: [btn_w, 36.0],
            color: c.accent,
            corner_radius: theme.radius.md,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        });
        cmds.push(td.draw_label(
            props.confirm_label,
            theme.typography.label,
            confirm_x + sp.sm,
            btn_y + 10.0,
            c.text_on_accent,
        ));
    }

    cmds
}
