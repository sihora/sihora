use std::collections::HashMap;

const ATLAS_SIZE: u32 = 1024;

pub(crate) struct GlyphAtlas {
    pub size: u32,
    pub data: Vec<u8>,
    cache: HashMap<u64, AtlasRegion>,
    cursor_x: u32,
    cursor_y: u32,
    row_height: u32,
    pub dirty: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct AtlasRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl GlyphAtlas {
    pub fn new() -> Self {
        Self {
            size: ATLAS_SIZE,
            data: vec![0; (ATLAS_SIZE * ATLAS_SIZE * 4) as usize],
            cache: HashMap::new(),
            cursor_x: 0,
            cursor_y: 0,
            row_height: 0,
            dirty: false,
        }
    }

    pub fn get(&self, cache_key: u64) -> Option<AtlasRegion> {
        self.cache.get(&cache_key).copied()
    }

    pub fn insert(
        &mut self,
        cache_key: u64,
        width: u32,
        height: u32,
        rgba_data: &[u8],
    ) -> Option<AtlasRegion> {
        if let Some(&region) = self.cache.get(&cache_key) {
            return Some(region);
        }

        if width == 0 || height == 0 {
            return None;
        }

        if self.cursor_x + width > self.size {
            self.cursor_y += self.row_height + 1;
            self.cursor_x = 0;
            self.row_height = 0;
        }

        if self.cursor_y + height > self.size {
            return None;
        }

        let region = AtlasRegion {
            x: self.cursor_x,
            y: self.cursor_y,
            width,
            height,
        };

        for row in 0..height {
            let src_start = (row * width * 4) as usize;
            let src_end = src_start + (width * 4) as usize;
            let dst_start = ((self.cursor_y + row) * self.size * 4 + self.cursor_x * 4) as usize;
            let dst_end = dst_start + (width * 4) as usize;
            if src_end <= rgba_data.len() {
                self.data[dst_start..dst_end].copy_from_slice(&rgba_data[src_start..src_end]);
            }
        }

        self.cache.insert(cache_key, region);
        self.cursor_x += width + 1;
        self.row_height = self.row_height.max(height);
        self.dirty = true;

        Some(region)
    }
}
