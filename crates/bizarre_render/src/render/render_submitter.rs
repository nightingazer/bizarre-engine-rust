use nalgebra_glm::Vec3;

use crate::{
    mesh::Mesh,
    render_math::{AmbientLight, DirectionalLight},
    render_package::RenderPackage,
    vertex::VertexData,
};

pub struct RenderSubmitter {
    vertex_buffer: Vec<VertexData>,
    index_buffer: Vec<u32>,
    clear_color: [f32; 4],
    ambient_light: AmbientLight,
    directional_lights: Vec<DirectionalLight>,
    camera_position: Vec3,
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
            index_buffer: Vec::new(),
            clear_color: [0.0, 0.0, 0.0, 1.0],
            ambient_light: AmbientLight::default(),
            directional_lights: Vec::new(),
            camera_position: Vec3::from([0., -2.5, 5.0]),
        }
    }

    pub fn submit_vertices(&mut self, mut vertices: Vec<VertexData>) {
        self.vertex_buffer.append(&mut vertices);
        let indices: Vec<u32> = (0..vertices.len() as u32).collect();
        self.insert_indices(indices.as_slice());
    }

    pub fn submit_meshes(&mut self, meshes: &[Mesh]) {
        for mesh in meshes {
            self.submit_vertices(mesh.vertices.clone());
            self.insert_indices(&mesh.indices);
        }
    }

    pub fn set_clear_color(&mut self, clear_color: [f32; 4]) {
        self.clear_color = clear_color;
    }

    pub fn set_ambient_light(&mut self, ambient_light: AmbientLight) {
        self.ambient_light = ambient_light;
    }

    pub fn submit_directional_light(&mut self, directional_light: DirectionalLight) {
        self.directional_lights.push(directional_light);
    }

    pub fn submit_camera_position(&mut self, position: Vec3) {
        self.camera_position = position;
    }

    pub fn finalize_submission(&mut self) -> RenderPackage {
        let package = RenderPackage {
            vertices: self.vertex_buffer.clone(),
            indices: self.index_buffer.clone(),
            ambient_light: self.ambient_light.clone(),
            directional_lights: self.directional_lights.clone(),
            clear_color: self.clear_color,
            camera_position: self.camera_position,
        };

        self.vertex_buffer.clear();
        self.index_buffer.clear();
        self.directional_lights.clear();

        package
    }

    fn insert_indices(&mut self, indices: &[u32]) {
        if self.index_buffer.is_empty() {
            self.index_buffer = indices.into();
        } else {
            let first_index = self.index_buffer.last().unwrap() + 1;
            self.index_buffer
                .append(&mut indices.iter().map(|i| i + first_index).collect());
        }
    }
}
