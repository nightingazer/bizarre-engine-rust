use std::path::Path;

use anyhow::Result;

use crate::texture::Texture;

pub struct CubeMap {
    pub side_width: u32,
    pub side_height: u32,
    pub texture_data: [Vec<u8>; 6],
}

impl CubeMap {
    pub fn new(path: String) -> Result<Self> {
        let path = Path::new(&path);

        let paths = [
            path.join("px.png"),
            path.join("nx.png"),
            path.join("py.png"),
            path.join("ny.png"),
            path.join("pz.png"),
            path.join("nz.png"),
        ];

        let mut dim = [0; 2];

        let texture_data_vec = paths
            .iter()
            .map(|p| {
                let texture = Texture::new(p)?;
                dim = [texture.size.0, texture.size.1];
                Ok(texture.bytes)
            })
            .collect::<Result<Vec<_>>>()?;

        let mut texture_data: [Vec<u8>; 6] = Default::default();

        for (i, data) in texture_data_vec.iter().enumerate() {
            texture_data[i] = data.clone();
        }

        let side_width = dim[0];
        let side_height = dim[1];

        Ok(Self {
            side_width,
            side_height,
            texture_data,
        })
    }
}

fn load_image(path: impl AsRef<Path>) -> (Vec<u8>, [u32; 2]) {
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(e) => panic!(
            "Failed to load image at path \"{}\": {:?}",
            path.as_ref().to_str().unwrap(),
            e
        ),
    };

    let cursor = std::io::Cursor::new(bytes);
    let decoder = png::Decoder::new(cursor);
    let mut reader = decoder.read_info().unwrap();

    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).unwrap();
    let bytes: Vec<u8> = buf[..info.buffer_size()].into();

    (bytes, [info.width, info.height])
}
