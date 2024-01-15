use nalgebra_glm::{
    look_at, look_at_lh, look_at_rh, ortho, perspective, quat_angle_axis, quat_rotate_vec3,
    rotate_vec3, Mat4, Quat, Vec3,
};
use specs::{Component, VecStorage};

use super::{Camera, CameraProjection};

/// A free camera that can be moved around.
#[derive(Debug, Clone, Component)]
#[storage(VecStorage)]
pub struct FreeCameraComponent {
    pub projection: CameraProjection,
    pub position: Vec3,
    pub up: Vec3,
    pub yaw: f32,
    pub pitch: f32,
}

impl FreeCameraComponent {
    pub fn new(projection: CameraProjection) -> Self {
        Self {
            projection,
            position: Vec3::zeros(),
            up: Vec3::y(),
            yaw: 0.0,
            pitch: 0.0,
        }
    }

    pub fn forward(&self) -> Vec3 {
        quat_rotate_vec3(&self.orientation(), &Vec3::z())
    }

    pub fn right(&self) -> Vec3 {
        quat_rotate_vec3(&self.orientation(), &-Vec3::x())
    }

    pub fn orientation(&self) -> Quat {
        Quat::identity()
            * quat_angle_axis(self.yaw.to_radians(), &Vec3::y_axis())
            * quat_angle_axis(self.pitch.to_radians(), &Vec3::x_axis())
    }

    pub fn update_aspect_ratio(&mut self, new_aspect: f32) {
        match &mut self.projection {
            CameraProjection::Perspective {
                aspect: aspect_ref, ..
            } => {
                *aspect_ref = new_aspect;
            }
            CameraProjection::Orthographic { .. } => {
                todo!()
            }
        }
    }
}

impl Camera for FreeCameraComponent {
    fn get_view_mat(&self) -> Mat4 {
        look_at(&self.position, &(self.position + self.forward()), &self.up)
    }

    fn get_projection_mat(&self) -> Mat4 {
        match self.projection {
            CameraProjection::Perspective {
                fovy,
                aspect,
                near,
                far,
            } => perspective(aspect, fovy, near, far),
            CameraProjection::Orthographic {
                left,
                right,
                bottom,
                top,
                near,
                far,
            } => ortho(left, right, bottom, top, near, far),
        }
    }
}
