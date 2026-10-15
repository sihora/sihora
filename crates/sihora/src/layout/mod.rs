//! Constraint-based layout types for Sihora.

/// 2-D size in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub const ZERO: Self = Self {
        width: 0.0,
        height: 0.0,
    };

    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

/// 2-D point in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// Min/max constraints passed down the tree during layout.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Constraints {
    pub min_width: f32,
    pub max_width: f32,
    pub min_height: f32,
    pub max_height: f32,
}

impl Constraints {
    /// Both min and max set to the given dimensions.
    pub const fn exact(width: f32, height: f32) -> Self {
        Self {
            min_width: width,
            max_width: width,
            min_height: height,
            max_height: height,
        }
    }

    /// Min = 0, max = infinity on both axes.
    pub const fn unbounded() -> Self {
        Self {
            min_width: 0.0,
            max_width: f32::INFINITY,
            min_height: 0.0,
            max_height: f32::INFINITY,
        }
    }

    /// Clamp a size to fit within these constraints.
    pub fn constrain(&self, size: Size) -> Size {
        Size {
            width: size.width.clamp(self.min_width, self.max_width),
            height: size.height.clamp(self.min_height, self.max_height),
        }
    }

    /// Shrink constraints by the given insets.
    pub fn shrink(&self, insets: EdgeInsets) -> Self {
        let h = insets.horizontal();
        let v = insets.vertical();
        Self {
            min_width: (self.min_width - h).max(0.0),
            max_width: (self.max_width - h).max(0.0),
            min_height: (self.min_height - v).max(0.0),
            max_height: (self.max_height - v).max(0.0),
        }
    }
}

/// Padding amounts for each edge.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct EdgeInsets {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl EdgeInsets {
    pub const ZERO: Self = Self {
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
        left: 0.0,
    };

    pub const fn all(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub const fn symmetric(horizontal: f32, vertical: f32) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    pub const fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }
}

/// Horizontal alignment within a container.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HAlign {
    Leading,
    #[default]
    Center,
    Trailing,
}

impl HAlign {
    /// Fraction along the horizontal axis: 0.0, 0.5, or 1.0.
    pub fn factor(self) -> f32 {
        match self {
            Self::Leading => 0.0,
            Self::Center => 0.5,
            Self::Trailing => 1.0,
        }
    }
}

/// Vertical alignment within a container.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VAlign {
    Top,
    #[default]
    Center,
    Bottom,
}

impl VAlign {
    /// Fraction along the vertical axis: 0.0, 0.5, or 1.0.
    pub fn factor(self) -> f32 {
        match self {
            Self::Top => 0.0,
            Self::Center => 0.5,
            Self::Bottom => 1.0,
        }
    }
}

/// Combined horizontal + vertical alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Alignment {
    pub h: HAlign,
    pub v: VAlign,
}

impl Alignment {
    pub const TOP_LEADING: Self = Self {
        h: HAlign::Leading,
        v: VAlign::Top,
    };
    pub const TOP: Self = Self {
        h: HAlign::Center,
        v: VAlign::Top,
    };
    pub const TOP_TRAILING: Self = Self {
        h: HAlign::Trailing,
        v: VAlign::Top,
    };
    pub const LEADING: Self = Self {
        h: HAlign::Leading,
        v: VAlign::Center,
    };
    pub const CENTER: Self = Self {
        h: HAlign::Center,
        v: VAlign::Center,
    };
    pub const TRAILING: Self = Self {
        h: HAlign::Trailing,
        v: VAlign::Center,
    };
    pub const BOTTOM_LEADING: Self = Self {
        h: HAlign::Leading,
        v: VAlign::Bottom,
    };
    pub const BOTTOM: Self = Self {
        h: HAlign::Center,
        v: VAlign::Bottom,
    };
    pub const BOTTOM_TRAILING: Self = Self {
        h: HAlign::Trailing,
        v: VAlign::Bottom,
    };
}
