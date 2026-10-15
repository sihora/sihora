//! Button widget — primary, secondary, ghost, destructive.

use crate::core::Color;
use crate::render::{DrawCommand, TextDraw};
use crate::theme::Theme;

/// Visual variant.
#[derive(Debug, Clone, Copy, Default)]
pub enum Variant {
    #[default]
    Primary,
    Secondary,
    Ghost,
    Destructive,
}

/// Interactive state.
#[derive(Debug, Clone, Copy, Default)]
pub enum State {
    #[default]
    Normal,
    Hover,
    Pressed,
    Focused,
    Disabled,
}

/// Button size preset.
#[derive(Debug, Clone, Copy, Default)]
pub enum Size {
    Sm,
    #[default]
    Md,
    Lg,
}

impl Size {
    fn height(self) -> f32 {
        match self {
            Self::Sm => 28.0,
            Self::Md => 36.0,
            Self::Lg => 44.0,
        }
    }

    fn font_size(self, theme: &Theme) -> f32 {
        match self {
            Self::Sm => theme.typography.body_sm,
            Self::Md => theme.typography.label,
            Self::Lg => theme.typography.body,
        }
    }

    fn padding_h(self, theme: &Theme) -> f32 {
        match self {
            Self::Sm => theme.spacing.sm,
            Self::Md => theme.spacing.md,
            Self::Lg => theme.spacing.lg,
        }
    }
}

/// Properties for the button widget.
pub struct Props<'a> {
    pub label: &'a str,
    pub variant: Variant,
    pub state: State,
    pub size: Size,
    pub x: f32,
    pub y: f32,
    pub width: f32,
}

impl<'a> Default for Props<'a> {
    fn default() -> Self {
        Self {
            label: "",
            variant: Variant::Primary,
            state: State::Normal,
            size: Size::Md,
            x: 0.0,
            y: 0.0,
            width: 120.0,
        }
    }
}

/// Draw a button.
pub fn draw(props: &Props<'_>, theme: &Theme, td: &mut dyn TextDraw) -> Vec<DrawCommand> {
    let h = props.size.height();
    let radius = theme.radius.md;

    let (bg, text_color, border_w, border_c) = resolve_style(props, theme);

    let mut cmds = Vec::with_capacity(3);

    cmds.push(DrawCommand::Rect {
        position: [props.x, props.y],
        size: [props.width, h],
        color: bg,
        corner_radius: radius,
        border_width: border_w,
        border_color: border_c,
    });

    if matches!(props.state, State::Focused) {
        cmds.push(DrawCommand::Rect {
            position: [props.x - 2.0, props.y - 2.0],
            size: [props.width + 4.0, h + 4.0],
            color: Color::TRANSPARENT,
            corner_radius: radius + 2.0,
            border_width: 2.0,
            border_color: theme.colors.focus_ring,
        });
    }

    let font_size = props.size.font_size(theme);
    let pad = props.size.padding_h(theme);
    let text_y = props.y + (h - font_size) / 2.0;
    cmds.push(td.draw_label(props.label, font_size, props.x + pad, text_y, text_color));

    cmds
}

fn resolve_style(props: &Props<'_>, theme: &Theme) -> (Color, Color, f32, Color) {
    let c = &theme.colors;

    if matches!(props.state, State::Disabled) {
        return (c.bg_subtle, c.text_disabled, 0.0, Color::TRANSPARENT);
    }

    match props.variant {
        Variant::Primary => {
            let bg = match props.state {
                State::Hover => c.accent_hover,
                State::Pressed => c.accent_pressed,
                _ => c.accent,
            };
            (bg, c.text_on_accent, 0.0, Color::TRANSPARENT)
        }
        Variant::Secondary => {
            let bg = match props.state {
                State::Hover | State::Pressed => c.bg_subtle,
                _ => c.surface,
            };
            (bg, c.text_primary, 1.0, c.border)
        }
        Variant::Ghost => {
            let bg = match props.state {
                State::Hover | State::Pressed => c.bg_subtle,
                _ => Color::TRANSPARENT,
            };
            (bg, c.text_primary, 0.0, Color::TRANSPARENT)
        }
        Variant::Destructive => {
            let bg = match props.state {
                State::Hover | State::Pressed => c.danger_subtle,
                _ => c.danger,
            };
            let text = match props.state {
                State::Hover | State::Pressed => c.danger,
                _ => c.text_on_accent,
            };
            (bg, text, 0.0, Color::TRANSPARENT)
        }
    }
}
