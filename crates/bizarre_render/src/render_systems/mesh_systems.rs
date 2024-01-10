use specs::{Join, ParJoin, ReadStorage, System, Write};

use crate::{
    render_components::{MeshComponent, TransformComponent},
    render_package::DrawSubmission,
    render_submitter::RenderSubmitter,
};

pub struct DrawMeshSystem;

impl<'a> System<'a> for DrawMeshSystem {
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
