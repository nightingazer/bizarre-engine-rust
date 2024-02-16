use std::collections::HashSet;

use nalgebra_glm::{Mat4, Vec3};

use crate::{
    mesh_loader::MeshHandle,
    render_components::MeshComponent,
    render_math::{AmbientLight, DirectionalLight},
    render_package::{DrawSubmission, MeshUpload, RenderPackage},
};

pub struct RenderSubmitter {
    mesh_uploads: Vec<MeshUpload>,
    draw_submissions: Vec<DrawSubmission>,
    clear_color: [f32; 4],
    ambient_light: Option<AmbientLight>,
    directional_lights: Vec<DirectionalLight>,
    view: Mat4,
    projection: Mat4,
    camera_forward: Option<Vec3>,
    view_was_updated: bool,
    projection_was_updated: bool,

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
            mesh_uploads: Vec::new(),
            draw_submissions: Vec::new(),
            clear_color: [0.0, 0.0, 0.0, 1.0],
            directional_lights: Vec::new(),
            ambient_light: None,
            view: Mat4::identity(),
            projection: Mat4::identity(),
            view_was_updated: false,
            projection_was_updated: false,
            camera_forward: None,
            frame_index: 0,
            frame_times_ms: [None; 100],
        }
    }

    pub fn upload_mesh(&mut self, mesh: &MeshComponent) {
        self.mesh_uploads.push(MeshUpload { mesh: mesh.0 });
    }

    pub fn submit_draw(&mut self, draw_submissions: &[DrawSubmission]) {
        self.draw_submissions.extend_from_slice(draw_submissions)
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
        self.view_was_updated = true;
    }

    pub fn update_projection(&mut self, projection: Mat4) {
        self.projection = projection;
        self.projection_was_updated = true;
    }

    pub fn update_camera_forward(&mut self, camera_forward: Vec3) {
        self.camera_forward = Some(camera_forward);
    }

    pub fn submit_frame_time(&mut self, frame_time_ms: f64) {
        self.frame_times_ms[self.frame_index] = Some(frame_time_ms);
    }

    pub fn finalize_submission(&mut self) -> RenderPackage {
        let avg_frame_time = self
            .frame_times_ms
            .iter()
            .filter_map(|t| *t)
            .reduce(|a, b| (a + b) / 2.0)
            .unwrap_or(0.0);

        let last_frame_time = self.frame_times_ms[self.frame_index].unwrap_or(0.0);

        self.draw_submissions
            .sort_by(|a, b| a.handle.cmp(&b.handle));

        let mut handles = HashSet::<MeshHandle>::new();

        self.mesh_uploads.retain(|e| {
            let is_first = handles.contains(&(e.mesh));
            handles.insert(e.mesh);
            !is_first
        });

        let (view, projection, view_projection) =
            match (self.view_was_updated, self.projection_was_updated) {
                (true, true) => {
                    self.view_was_updated = false;
                    self.projection_was_updated = false;
                    (
                        Some(self.view),
                        Some(self.projection),
                        Some(self.projection * self.view),
                    )
                }

                (true, false) => {
                    self.view_was_updated = false;
                    (Some(self.view), None, Some(self.projection * self.view))
                }
                (false, true) => {
                    self.projection_was_updated = false;
                    (
                        None,
                        Some(self.projection),
                        Some(self.projection * self.view),
                    )
                }
                (false, false) => (None, None, None),
            };

        let package = RenderPackage {
            mesh_uploads: self.mesh_uploads.clone(),
            mesh_deletes: Vec::new(),
            draw_submissions: self.draw_submissions.clone(),
            avg_frame_time_ms: avg_frame_time,
            last_frame_time_ms: last_frame_time,
            view,
            projection,
            view_projection,
            camera_forward: self.camera_forward.take(),
        };

        self.draw_submissions.clear();
        self.mesh_uploads.clear();
        self.frame_index = (self.frame_index + 1) % self.frame_times_ms.len();

        package
    }
}
