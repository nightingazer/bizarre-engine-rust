use std::ops::Range;

use nalgebra_glm::{Mat4, Vec3};

use crate::{
    render_components::{Mesh, Transform},
    render_math::{AmbientLight, DirectionalLight},
    render_package::{MeshSubmission, RenderPackage},
    vertex::ColorNormalVertex,
};

pub struct RenderSubmitter {
    meshes: Vec<MeshSubmission>,
    model_matrices: Vec<Mat4>,
    vertex_buffer: Vec<ColorNormalVertex>,
    index_buffer: Vec<u32>,
    clear_color: [f32; 4],
    ambient_light: Option<AmbientLight>,
    directional_lights: Vec<DirectionalLight>,
    view: Mat4,
    projection: Mat4,
    view_projection_was_updated: bool,

    frame_index: usize,
    frame_times_ms: [Option<f64>; 100],
}

impl Default for RenderSubmitter {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderSubmitter {
    pub fn new() -> Self {
        Self {
            meshes: Vec::new(),
            model_matrices: Vec::new(),
            vertex_buffer: Vec::new(),
            index_buffer: Vec::new(),
            clear_color: [0.0, 0.0, 0.0, 1.0],
            directional_lights: Vec::new(),
            ambient_light: None,
            view: Mat4::identity(),
            projection: Mat4::identity(),
            view_projection_was_updated: false,
            frame_index: 0,
            frame_times_ms: [None; 100],
        }
    }

    pub fn submit_vertices(&mut self, mut vertices: Vec<ColorNormalVertex>) {
        self.vertex_buffer.append(&mut vertices);
        let indices: Vec<u32> = (0..vertices.len() as u32).collect();
        let range = self.insert_indices(indices.as_slice());
        let mesh_submission = MeshSubmission {
            index_range: range,
            model_matrix_offset: 0,
        };
        self.meshes.push(mesh_submission);
    }

    pub fn submit_meshes(&mut self, meshes: &[(&Mesh, &Transform)]) {
        self.meshes.reserve(meshes.len());
        for (mesh, transform) in meshes {
            let model_matrix = Mat4::from(*transform);
            let range = self.insert_indices(&mesh.indices);
            self.vertex_buffer.append(&mut mesh.vertices.to_vec());
            let model_matrix_offset = self.model_matrices.len() as u32;
            self.model_matrices.push(model_matrix);

            self.meshes.push(MeshSubmission {
                index_range: range,
                model_matrix_offset,
            })
        }
    }

    pub fn set_clear_color(&mut self, clear_color: [f32; 4]) {
        self.clear_color = clear_color;
    }

    pub fn submit_ambient_light(&mut self, ambient_light: AmbientLight) {
        self.ambient_light = Some(ambient_light);
    }

    pub fn submit_directional_light(&mut self, directional_light: DirectionalLight) {
        self.directional_lights.push(directional_light);
    }

    pub fn update_view(&mut self, view: Mat4) {
        self.view = view;
        self.view_projection_was_updated = true;
    }

    pub fn update_projection(&mut self, projection: Mat4) {
        self.projection = projection;
        self.view_projection_was_updated = true;
    }

    pub fn submit_frame_time(&mut self, frame_time_ms: f64) {
        self.frame_times_ms[self.frame_index] = Some(frame_time_ms);
    }

    pub fn finalize_submission(&mut self) -> RenderPackage {
        let mut model_matrices = [Mat4::default(); 100];
        model_matrices[0] = Mat4::identity();

        for (i, m) in self.model_matrices.iter().enumerate() {
            model_matrices[i] = *m;
        }

        let avg_frame_time = self
            .frame_times_ms
            .iter()
            .filter_map(|t| *t)
            .reduce(|a, b| (a + b) / 2.0)
            .unwrap_or(0.0);

        let last_frame_time = self.frame_times_ms[self.frame_index].unwrap_or(0.0);

        let package = RenderPackage {
            meshes: self.meshes.clone(),
            model_matrices,
            vertices: self.vertex_buffer.clone(),
            indices: self.index_buffer.clone(),
            ambient_light: self.ambient_light.clone(),
            directional_lights: self.directional_lights.clone(),
            clear_color: self.clear_color,
            view: self.view,
            projection: self.projection,
            view_projection_was_updated: self.view_projection_was_updated,
            avg_frame_time_ms: avg_frame_time,
            last_frame_time_ms: last_frame_time,
        };

        self.meshes.clear();
        self.model_matrices.clear();
        self.vertex_buffer.clear();
        self.index_buffer.clear();
        self.directional_lights.clear();
        self.ambient_light = None;
        self.frame_index = (self.frame_index + 1) % self.frame_times_ms.len();

        package
    }

    fn insert_indices(&mut self, indices: &[u32]) -> Range<u32> {
        let first_index = self.vertex_buffer.len() as u32;
        let range_start = self.index_buffer.len() as u32;
        self.index_buffer
            .append(&mut indices.iter().map(|i| i + first_index).collect());
        range_start..self.index_buffer.len() as u32
    }
}
