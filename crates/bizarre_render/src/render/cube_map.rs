// use std::{fs::File, io::Read};

// use vulkano::image::ImageDimensions;

// pub struct CubeMapSidePaths {
//     pub front: String,
//     pub back: String,
//     pub top: String,
//     pub bottom: String,
//     pub right: String,
//     pub left: String,
// }

// pub struct CubeMap {
//     pub side_width: u32,
//     pub side_height: u32,
// }

// fn load_image(path: String) -> (Vec<u8>, ImageDimensions) {
//     let bytes = match std::fs::read(path) {
//         Ok(bytes) => bytes,
//         Err(e) => panic!("Failed to load image at path \"{}\": {:?}", path, e),
//     };

//     let cursor = std::io::Cursor::new(bytes);
//     let decoder = png::Decoder::new(cursor);
//     let mut reader = decoder.read_info().unwrap();
//     let info = reader.info();
//     let image_dimensions = ImageDimensions::Dim2d {
//         width: info.width,
//         height: info.height,
//         array_layers: 1,
//     };

//     let buf_size = info.bytes_per_pixel() * info.width as usize * info.height as usize;

//     let mut buf = vec![0; buf_size];
//     reader.next_frame(&mut buf).unwrap();
//     buf
// }
