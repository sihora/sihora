//! Text widget — renders a label using the typography scale.

use crate::core::Color;
use crate::render::{DrawCommand, TextDraw};
use crate::theme::Theme;

/// Typography role selects from the theme's type scale.
#[derive(Debug, Clone, Copy, Default)]
pub enum Role {
    Caption,
    BodySm,
    #[default]
    Body,
    Label,
    TitleSm,
    Title,
    Headline,
    Display,
}

/// Text color semantic slot.
#[derive(Debug, Clone, Copy, Default)]
pub enum TextColor {
    #[default]
    Primary,
    Secondary,
    Disabled,
    Danger,
    Success,
    Custom(Color),
}

/// Properties for the text widget.
pub struct Props<'a> {
    pub text: &'a str,
    pub role: Role,
    pub color: TextColor,
    pub x: f32,
    pub y: f32,
}

/// Draw a text label.
pub fn draw(props: &Props<'_>, theme: &Theme, td: &mut dyn TextDraw) -> Vec<DrawCommand> {
    let size = match props.role {
        Role::Caption => theme.typography.caption,
        Role::BodySm => theme.typography.body_sm,
        Role::Body => theme.typography.body,
        Role::Label => theme.typography.label,
        Role::TitleSm => theme.typography.title_sm,
        Role::Title => theme.typography.title,
        Role::Headline => theme.typography.headline,
        Role::Display => theme.typography.display,
    };

    let color = match props.color {
        TextColor::Primary => theme.colors.text_primary,
        TextColor::Secondary => theme.colors.text_secondary,
        TextColor::Disabled => theme.colors.text_disabled,
        TextColor::Danger => theme.colors.danger,
        TextColor::Success => theme.colors.success,
        TextColor::Custom(c) => c,
    };

    vec![td.draw_label(props.text, size, props.x, props.y, color)]
}
