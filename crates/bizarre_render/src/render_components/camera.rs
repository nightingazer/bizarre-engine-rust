use nalgebra_glm::{
    look_at, ortho, perspective, quat_angle, quat_angle_axis, quat_euler_angles, quat_rotate_vec3,
    vec3, Mat4, Quat, Vec2, Vec3,
};
use specs::{Component, VecStorage};

pub enum CameraProjection {
    Perspective {
        fovy: f32,
        aspect: f32,
        near: f32,
        far: f32,
    },
    Orthographic {
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    },
}

pub struct Camera {
    pub yaw: f32,
    pub pitch: f32,
    pub target: Vec3,
    pub distance: f32,
    pub projection: CameraProjection,
}

impl Component for Camera {
    type Storage = VecStorage<Self>;
}

impl Camera {
    pub fn with_projection(projection: CameraProjection) -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            distance: 5.0,
            target: Vec3::zeros(),
            projection,
        }
    }

    pub fn update_aspect_ratio(&mut self, aspect: f32) {
        if let CameraProjection::Perspective {
            fovy, near, far, ..
        } = self.projection
        {
            self.projection = CameraProjection::Perspective {
                fovy,
                aspect,
                near,
                far,
            };
        }
    }

    pub fn get_view_mat(&self) -> Mat4 {
        let up = Vec3::y_axis();
        let arm = quat_rotate_vec3(&self.orientation(), &Vec3::z_axis()) * self.distance;
        let eye = self.target + arm;
        look_at(&eye, &self.target, &up)
    }

    pub fn get_projection_mat(&self) -> Mat4 {
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

    pub fn get_view_projection_mat(&self) -> Mat4 {
        self.get_projection_mat() * self.get_view_mat()
    }

    pub fn orientation(&self) -> Quat {
        Quat::identity()
            * quat_angle_axis(self.yaw, &Vec3::y_axis())
            * quat_angle_axis(self.pitch, &Vec3::x_axis())
    }

    pub fn rotate_quat(&mut self, delta: &Quat) {
        let angles = quat_euler_angles(delta);
        self.yaw += angles.y;
        self.pitch += angles.x;
    }

    pub fn rotate_euler(&mut self, angles: &Vec2) {
        self.pitch -= angles.x;
        self.yaw -= angles.y;
    }

    pub fn right(&self) -> Vec3 {
        let right = vec3(1.0, 0.0, 0.0);
        quat_rotate_vec3(&self.orientation(), &right)
    }

    pub fn up(&self) -> Vec3 {
        let up = vec3(0.0, 1.0, 0.0);
        quat_rotate_vec3(&self.orientation(), &up)
    }

    pub fn forward(&self) -> Vec3 {
        let forward = vec3(0.0, 0.0, -1.0);
        quat_rotate_vec3(&self.orientation(), &forward)
    }
}
