use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

pub struct Texture {
    pub size: (u32, u32),
    pub bytes: Vec<u8>,
}

impl Texture {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let path_str = path.as_ref().to_str().unwrap();
        let bytes = std::fs::read(&path)
            .map_err(|e| anyhow!("Failed to load image at \"{}\": {:?}", path_str, e))?;

        let cursor = std::io::Cursor::new(bytes);
        let decoder = png::Decoder::new(cursor);
        let mut reader = decoder
            .read_info()
            .map_err(|e| anyhow!("Failed to read image info at \"{}\": {:?}", path_str, e))?;

        let mut buf = vec![0; reader.output_buffer_size()];
        let info = reader
            .next_frame(&mut buf)
            .map_err(|e| anyhow!("Failed to read image data at \"{}\": {:?}", path_str, e))?;
        let bytes: Vec<u8> = buf[..info.buffer_size()].into();

        Ok(Self {
            size: (info.width, info.height),
            bytes,
        })
    }
}
