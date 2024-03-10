use std::{
    collections::BTreeMap,
    sync::{LazyLock, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use anyhow::Result;
use bizarre_common::handle::Handle;
use bizarre_logger::core_error;
use bizarre_memory::{Constructor, SyncArenaChunk, TypedArena};

use crate::mesh::{load_meshes_from_obj, Mesh};

pub type MeshHandle = Handle<Mesh>;

pub struct MeshLoader {
    map: BTreeMap<MeshHandle, Box<Mesh>>,
    arena: TypedArena<Mesh, SyncArenaChunk>,
    next_id: usize,
}

static MESH_LOADER: LazyLock<RwLock<MeshLoader>> =
    LazyLock::new(|| RwLock::new(MeshLoader::default()));

const MESH_ARENA_LEN: usize = 512;

pub fn get_mesh_loader() -> RwLockReadGuard<'static, MeshLoader> {
    MESH_LOADER.read().unwrap()
}

pub fn get_mesh_loader_mut() -> RwLockWriteGuard<'static, MeshLoader> {
    MESH_LOADER.write().unwrap()
}

impl Default for MeshLoader {
    fn default() -> Self {
        Self {
            map: BTreeMap::new(),
            arena: TypedArena::new(MESH_ARENA_LEN),
            next_id: 0,
        }
    }
}

impl MeshLoader {
    pub fn load_obj(&mut self, path: String, names: Option<&[String]>) -> Result<Vec<MeshHandle>> {
        let meshes = load_meshes_from_obj(path, self.next_id, names)?;
        self.next_id += meshes.len();

        let handles = meshes
            .into_iter()
            .map(|mesh| {
                let handle = mesh.id;
                let ptr = self.arena.construct(mesh)?;
                self.map.insert(handle, unsafe { Box::from_raw(ptr) });
                Ok(handle)
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(handles)
    }

    pub fn get(&self, handle: MeshHandle) -> Option<*const Mesh> {
        if !self.map.contains_key(&handle) {
            core_error!("Mesh with handle {:?} not found", handle);
            None
        } else {
            Some(&*self.map[&handle] as *const Mesh)
        }
    }
}
