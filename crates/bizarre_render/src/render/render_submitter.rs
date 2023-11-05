use crate::{
    render_math::{AmbientLight, DirectionalLight},
    render_package::RenderPackage,
    vertex::VertexData,
};

pub struct RenderSubmitter {
    vertex_buffer: Vec<VertexData>,
    clear_color: [f32; 4],
    ambient_light: AmbientLight,
    directional_light: DirectionalLight,
}

impl Default for RenderSubmitter {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderSubmitter {
    pub fn new() -> Self {
        Self {
            vertex_buffer: Vec::new(),
            clear_color: [0.0, 0.0, 0.0, 1.0],
            ambient_light: AmbientLight::default(),
            directional_light: DirectionalLight::default(),
        }
    }

    pub fn submit_vertices(&mut self, vertices: &mut Vec<VertexData>) {
        self.vertex_buffer.append(vertices);
    }

    pub fn set_clear_color(&mut self, clear_color: [f32; 4]) {
        self.clear_color = clear_color;
    }

    pub fn set_ambient_light(&mut self, ambient_light: AmbientLight) {
        self.ambient_light = ambient_light;
    }

    pub fn set_directional_light(&mut self, directional_light: DirectionalLight) {
        self.directional_light = directional_light;
    }

    pub fn get_directional_light(&self) -> &DirectionalLight {
        &self.directional_light
    }

    pub fn finalize_submission(&mut self) -> RenderPackage {
        let package = RenderPackage {
            vertices: self.vertex_buffer.clone(),
            ambient_light: self.ambient_light.clone(),
            directional_light: self.directional_light.clone(),
            clear_color: self.clear_color,
        };

        self.vertex_buffer.clear();

        package
    }
}
