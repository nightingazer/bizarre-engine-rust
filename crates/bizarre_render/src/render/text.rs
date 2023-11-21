use std::sync::Arc;

use nalgebra_glm::{vec2, Vec2, Vec3};
use specs::rayon::iter::ParallelBridge;

use crate::{
    bitmap_font::BitmapFont,
    vertex::{Vertex, Vertex2D},
};

pub struct ScreenText {
    pub font: Arc<BitmapFont>,
    pub text: String,
    /// Font size in px
    pub font_size: f32,
    /// Position on screen in px
    pub position: Vec2,
    pub color: Vec3,
}

impl ScreenText {
    pub fn vertex_buffer(&self, screen_size: [u32; 2]) -> Vec<Vertex2D> {
        let (bitmap_w, bitmap_h) = self.font.bitmap.size;
        let screen_w = screen_size[0] as f32;
        let screen_h = screen_size[1] as f32;

        let char_positions = self.font.font_config.parse(&self.text).unwrap();
        let base_font_size = self.font.font_config.base_height() as f32;
        let scale = self.font_size / base_font_size;

        let shapes: Vec<Vertex2D> = char_positions
            .into_iter()
            .map(|c| {
                let min_u = c.page_rect.x as f32 / bitmap_w as f32;
                let min_v = c.page_rect.y as f32 / bitmap_h as f32;
                let max_u = c.page_rect.max_x() as f32 / bitmap_w as f32;
                let max_v = c.page_rect.max_y() as f32 / bitmap_h as f32;

                let min_x = c.screen_rect.x as f32 * scale / screen_w;
                let min_y = c.screen_rect.y as f32 * scale / screen_h;
                let max_x = c.screen_rect.max_x() as f32 * scale / screen_w;
                let max_y = c.screen_rect.max_y() as f32 * scale / screen_h;

                vec![
                    Vertex2D {
                        position: vec2(min_x, min_y),
                        uv: vec2(min_u, max_v),
                        color: self.color,
                    },
                    Vertex2D {
                        position: vec2(min_x, max_y),
                        uv: vec2(min_u, min_v),
                        color: self.color,
                    },
                    Vertex2D {
                        position: vec2(max_x, max_y),
                        uv: vec2(max_u, min_v),
                        color: self.color,
                    },
                    Vertex2D {
                        position: vec2(max_x, min_y),
                        uv: vec2(max_u, max_v),
                        color: self.color,
                    },
                ]
            })
            .fold(Vec::new(), |acc, e| [acc, e].concat());

        shapes
    }

    pub fn index_buffer(&self) -> Vec<u32> {
        let mut result = Vec::with_capacity(self.text.len() * 6);

        for i in 0..self.text.len() as u32 {
            result.push(4 * i);
            result.push(4 * i + 1);
            result.push(4 * i + 2);
            result.push(4 * i);
            result.push(4 * i + 2);
            result.push(4 * i + 3);
        }

        result
    }
}
