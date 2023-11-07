use std::{
    ops::Deref,
    sync::{Arc, Mutex, RwLock},
};

use crate::{event::Event, storage::ErasedStorage};

pub trait Observer {
    fn initialize(event_bus: &EventBus, system: SyncObserver<Self>)
    where
        Self: Sized;
}

pub trait Handler<E: Event, O: Observer> {
    fn handle(&mut self, observer: &mut O, event: &E);
}

impl<O: Observer, E: Event, F: Fn(&mut O, &E)> Handler<E, O> for F {
    fn handle(&mut self, observer: &mut O, event: &E) {
        self(observer, event)
    }
}

struct InternalSystem<O: Observer> {
    state: *mut O,
    handlers: ErasedStorage,
}

impl<O: Observer + 'static> InternalSystem<O> {
    pub fn try_handle<E>(&mut self, event: &E)
    where
        E: Event + 'static,
    {
        let handler = self.handlers.get_dyn_mut::<dyn Handler<E, O>>();
        let state = unsafe { &mut *self.state };
        if let Some(handler) = handler {
            handler.handle(state, event)
        }
    }

    pub fn subscribe<E>(&mut self, handler: impl Handler<E, O> + 'static)
    where
        E: Event + 'static,
    {
        self.handlers.put_dyn::<dyn Handler<E, O>>(handler);
    }
}

pub struct SyncObserver<S: Observer>(Arc<Mutex<InternalSystem<S>>>);

impl<O: Observer> SyncObserver<O> {
    pub fn new(observer: &mut O) -> Self {
        Self(Arc::new(Mutex::new(InternalSystem {
            state: observer as *mut O,
            handlers: ErasedStorage::default(),
        })))
    }
}

pub trait Caller<E: Event + 'static> {
    fn call(&self, event: &E);
}

impl<O, E> Caller<E> for SyncObserver<O>
where
    O: Observer + 'static,
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
    pub fn subscribe<O>(&mut self, system: SyncObserver<O>, handler: impl Handler<E, O> + 'static)
    where
        O: Observer + 'static,
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

impl<E: Event + 'static> Default for SyncEventBus<E> {
    fn default() -> Self {
        Self(RwLock::new(TypedEventBus::<E> {
            listeners: Vec::new(),
        }))
    }
}

impl<E: Event + 'static> SyncEventBus<E> {
    #[deprecated]
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

#[derive(Clone, Default)]
pub struct EventBus {
    buses: Arc<RwLock<ErasedStorage>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            buses: Arc::new(RwLock::new(ErasedStorage::default())),
        }
    }

    fn with_new_event_bus<E, F>(&self, f: F)
    where
        E: Event + 'static,
        F: FnOnce(&SyncEventBus<E>),
    {
        let mut lock = self.buses.write().expect("Poisoned RwLock");
        lock.put(SyncEventBus::<E>::default());

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

    pub fn subscribe<O, E>(&self, observer: SyncObserver<O>, handler: impl Handler<E, O> + 'static)
    where
        O: Observer + 'static,
        E: Event + 'static,
    {
        self.with_event_bus(|bus| {
            bus.write().unwrap().subscribe(observer, handler);
        })
    }

    pub fn add_system<S: Observer + 'static>(&self, observer: &mut S) {
        let observer = SyncObserver::new(observer);
        S::initialize(self, observer);
    }
}
