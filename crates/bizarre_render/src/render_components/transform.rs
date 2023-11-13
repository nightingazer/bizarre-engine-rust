use nalgebra_glm::{
    quat_angle, quat_angle_axis, quat_axis, quat_euler_angles, quat_rotate_vec3, rotate, vec3,
    vec4, vec4_to_vec3, Mat4, Quat, Vec3,
};
use specs::{Component, RunNow, VecStorage};

#[derive(Debug, Clone)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub fn rotate(&mut self, angle: f32, axis: &Vec3) -> &mut Self {
        self.rotation = quat_angle_axis(angle, axis) * self.rotation;
        self
    }

    pub fn rotate_quat(&mut self, rotation: &Quat) -> &mut Self {
        self.rotation *= rotation;
        self
    }

    pub fn rotate_euler(&mut self, euler: &Vec3) -> &mut Self {
        let quat = quat_angle_axis(euler.x, &Vec3::x_axis())
            * quat_angle_axis(euler.y, &Vec3::y_axis())
            * quat_angle_axis(euler.z, &Vec3::z_axis());
        self.rotate_quat(&quat)
    }

    pub fn move_position(&mut self, position_delta: &Vec3) {
        self.position += position_delta;
    }

    pub fn forward(&self) -> Vec3 {
        let forward = vec3(0.0, 0.0, 1.0);
        quat_rotate_vec3(&self.rotation, &forward)
    }

    pub fn up(&self) -> Vec3 {
        let up = vec3(0.0, 1.0, 0.0);
        quat_rotate_vec3(&self.rotation, &up)
    }

    pub fn right(&self) -> Vec3 {
        let right = vec3(1.0, 0.0, 0.0);
        quat_rotate_vec3(&self.rotation, &right)
    }
}

impl From<Transform> for Mat4 {
    fn from(transform: Transform) -> Self {
        let mut mat = Mat4::identity();
        mat = mat.append_nonuniform_scaling(&transform.scale);

        let rotation = transform.rotation.normalize();

        let rotation_axis = quat_axis(&rotation).normalize();
        let rotation_angle = quat_angle(&rotation);

        mat = rotate(&mat, rotation_angle, &rotation_axis);

        mat = mat.append_translation(&transform.position);

        mat
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Default::default(),
            rotation: Default::default(),
            scale: vec3(1.0, 1.0, 1.0),
        }
    }
}

impl Component for Transform {
    type Storage = VecStorage<Self>;
}

// impl From<Mat4> for Transform {
//     fn from(mat: Mat4) -> Self {
//         let position = mat.column(3).xyz();
//         let scale = vec3(
//             mat.column(0).xyz().magnitude(),
//             mat.column(1).xyz().magnitude(),
//             mat.column(2).xyz().magnitude(),
//         );

//         let mut r = mat.clone();
//         r.set_column(3, &vec4(0.0, 0.0, 0.0, 1.0));
//         r.set_row(3, &vec4(0.0, 0.0, 0.0, 1.0).transpose());

//         let threshold: f32 = 0.0;

//         let q1 = if r[(0, 0)] + r[(1, 1)] + r[(2, 2)] > threshold {
//             0.5 * (1.0 + r[(0, 0)] + r[(1, 1)] + r[(2, 2)]).sqrt()
//         } else {
//             let numerator = (r[(2, 1)] - r[(1, 2)]).powi(2)
//                 + (r[(0, 2)] - r[(2, 0)]).powi(2)
//                 + (r[(1, 0)] - r[(0, 1)]).powi(2);
//             let denominator = 3.0 - r[(0, 0)] - r[(1, 1)] - r[(2, 2)];
//             0.5 * (numerator / denominator).sqrt()
//         };

//         let q2 = if r[(0, 0)] - r[(1, 1)] - r[(2, 2)] > threshold {
//             0.5 * (1.0 + r[(0, 0)] - r[(1, 1)] - r[(2, 2)]).sqrt()
//         } else {
//             let numerator = (r[(1, 2)] - r[(2, 1)]).powi(2)
//                 + (r[(1, 0)] - r[(0, 1)]).powi(2)
//                 + (r[(0, 2)] - r[(2, 0)]).powi(2);
//             let denominator = 3.0 - r[(0, 0)] + r[(1, 1)] + r[(2, 2)];
//             0.5 * (numerator / denominator).sqrt()
//         };

//         let q3 = if -r[(0, 0)] - r[(1, 1)] + r[(2, 2)] > threshold {
//             0.5 * (1.0 - r[(0, 0)] + r[(1, 1)] - r[(2, 2)]).sqrt()
//         } else {
//             let numerator = (r[(2, 0)] - r[(0, 2)]).powi(2)
//                 + (r[(1, 0)] + r[(0, 1)]).powi(2)
//                 + (r[(2, 1)] + r[(1, 2)]).powi(2);
//             let denominator = 3.0 + r[(0, 0)] - r[(1, 1)] + r[(2, 2)];
//             0.5 * (numerator / denominator).sqrt()
//         };

//         let q4 = if -r[(0, 0)] - r[(1, 1)] + r[(2, 2)] > threshold {
//             0.5 * (1.0 - r[(0, 0)] - r[(1, 1)] + r[(2, 2)]).sqrt()
//         } else {
//             let numerator = (r[(0, 1)] - r[(1, 0)]).powi(2)
//                 + (r[(2, 0)] + r[(0, 2)]).powi(2)
//                 + (r[(2, 1)] + r[(1, 2)]).powi(2);
//             let denominator = 3.0 + r[(0, 0)] + r[(1, 1)] - r[(2, 2)];
//             0.5 * (numerator / denominator).sqrt()
//         };

//         let rotation = Quat {
//             coords: vec4(q1, q2, q3, q4),
//         };

//         Self {
//             position,
//             scale,
//             rotation,
//         }
//     }
// }
