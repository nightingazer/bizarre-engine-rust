use std::sync::Arc;

use nalgebra_glm::{vec2, Vec2, Vec3};

use crate::{bitmap_font::BitmapFont, vertex::Vertex2D};

pub enum TextAlignment {
    Left,
    Center,
    Right,
}

pub struct ScreenText {
    pub font: Arc<BitmapFont>,
    pub text: String,
    /// Font size in px
    pub font_size: f32,
    /// Position on screen in px. Origin is top left corner.
    pub position: Vec2,
    pub color: Vec3,
    pub alignment: TextAlignment,
}

impl ScreenText {
    pub fn vertex_buffer(&self, screen_size: [u32; 2]) -> Vec<Vertex2D> {
        let (bitmap_w, bitmap_h) = self.font.bitmap.size;
        let screen_size = vec2(screen_size[0] as f32, screen_size[1] as f32);
        let screen_w = screen_size[0];
        let screen_h = screen_size[1];

        let base_font_size = self.font.font_config.base_height() as f32;
        let scale = self.font_size / base_font_size;

        let char_positions = self
            .font
            .font_config
            .parse(&self.text)
            .unwrap()
            .collect::<Vec<_>>();

        let text_width = {
            let mut min_x = f32::MIN;
            let mut max_x = f32::MAX;

            for c in char_positions.iter() {
                min_x = min_x.min(c.screen_rect.x as f32);
                max_x = max_x.max(c.screen_rect.max_x() as f32);
            }

            max_x - min_x
        };

        let text_width = text_width * scale;
        let text_height = base_font_size * scale;

        let origin_y = screen_h - self.position.y + text_height / 2.0;

        let text_origin = match self.alignment {
            TextAlignment::Left => vec2(self.position.x, origin_y),
            TextAlignment::Center => vec2(self.position.x - text_width / 2.0, origin_y),
            TextAlignment::Right => vec2(self.position.x - text_width, origin_y),
        };

        let text_origin =
            (vec2(text_origin.x / screen_w, text_origin.y / screen_h) - vec2(0.5, 0.5)) * 2.0;

        let shapes: Vec<Vertex2D> = char_positions
            .iter()
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
                        position: vec2(min_x, min_y) + text_origin,
                        uv: vec2(min_u, max_v),
                        color: self.color,
                    },
                    Vertex2D {
                        position: vec2(min_x, max_y) + text_origin,
                        uv: vec2(min_u, min_v),
                        color: self.color,
                    },
                    Vertex2D {
                        position: vec2(max_x, max_y) + text_origin,
                        uv: vec2(max_u, min_v),
                        color: self.color,
                    },
                    Vertex2D {
                        position: vec2(max_x, min_y) + text_origin,
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
