use std::{
    collections::BTreeMap,
    ops::Deref,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, RwLock, RwLockReadGuard, RwLockWriteGuard,
    },
};

use bizarre_common::handle::Handle;
use bizarre_logger::core_critical;

use crate::material::{Material, MaterialInstance};

pub type MaterialHandle = Handle<Material>;
pub type MaterialInstanceHandle = Handle<MaterialInstance>;

pub struct MaterialLoader {
    materials: BTreeMap<MaterialHandle, Arc<Material>>,
    instances: BTreeMap<MaterialInstanceHandle, RwLock<MaterialInstance>>,
    next_material_id: AtomicUsize,
    next_instance_id: AtomicUsize,
    material_map: BTreeMap<String, MaterialHandle>,
    instance_map: BTreeMap<String, MaterialInstanceHandle>,
}

impl Default for MaterialLoader {
    fn default() -> Self {
        Self {
            materials: Default::default(),
            instances: Default::default(),
            material_map: Default::default(),
            instance_map: Default::default(),
            next_material_id: AtomicUsize::new(1),
            next_instance_id: AtomicUsize::new(1),
        }
    }
}

impl MaterialLoader {
    pub fn add_material(&mut self, material: Material, name: String) -> MaterialHandle {
        let id = self.next_material_id.fetch_add(1, Ordering::SeqCst);
        let handle = MaterialHandle::new(id);
        self.materials.insert(handle, Arc::new(material));

        if self.material_map.insert(name.clone(), handle).is_some() {
            panic!("Name conflict! This material loader already has a material named \"{name}\"");
        }

        handle
    }

    pub fn add_instance(
        &mut self,
        instance: MaterialInstance,
        name: String,
    ) -> MaterialInstanceHandle {
        let id = self.next_instance_id.fetch_add(1, Ordering::SeqCst);
        let handle = MaterialInstanceHandle::new(id);
        self.instances.insert(handle, RwLock::new(instance));

        if self.instance_map.insert(name.clone(), handle).is_some() {
            panic!("Name conflict! This material loader already has a material instance named \"{name}\"");
        }

        handle
    }

    pub fn get_material(&self, handle: MaterialHandle) -> Arc<Material> {
        match self.materials.get(&handle) {
            None => {
                let msg = format!("There is no material ({handle:?}) in this MaterialLoader");
                core_critical!(msg);
                panic!("{}", msg);
            }
            Some(material) => material.clone(),
        }
    }

    pub fn find_material_handle(&self, name: &String) -> Option<&MaterialHandle> {
        self.material_map.get(name)
    }

    pub fn get_instance(
        &self,
        handle: MaterialInstanceHandle,
    ) -> RwLockReadGuard<MaterialInstance> {
        match self.instances.get(&handle) {
            None => {
                let msg =
                    format!("There is no material instance ({handle:?}) in this MaterialLoader");
                core_critical!(msg);
                panic!("{}", msg);
            }
            Some(instance) => match instance.read() {
                Ok(lock) => lock,
                Err(err) => {
                    let msg = format!(
                        "Failed to get a read lock for material instance ({handle:?}): {err}"
                    );
                    core_critical!(msg);
                    panic!("{}", msg);
                }
            },
        }
    }

    pub fn get_instance_handle(&self, name: &String) -> Option<&MaterialInstanceHandle> {
        self.instance_map.get(name)
    }

    pub fn get_instance_mut(
        &self,
        handle: MaterialInstanceHandle,
    ) -> RwLockWriteGuard<MaterialInstance> {
        match self.instances.get(&handle) {
            None => {
                let msg =
                    format!("There is no material instance ({handle:?}) in this MaterialLoader");
                core_critical!(msg);
                panic!("{}", msg);
            }
            Some(instance) => match instance.write() {
                Ok(lock) => lock,
                Err(err) => {
                    let msg = format!(
                        "Failed to get a write lock for material instance ({handle:?}): {err}"
                    );
                    core_critical!(msg);
                    panic!("{}", msg);
                }
            },
        }
    }
}
