#![no_std]

extern crate alloc;

mod components;
mod entities;
mod query;
mod resources;
mod systems;

use alloc::{boxed::Box, collections::vec_deque::VecDeque};
use components::{ChunkComponents, ComponentsBuilder, ComponentsMut, ComponentsRef};
use entities::{ChunkEntities, Entity};
use prelude::ResourcesBuilder;
use resources::Resources;
use spin::{RwLock, RwLockWriteGuard};
use systems::SystemsContext;

pub mod prelude {
    pub use crate::components::{ComponentsMut, ComponentsRef};
    pub use crate::entities::{Entity, Entities};
    pub use crate::query::*;
    pub use crate::resources::{ResourceMut, ResourceRef, Resources, ResourcesBuilder, ItemMut, ItemRef};
    pub use crate::systems::{System, SystemsContext};
    pub use crate::{Chunk, ChunkBuilder, CommandQueue, Commands};
}

#[derive(Clone, Debug)]
pub enum Error {
    InvalidEntity(Entity),
    InternalStorageError(&'static str),
    ComponentNotRegistered(&'static str),
    ComponentAlreadyBorrowedMutably(&'static str),
    ResourceNotFound(&'static str),
    ResourceAlreadyBorrowedMutably(&'static str),
    CorruptedResource(&'static str),
    CommandQueueMissing,
    CommandQueueAlreadyBorrowedMutably,
}

#[derive(Default)]
pub struct ChunkBuilder {
    components_builder: ComponentsBuilder,
    items_builder: ResourcesBuilder,
}

impl ChunkBuilder {
    pub fn with_component<T: 'static>(mut self) -> Self {
        self.components_builder = self.components_builder.with_component::<T>();
        self
    }

    pub fn with_item<T: 'static>(mut self, value: T) -> Self {
        self.items_builder = self.items_builder.with_resource::<T>(value);
        self
    }

    pub fn build(self) -> Chunk {
        Chunk {
            entities: ChunkEntities::new(),
            components: self.components_builder.build(),
            items: self.items_builder.build(),
        }
    }
}

pub struct Chunk {
    entities: ChunkEntities,
    components: ChunkComponents,
    items: Resources,
}

impl Chunk {
    pub fn with<'a>(
        &'a mut self,
        resources: &'a mut Resources,
        command_queue: &'a mut CommandQueue,
    ) -> SystemsContext<'a> {
        SystemsContext::new(self, resources, command_queue)
    }

    #[inline]
    pub fn spawn(&mut self) -> Result<Entity, Error> {
        self.entities.spawn(&mut self.components)
    }

    #[inline]
    pub fn destroy(&mut self, entity: Entity) -> Result<(), Error> {
        self.entities.destroy(&mut self.components, entity)
    }

    pub fn add_component<T: 'static>(&mut self, entity: Entity, value: T) -> Result<(), Error> {
        let index = self
            .entities
            .index(entity)
            .ok_or(Error::InvalidEntity(entity))?;
        self.components_mut::<T>()?.values.set(index, Some(value));
        Ok(())
    }

    pub fn remove_component<T: 'static>(&mut self, entity: Entity) -> Result<(), Error> {
        let index = self
            .entities
            .index(entity)
            .ok_or(Error::InvalidEntity(entity))?;
        self.components_mut::<T>()?.values.set(index, None);
        Ok(())
    }

    #[inline]
    pub fn components_ref<T: 'static>(&self) -> Result<ComponentsRef<T>, Error> {
        self.components.components_ref::<T>(&self.entities)
    }

    #[inline]
    pub fn components_mut<T: 'static>(&self) -> Result<ComponentsMut<T>, Error> {
        self.components.components_mut::<T>(&self.entities)
    }
}

pub struct CommandQueue(RwLock<VecDeque<Command>>);

impl CommandQueue {
    pub fn new() -> Self {
        Self(RwLock::new(VecDeque::new()))
    }

    pub fn flush(&mut self, chunk: &mut Chunk, resources: &mut Resources) -> Result<(), Error> {
        let mut command_queue = self
            .0
            .try_write()
            .ok_or(Error::CommandQueueAlreadyBorrowedMutably)?;
        while let Some(command) = command_queue.pop_front() {
            (command)(chunk, resources)?;
        }
        Ok(())
    }

    pub(crate) fn deferred_commands(&self) -> Result<Commands, Error> {
        self.0
            .try_write()
            .ok_or(Error::CommandQueueAlreadyBorrowedMutably)
            .map(Commands)
    }
}

type Command = Box<dyn Fn(&mut Chunk, &mut Resources) -> Result<(), Error> + Send + Sync>;

pub struct Commands<'a>(pub(crate) RwLockWriteGuard<'a, VecDeque<Command>>);

impl Commands<'_> {
    pub fn defer(
        &mut self,
        command: impl Fn(&mut Chunk, &mut Resources) -> Result<(), Error> + Send + Sync + 'static,
    ) {
        self.0.push_back(Box::new(command));
    }
}
