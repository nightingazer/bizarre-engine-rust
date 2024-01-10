use crate::{render_components::Transform, mesh_loader::MeshHandle};

pub struct RenderObject {
    pub mesh: MeshHandle,
    pub transform: Transform,
}

pub struct IndirectBatch {
    pub mesh: MeshHandle,
    pub first: u32,
    pub count: u32,
}

pub fn compact_draws(objects: &[RenderObject]) -> Vec<IndirectBatch> {
    let draws = Vec::new();

    let first_draw = IndirectBatch

    draws
}
