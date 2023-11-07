use std::default;

use specs::{Component, VecStorage};

#[derive(Debug)]
pub struct CubeMesh {}

impl Component for CubeMesh {
    type Storage = VecStorage<Self>;
}

pub struct Transform {
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub scale: [f32; 3],
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Default::default(),
            rotation: Default::default(),
            scale: [1.0; 3],
        }
    }
}

impl Component for Transform {
    type Storage = VecStorage<Self>;
}
