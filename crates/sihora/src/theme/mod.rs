//! Design tokens and theming for Sihora.
//!
//! Provides semantic color tokens, typography scale, spacing, radius,
//! and motion presets. Light and dark themes are built in.
//!
//! # Example
//! ```
//! let theme = sihora::theme::Theme::light();
//! let bg = theme.colors.bg;
//! ```

use crate::core::Color;

/// Complete theme: colors, typography, spacing, radius, motion.
pub struct Theme {
    /// Semantic color tokens.
    pub colors: ColorTokens,
    /// Typography size scale.
    pub typography: TypographyScale,
    /// Spacing values on a 4px grid.
    pub spacing: SpacingScale,
    /// Corner radius presets.
    pub radius: RadiusScale,
    /// Motion duration presets.
    pub motion: MotionPresets,
}

/// Semantic color tokens. Widgets reference only these — never raw hex.
pub struct ColorTokens {
    pub bg: Color,
    pub bg_subtle: Color,
    pub surface: Color,
    pub surface_raised: Color,
    pub surface_overlay: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_disabled: Color,
    pub text_on_accent: Color,
    pub border: Color,
    pub border_strong: Color,
    pub focus_ring: Color,
    pub accent: Color,
    pub accent_hover: Color,
    pub accent_pressed: Color,
    pub success: Color,
    pub success_subtle: Color,
    pub warning: Color,
    pub warning_subtle: Color,
    pub danger: Color,
    pub danger_subtle: Color,
}

/// Font size scale (logical pixels).
pub struct TypographyScale {
    pub caption: f32,
    pub body_sm: f32,
    pub body: f32,
    pub label: f32,
    pub title_sm: f32,
    pub title: f32,
    pub headline: f32,
    pub display: f32,
}

/// Spacing on a 4px grid (logical pixels).
pub struct SpacingScale {
    pub xxs: f32,
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub xxl: f32,
}

/// Corner radii (logical pixels).
pub struct RadiusScale {
    pub none: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub full: f32,
}

/// Motion duration presets (seconds).
pub struct MotionPresets {
    pub fast: f32,
    pub normal: f32,
    pub slow: f32,
}

/// Color scheme selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorScheme {
    Light,
    Dark,
}

impl Theme {
    /// Create a theme for the given color scheme.
    pub fn new(scheme: ColorScheme) -> Self {
        match scheme {
            ColorScheme::Light => Self::light(),
            ColorScheme::Dark => Self::dark(),
        }
    }

    /// Light theme.
    pub fn light() -> Self {
        Self {
            colors: ColorTokens {
                bg: Color::rgb(0.976, 0.976, 0.976),
                bg_subtle: Color::rgb(0.953, 0.953, 0.953),
                surface: Color::rgb(1.0, 1.0, 1.0),
                surface_raised: Color::rgb(1.0, 1.0, 1.0),
                surface_overlay: Color::rgba(0.0, 0.0, 0.0, 0.4),
                text_primary: Color::rgb(0.09, 0.09, 0.09),
                text_secondary: Color::rgb(0.40, 0.40, 0.40),
                text_disabled: Color::rgb(0.65, 0.65, 0.65),
                text_on_accent: Color::rgb(1.0, 1.0, 1.0),
                border: Color::rgb(0.85, 0.85, 0.85),
                border_strong: Color::rgb(0.60, 0.60, 0.60),
                focus_ring: Color::rgba(0.15, 0.39, 0.85, 0.7),
                accent: Color::rgb(0.15, 0.39, 0.85),
                accent_hover: Color::rgb(0.11, 0.31, 0.75),
                accent_pressed: Color::rgb(0.09, 0.25, 0.65),
                success: Color::rgb(0.09, 0.48, 0.24),
                success_subtle: Color::rgb(0.89, 0.96, 0.90),
                warning: Color::rgb(0.55, 0.40, 0.02),
                warning_subtle: Color::rgb(0.99, 0.96, 0.89),
                danger: Color::rgb(0.75, 0.15, 0.15),
                danger_subtle: Color::rgb(0.99, 0.91, 0.91),
            },
            typography: TypographyScale::default(),
            spacing: SpacingScale::default(),
            radius: RadiusScale::default(),
            motion: MotionPresets::default(),
        }
    }

    /// Dark theme.
    pub fn dark() -> Self {
        Self {
            colors: ColorTokens {
                bg: Color::rgb(0.06, 0.06, 0.07),
                bg_subtle: Color::rgb(0.09, 0.09, 0.10),
                surface: Color::rgb(0.12, 0.12, 0.13),
                surface_raised: Color::rgb(0.17, 0.17, 0.18),
                surface_overlay: Color::rgba(0.0, 0.0, 0.0, 0.6),
                text_primary: Color::rgb(0.95, 0.95, 0.95),
                text_secondary: Color::rgb(0.64, 0.64, 0.64),
                text_disabled: Color::rgb(0.38, 0.38, 0.38),
                text_on_accent: Color::rgb(1.0, 1.0, 1.0),
                border: Color::rgb(0.22, 0.22, 0.24),
                border_strong: Color::rgb(0.40, 0.40, 0.42),
                focus_ring: Color::rgba(0.20, 0.42, 0.90, 0.7),
                accent: Color::rgb(0.20, 0.42, 0.85),
                accent_hover: Color::rgb(0.16, 0.36, 0.78),
                accent_pressed: Color::rgb(0.14, 0.30, 0.68),
                success: Color::rgb(0.16, 0.65, 0.35),
                success_subtle: Color::rgb(0.08, 0.15, 0.10),
                warning: Color::rgb(0.85, 0.65, 0.08),
                warning_subtle: Color::rgb(0.16, 0.13, 0.06),
                danger: Color::rgb(0.88, 0.28, 0.28),
                danger_subtle: Color::rgb(0.18, 0.08, 0.08),
            },
            typography: TypographyScale::default(),
            spacing: SpacingScale::default(),
            radius: RadiusScale::default(),
            motion: MotionPresets::default(),
        }
    }
}

impl Default for TypographyScale {
    fn default() -> Self {
        Self {
            caption: 12.0,
            body_sm: 13.0,
            body: 15.0,
            label: 14.0,
            title_sm: 17.0,
            title: 20.0,
            headline: 24.0,
            display: 32.0,
        }
    }
}

impl Default for SpacingScale {
    fn default() -> Self {
        Self {
            xxs: 2.0,
            xs: 4.0,
            sm: 8.0,
            md: 16.0,
            lg: 24.0,
            xl: 32.0,
            xxl: 48.0,
        }
    }
}

impl Default for RadiusScale {
    fn default() -> Self {
        Self {
            none: 0.0,
            sm: 4.0,
            md: 8.0,
            lg: 12.0,
            full: 9999.0,
        }
    }
}

impl Default for MotionPresets {
    fn default() -> Self {
        Self {
            fast: 0.12,
            normal: 0.20,
            slow: 0.32,
        }
    }
}

/// Compute WCAG 2.1 relative luminance of an opaque color.
pub fn relative_luminance(c: Color) -> f64 {
    fn linearize(v: f64) -> f64 {
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    }
    0.2126 * linearize(c.r as f64) + 0.7152 * linearize(c.g as f64) + 0.0722 * linearize(c.b as f64)
}

/// WCAG 2.1 contrast ratio between two opaque colors (always >= 1.0).
pub fn contrast_ratio(a: Color, b: Color) -> f64 {
    let la = relative_luminance(a);
    let lb = relative_luminance(b);
    let (lighter, darker) = if la > lb { (la, lb) } else { (lb, la) };
    (lighter + 0.05) / (darker + 0.05)
}

#[cfg(test)]
mod tests {
    use super::*;

    const AA_NORMAL: f64 = 4.5;
    const AA_LARGE: f64 = 3.0;

    fn check(fg: Color, bg: Color, label: &str, threshold: f64) {
        let cr = contrast_ratio(fg, bg);
        assert!(
            cr >= threshold,
            "{label}: contrast {cr:.2}:1 < {threshold}:1 (fg={fg:?}, bg={bg:?})"
        );
    }

    #[test]
    fn light_theme_contrast() {
        let t = Theme::light();
        let c = &t.colors;

        check(c.text_primary, c.bg, "text_primary on bg", AA_NORMAL);
        check(
            c.text_primary,
            c.surface,
            "text_primary on surface",
            AA_NORMAL,
        );
        check(c.text_secondary, c.bg, "text_secondary on bg", AA_NORMAL);
        check(
            c.text_secondary,
            c.surface,
            "text_secondary on surface",
            AA_NORMAL,
        );
        check(
            c.text_on_accent,
            c.accent,
            "text_on_accent on accent",
            AA_NORMAL,
        );
        check(
            c.text_on_accent,
            c.accent_hover,
            "text_on_accent on accent_hover",
            AA_NORMAL,
        );
        check(
            c.text_on_accent,
            c.accent_pressed,
            "text_on_accent on accent_pressed",
            AA_NORMAL,
        );
        check(c.danger, c.bg, "danger text on bg", AA_LARGE);
        check(c.success, c.bg, "success text on bg", AA_LARGE);
    }

    #[test]
    fn dark_theme_contrast() {
        let t = Theme::dark();
        let c = &t.colors;

        check(c.text_primary, c.bg, "text_primary on bg", AA_NORMAL);
        check(
            c.text_primary,
            c.surface,
            "text_primary on surface",
            AA_NORMAL,
        );
        check(c.text_secondary, c.bg, "text_secondary on bg", AA_NORMAL);
        check(
            c.text_secondary,
            c.surface,
            "text_secondary on surface",
            AA_NORMAL,
        );
        check(
            c.text_on_accent,
            c.accent,
            "text_on_accent on accent",
            AA_NORMAL,
        );
        check(
            c.text_on_accent,
            c.accent_hover,
            "text_on_accent on accent_hover",
            AA_NORMAL,
        );
        check(
            c.text_on_accent,
            c.accent_pressed,
            "text_on_accent on accent_pressed",
            AA_NORMAL,
        );
        check(c.danger, c.bg, "danger text on bg", AA_LARGE);
        check(c.success, c.bg, "success text on bg", AA_LARGE);
    }

    #[test]
    fn black_on_white_is_21() {
        let cr = contrast_ratio(Color::BLACK, Color::WHITE);
        assert!((cr - 21.0).abs() < 0.1);
    }

    #[test]
    fn same_color_is_1() {
        let cr = contrast_ratio(Color::rgb(0.5, 0.5, 0.5), Color::rgb(0.5, 0.5, 0.5));
        assert!((cr - 1.0).abs() < 0.01);
    }
}
