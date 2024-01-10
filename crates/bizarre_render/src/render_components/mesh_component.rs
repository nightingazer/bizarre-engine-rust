use std::ops::Deref;

use specs::{Component, VecStorage};

use crate::mesh_loader::MeshHandle;

#[derive(Clone, Debug)]
pub struct MeshComponent(pub MeshHandle);

impl Component for MeshComponent {
    type Storage = VecStorage<Self>;
}

impl Deref for MeshComponent {
    type Target = MeshHandle;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
