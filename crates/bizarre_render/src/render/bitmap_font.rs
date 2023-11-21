use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Result;
use bmfont::{BMFont, OrdinateOrientation};

use crate::texture::Texture;

pub struct BitmapFont {
    pub bitmap: Texture,
    pub font_config: BMFont,
}

impl BitmapFont {
    pub fn new(fnt_path: impl AsRef<Path>, png_path: impl AsRef<Path>) -> Result<Arc<Self>> {
        let fnt_file = std::fs::File::open(fnt_path)?;
        let font_config = BMFont::new(fnt_file, OrdinateOrientation::BottomToTop)?;
        let texture = Texture::new(png_path)?;

        let bitmap_font = Self {
            bitmap: texture,
            font_config,
        };

        Ok(Arc::new(bitmap_font))
    }
}
