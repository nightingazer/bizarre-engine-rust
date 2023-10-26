use std::{
    ops::Deref,
    sync::{Arc, Mutex, RwLock},
};

use crate::{event::Event, storage::ErasedStorage};

pub trait System {
    fn initialize(event_bus: &EventBus, system: SyncSystem<Self>)
    where
        Self: Sized;
}

pub trait Handler<E: Event, S: System> {
    fn handle(&mut self, system: &mut S, event: &E);
}

impl<S: System, E: Event, F: Fn(&mut S, &E)> Handler<E, S> for F {
    fn handle(&mut self, system: &mut S, event: &E) {
        self(system, event)
    }
}

struct InternalSystem<S: System> {
    state: S,
    handlers: ErasedStorage,
}

impl<S: System + 'static> InternalSystem<S> {
    pub fn try_handle<E>(&mut self, event: &E)
    where
        E: Event + 'static,
    {
        let handler = self.handlers.get_dyn_mut::<dyn Handler<E, S>>();
        match handler {
            Some(handler) => handler.handle(&mut self.state, event),
            None => {}
        }
    }

    pub fn subscribe<E>(&mut self, handler: impl Handler<E, S> + 'static)
    where
        E: Event + 'static,
    {
        self.handlers.put_dyn::<dyn Handler<E, S>>(handler);
    }
}

pub struct SyncSystem<S: System>(Arc<Mutex<InternalSystem<S>>>);

impl<S: System> SyncSystem<S> {
    pub fn new(system: S) -> Self {
        Self(Arc::new(Mutex::new(InternalSystem {
            state: system,
            handlers: ErasedStorage::new(),
        })))
    }
}

pub trait Caller<E: Event + 'static> {
    fn call(&self, event: &E);
}

impl<S, E> Caller<E> for SyncSystem<S>
where
    S: System + 'static,
    E: Event + 'static,
{
    fn call(&self, event: &E) {
        self.0.lock().unwrap().try_handle(event);
    }
}

pub struct TypedEventBus<E: Event + 'static> {
    listeners: Vec<Box<dyn Caller<E>>>,
}

impl<E> TypedEventBus<E>
where
    E: Event + 'static,
{
    pub fn subscribe<S>(&mut self, system: SyncSystem<S>, handler: impl Handler<E, S> + 'static)
    where
        S: System + 'static,
    {
        system.0.lock().unwrap().subscribe(handler);
        self.listeners.push(Box::new(system));
    }

    pub fn push_event(&self, event: E) {
        for listener in &self.listeners {
            listener.call(&event);
        }
    }
}

pub struct SyncEventBus<E: Event + 'static>(RwLock<TypedEventBus<E>>);

impl<E: Event + 'static> SyncEventBus<E> {
    pub fn new() -> Self {
        Self(RwLock::new(TypedEventBus::<E> {
            listeners: Vec::new(),
        }))
    }
}

impl<E> Deref for SyncEventBus<E>
where
    E: Event + 'static,
{
    type Target = RwLock<TypedEventBus<E>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone)]
pub struct EventBus {
    buses: Arc<RwLock<ErasedStorage>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            buses: Arc::new(RwLock::new(ErasedStorage::new())),
        }
    }

    fn with_new_event_bus<E, F>(&self, f: F)
    where
        E: Event + 'static,
        F: FnOnce(&SyncEventBus<E>),
    {
        let mut lock = self.buses.write().expect("Poisoned RwLock");
        lock.put(SyncEventBus::<E>::new());

        let bus = lock.get().unwrap();
        f(bus);
    }

    fn with_event_bus<E, F>(&self, f: F)
    where
        E: Event + 'static,
        F: FnOnce(&SyncEventBus<E>),
    {
        let lock = self.buses.read().expect("Poisoned RwLock");
        let bus = lock.get();
        match bus {
            Some(bus) => f(bus),
            None => {
                drop(bus);
                drop(lock);
                self.with_new_event_bus(f);
            }
        }
    }

    pub fn push_event<E>(&self, event: E)
    where
        E: Event + 'static,
    {
        self.with_event_bus(|bus| {
            bus.write().unwrap().push_event(event);
        })
    }

    pub fn subscribe<S, E>(&self, system: SyncSystem<S>, handler: impl Handler<E, S> + 'static)
    where
        S: System + 'static,
        E: Event + 'static,
    {
        self.with_event_bus(|bus| {
            bus.write().unwrap().subscribe(system, handler);
        })
    }

    pub fn add_system<S: System + 'static>(&self, system: S) {
        let system = SyncSystem::new(system);
        S::initialize(self, system);
    }
}
