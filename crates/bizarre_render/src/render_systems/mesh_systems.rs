use bizarre_logger::core_debug;
use specs::{
    shrev::EventChannel, storage::ComponentEvent, Entities, Join, Read, ReadStorage, ReaderId,
    System, SystemData, WorldExt, Write, WriteStorage,
};

use crate::{
    render_components::{MeshComponent, TransformComponent},
    render_package::DrawSubmission,
    render_submitter::RenderSubmitter,
};

#[derive(Default)]
pub struct MeshDrawRequestSystem;

impl MeshDrawRequestSystem {
    pub const DEFAULT_NAME: &'static str = "mesh_draw_request";
}

impl<'a> System<'a> for MeshDrawRequestSystem {
    type SystemData = (
        Write<'a, RenderSubmitter>,
        ReadStorage<'a, MeshComponent>,
        ReadStorage<'a, TransformComponent>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut submitter, meshes, transforms) = data;

        let draw_submissions = (&meshes, &transforms)
            .join()
            .map(|(m, t)| DrawSubmission {
                handle: **m,
                model_matrix: t.into(),
            })
            .collect::<Vec<_>>();

        submitter.submit_draw(&draw_submissions);
    }
}

pub struct MeshManagementSystem {
    pub reader_id: ReaderId<ComponentEvent>,
}

impl MeshManagementSystem {
    pub const DEFAULT_NAME: &'static str = "mesh_management_system";
}

impl<'a> System<'a> for MeshManagementSystem {
    type SystemData = (
        Write<'a, RenderSubmitter>,
        ReadStorage<'a, MeshComponent>,
        Entities<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut submitter, meshes, entities) = data;

        let events = meshes.channel().read(&mut self.reader_id);

        for event in events {
            match event {
                ComponentEvent::Inserted(id) => {
                    let entity = entities.entity(*id);
                    let mesh = meshes.get(entity).unwrap();
                    submitter.upload_mesh(mesh);
                }
                ComponentEvent::Removed(id) => {
                    let entity = entities.entity(*id);
                    let _mesh_handle = meshes.get(entity).unwrap().0;
                    todo!();
                }
                _ => {}
            }
        }
    }
}
