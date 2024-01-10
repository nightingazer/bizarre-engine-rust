use std::{
    hash::{Hash, Hasher},
    marker::PhantomData,
    ops::Deref,
};

#[derive(Clone, Debug)]
pub struct Handle<T>(pub usize, PhantomData<T>);

impl<T: Clone> Copy for Handle<T> {}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> PartialOrd for Handle<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Eq for Handle<T> {}

impl<T> Ord for Handle<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T> Hash for Handle<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<T> Handle<T> {
    pub fn new(id: usize) -> Self {
        Self(id, PhantomData)
    }

    pub fn null() -> Self {
        Self(0, PhantomData)
    }
}

impl<T> Deref for Handle<T> {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
