use crate::core::Color;

/// A draw command in the display list.
pub enum DrawCommand {
    /// Filled rectangle with optional rounded corners and border.
    Rect {
        position: [f32; 2],
        size: [f32; 2],
        color: Color,
        corner_radius: f32,
        border_width: f32,
        border_color: Color,
    },
    /// Batch of text glyphs to render.
    Text(TextBatch),
}

/// A batch of glyphs with atlas uploads and positioned instances.
pub struct TextBatch {
    /// New glyphs to upload to the atlas.
    pub uploads: Vec<GlyphUpload>,
    /// Positioned glyphs referencing atlas entries.
    pub glyphs: Vec<TextGlyph>,
}

/// RGBA bitmap data for a single glyph to upload to the atlas.
pub struct GlyphUpload {
    /// Stable hash identifying this glyph in the atlas.
    pub cache_key: u64,
    /// Bitmap width (pixels).
    pub width: u32,
    /// Bitmap height (pixels).
    pub height: u32,
    /// RGBA pixel data.
    pub data: Vec<u8>,
}

/// A positioned glyph referencing an atlas entry.
pub struct TextGlyph {
    /// Screen X position (logical pixels).
    pub x: f32,
    /// Screen Y position (logical pixels).
    pub y: f32,
    /// Width of the glyph bitmap (pixels).
    pub width: u32,
    /// Height of the glyph bitmap (pixels).
    pub height: u32,
    /// Atlas cache key matching a prior upload.
    pub cache_key: u64,
    /// Text color.
    pub color: Color,
}
