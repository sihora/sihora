//! Text shaping, layout, and rasterization for Sihora.

use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping, SwashCache, SwashContent};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Manages fonts and glyph rasterization.
pub struct TextSystem {
    font_system: FontSystem,
    swash_cache: SwashCache,
}

impl TextSystem {
    /// Create a new text system, loading system fonts.
    pub fn new() -> Self {
        Self {
            font_system: FontSystem::new(),
            swash_cache: SwashCache::new(),
        }
    }

    /// Create a shaped text buffer.
    pub fn create_buffer(
        &mut self,
        text: &str,
        font_size: f32,
        line_height: f32,
        max_width: Option<f32>,
    ) -> TextBuffer {
        let metrics = Metrics::new(font_size, line_height);
        let mut buffer = Buffer::new(&mut self.font_system, metrics);
        buffer.set_size(&mut self.font_system, max_width, None);
        buffer.set_text(
            &mut self.font_system,
            text,
            Attrs::new().family(Family::SansSerif),
            Shaping::Advanced,
        );
        buffer.shape_until_scroll(&mut self.font_system, false);
        TextBuffer { inner: buffer }
    }

    /// Resize a text buffer's wrapping width.
    pub fn resize_buffer(&mut self, buffer: &mut TextBuffer, max_width: Option<f32>) {
        buffer
            .inner
            .set_size(&mut self.font_system, max_width, None);
        buffer
            .inner
            .shape_until_scroll(&mut self.font_system, false);
    }

    /// Rasterize all visible glyphs in the buffer.
    pub fn shaped_glyphs(&mut self, buffer: &TextBuffer) -> Vec<ShapedGlyph> {
        let mut glyphs = Vec::new();
        for run in buffer.inner.layout_runs() {
            for glyph in run.glyphs.iter() {
                let physical = glyph.physical((0., run.line_y), 1.0);
                let key = hash_cache_key(&physical.cache_key);

                let image = self
                    .swash_cache
                    .get_image_uncached(&mut self.font_system, physical.cache_key);
                let Some(image) = image else {
                    continue;
                };

                let w = image.placement.width;
                let h = image.placement.height;
                if w == 0 || h == 0 {
                    continue;
                }

                let rgba = match image.content {
                    SwashContent::Mask => image
                        .data
                        .iter()
                        .flat_map(|&a| [255, 255, 255, a])
                        .collect(),
                    SwashContent::SubpixelMask => image
                        .data
                        .chunks(3)
                        .flat_map(|rgb| {
                            let avg = ((u16::from(rgb[0]) + u16::from(rgb[1]) + u16::from(rgb[2]))
                                / 3) as u8;
                            [255, 255, 255, avg]
                        })
                        .collect(),
                    SwashContent::Color => image.data,
                };

                glyphs.push(ShapedGlyph {
                    x: physical.x as f32 + image.placement.left as f32,
                    y: physical.y as f32 - image.placement.top as f32,
                    width: w,
                    height: h,
                    cache_key: key,
                    data: rgba,
                });
            }
        }
        glyphs
    }
}

impl Default for TextSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// A shaped text buffer with layout information.
pub struct TextBuffer {
    inner: Buffer,
}

impl TextBuffer {
    /// Measure the laid-out text bounds.
    pub fn measure(&self) -> (f32, f32) {
        let mut width: f32 = 0.0;
        let mut max_bottom: f32 = 0.0;
        for run in self.inner.layout_runs() {
            width = width.max(run.line_w);
            max_bottom = max_bottom.max(run.line_y + run.line_height);
        }
        (width, max_bottom)
    }
}

/// A rasterized glyph ready for atlas upload.
pub struct ShapedGlyph {
    /// X offset from the buffer origin (pixels).
    pub x: f32,
    /// Y offset from the buffer origin (pixels).
    pub y: f32,
    /// Bitmap width (pixels).
    pub width: u32,
    /// Bitmap height (pixels).
    pub height: u32,
    /// Stable hash for atlas caching.
    pub cache_key: u64,
    /// RGBA bitmap data.
    pub data: Vec<u8>,
}

fn hash_cache_key(key: &cosmic_text::CacheKey) -> u64 {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    hasher.finish()
}
