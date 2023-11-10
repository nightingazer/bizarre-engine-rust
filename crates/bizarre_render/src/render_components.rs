use std::default;

use nalgebra_glm::{vec3, Vec3};
use specs::{Component, VecStorage};

use crate::mesh::Mesh;

impl Component for Mesh {
    type Storage = VecStorage<Self>;
}

pub struct Transform {
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
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
