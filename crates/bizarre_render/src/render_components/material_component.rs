use specs::{prelude::*, Component};

use crate::material_loader::MaterialInstanceHandle;

#[derive(Component)]
pub struct MaterialComponent(pub MaterialInstanceHandle);
