use std::marker::PhantomData;

use specs::{System, Write};

pub trait Event: Send + Sync + 'static {}

pub struct EventQueue<E: Event> {
    seq_front: Vec<E>,
    seq_back: Vec<E>,
}

impl<E: Event> Default for EventQueue<E> {
    fn default() -> Self {
        Self {
            seq_front: Default::default(),
            seq_back: Default::default(),
        }
    }
}

impl<E: Event> EventQueue<E> {
    pub fn clear(&mut self) {
        self.seq_back.clear();
        self.seq_front.clear();
    }

    pub fn update(&mut self) {
        std::mem::swap(&mut self.seq_front, &mut self.seq_back);
        self.seq_back.clear()
    }

    pub fn get_events(&self) -> &[E] {
        &self.seq_front
    }

    pub fn push_event(&mut self, event: E) {
        self.seq_back.push(event);
    }

    pub fn push_batch<I>(&mut self, events: I)
    where
        I: IntoIterator<Item = E>,
    {
        self.seq_back.extend(events)
    }
}

pub struct EventQueueUpdateSystem<E: Event> {
    _phantom: PhantomData<E>,
}

impl<E: Event> Default for EventQueueUpdateSystem<E> {
    fn default() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }
}

impl<'a, E: Event> System<'a> for EventQueueUpdateSystem<E> {
    type SystemData = (Write<'a, EventQueue<E>>);

    fn run(&mut self, data: Self::SystemData) {
        let (mut queue) = data;
        queue.update();
    }
}
