use specs::{Component, VecStorage};

#[derive(Debug)]
pub struct CubeMesh {}

impl Component for CubeMesh {
    type Storage = VecStorage<Self>;
}
