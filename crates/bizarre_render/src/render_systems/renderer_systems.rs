use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
};

use anyhow::anyhow;
use ash::vk::ExtPrimitivesGeneratedQueryFn;
use bizarre_logger::core_error;
use specs::{Read, System, Write};

use crate::{render_submitter::RenderSubmitter, scene::RenderScene, Renderer};

pub struct RendererUpdateSystem;

impl RendererUpdateSystem {
    pub const DEFAULT_NAME: &'static str = "renderer_update_system";
}

impl<'a> System<'a> for RendererUpdateSystem {
    type SystemData = (
        Write<'a, RenderSubmitter>,
        Write<'a, RendererResource>,
        Write<'a, RenderScene>,
    );

    fn run(&mut self, (mut submitter, renderer, mut render_scene): Self::SystemData) {
        let render_package = submitter.finalize_submission();
        let render_result = match renderer.lock() {
            Ok(mut r) => r.render(&render_package, &mut render_scene),
            Err(err) => Err(anyhow!("{}", err)),
        };

        if let Err(err) = render_result {
            core_error!("Failed to render the frame: {}", err);
        }
    }
}

#[derive(Default)]
pub struct RendererResource(pub Arc<Option<Mutex<Renderer>>>);
unsafe impl Sync for RendererResource {}
unsafe impl Send for RendererResource {}

impl RendererResource {
    pub fn new(renderer: Renderer) -> Self {
        Self(Arc::new(Some(Mutex::new(renderer))))
    }
}

impl Deref for RendererResource {
    type Target = Mutex<Renderer>;

    fn deref(&self) -> &Self::Target {
        &self
            .0
            .as_ref()
            .as_ref()
            .expect("Trying to deref RendererResource before initialization!")
    }
}
