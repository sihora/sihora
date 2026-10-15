/// RGBA color with components in `[0.0, 1.0]`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const WHITE: Self = Self::rgb(1.0, 1.0, 1.0);
    pub const BLACK: Self = Self::rgb(0.0, 0.0, 0.0);
    pub const TRANSPARENT: Self = Self::rgba(0.0, 0.0, 0.0, 0.0);

    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub const fn to_array(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Return a copy with the alpha channel multiplied by `opacity`.
    ///
    /// # Example
    /// ```
    /// use sihora::core::Color;
    /// let c = Color::WHITE.with_alpha(0.5);
    /// assert!((c.a - 0.5).abs() < f32::EPSILON);
    /// ```
    pub fn with_alpha(self, opacity: f32) -> Self {
        Self {
            a: self.a * opacity,
            ..self
        }
    }
}
