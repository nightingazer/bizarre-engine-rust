use std::{
    any::{Any, TypeId},
    boxed::ThinBox,
    collections::HashMap,
    marker::Unsize,
    ops::{Deref, DerefMut},
};

pub struct ErasedStorage {
    items: HashMap<TypeId, Box<dyn Any>>,
    dyn_items: HashMap<TypeId, ThinBox<dyn Any>>,
}

impl ErasedStorage {
    pub fn new() -> Self {
        Self {
            items: Default::default(),
            dyn_items: Default::default(),
        }
    }

    pub fn put<T: 'static>(&mut self, item: T) {
        self.items.insert(TypeId::of::<T>(), Box::new(item));
    }

    pub fn put_dyn<T: 'static + ?Sized>(&mut self, item: impl Unsize<T>) {
        self.put_dyn_boxed(ThinBox::<T>::new_unsize(item));
    }

    pub fn put_dyn_boxed<T: 'static + ?Sized>(&mut self, item: ThinBox<T>) {
        let any = unsafe { std::mem::transmute::<_, ThinBox<dyn Any>>(item) };
        self.dyn_items.insert(TypeId::of::<T>(), any);
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        let any = self.items.get(&TypeId::of::<T>());
        any.map(|value| value.downcast_ref::<T>().unwrap())
    }

    pub fn get_dyn<T: 'static + ?Sized>(&self) -> Option<&T> {
        let any = self.dyn_items.get(&TypeId::of::<T>());
        any.map(|value| unsafe { std::mem::transmute::<_, &ThinBox<T>>(value) }.deref())
    }

    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        let any = self.items.get_mut(&TypeId::of::<T>());
        any.map(|value| value.downcast_mut::<T>().unwrap())
    }

    pub fn get_dyn_mut<T: 'static + ?Sized>(&mut self) -> Option<&mut T> {
        let any = self.dyn_items.get_mut(&TypeId::of::<T>());
        any.map(|value| unsafe { std::mem::transmute::<_, &mut ThinBox<T>>(value) }.deref_mut())
    }
}
