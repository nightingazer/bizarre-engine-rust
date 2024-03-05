use std::path::Path;

use anyhow::{anyhow, Result};
use bizarre_common::handle::Handle;
use nalgebra_glm::Vec2;

use crate::vulkan::image::VulkanImage;

pub type TextureHandle = Handle<Texture>;

pub struct Texture {
    handle: TextureHandle,
    image: VulkanImage,
}

impl Texture {
    pub fn new(handle: TextureHandle, path: impl AsRef<Path>) -> Result<Self> {
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

        todo!()
    }
}
