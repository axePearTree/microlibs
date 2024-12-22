use crate::entities::{ChunkEntities, Entity};
use crate::Error;
use alloc::{boxed::Box, vec::Vec};
use core::any::type_name;
use core::any::{Any, TypeId};
use hashbrown::HashMap;
use spin::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub struct ComponentsRef<'a, T> {
    entities: &'a ChunkEntities,
    pub(crate) values: RwLockReadGuard<'a, ComponentsImpl<T>>,
}

pub struct ComponentsMut<'a, T> {
    entities: &'a ChunkEntities,
    pub(crate) values: RwLockWriteGuard<'a, ComponentsImpl<T>>,
}

impl<'a, T> ComponentsRef<'a, T> {
    pub fn get(&self, entity: Entity) -> Option<&T> {
        let index = self.entities.index(entity)?;
        self.values.get(index)
    }
}

impl<'a, T> ComponentsMut<'a, T> {
    pub fn insert(&mut self, entity: Entity, value: T) -> Result<(), Error> {
        let index = self
            .entities
            .index(entity)
            .ok_or(Error::InvalidEntity(entity))?;
        self.values.set(index, Some(value));
        Ok(())
    }

    pub fn remove(&mut self, entity: Entity) -> Result<(), Error> {
        let index = self
            .entities
            .index(entity)
            .ok_or(Error::InvalidEntity(entity))?;
        self.values.set(index, None);
        Ok(())
    }

    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        let index = self.entities.index(entity)?;
        self.values.get_mut(index)
    }

    pub fn get(&self, entity: Entity) -> Option<&T> {
        let index = self.entities.index(entity)?;
        self.values.get(index)
    }
}

#[derive(Default)]
pub(crate) struct ComponentsBuilder(HashMap<TypeId, Box<dyn ComponentStorage>>);

impl ComponentsBuilder {
    pub fn with_component<T: 'static>(mut self) -> Self {
        let vec: ComponentsImpl<T> = ComponentsImpl::new();
        self.0.insert(TypeId::of::<T>(), Box::new(RwLock::new(vec)));
        self
    }

    pub fn build(self) -> ChunkComponents {
        ChunkComponents(self.0)
    }
}

pub(crate) struct ChunkComponents(HashMap<TypeId, Box<dyn ComponentStorage>>);

impl ChunkComponents {
    pub fn components_ref<'a, T: 'static>(
        &'a self,
        entities: &'a ChunkEntities,
    ) -> Result<ComponentsRef<T>, Error> {
        let values = self
            .components_rwlock()?
            .try_read()
            .ok_or(Error::ComponentAlreadyBorrowedMutably(type_name::<T>()))?;
        Ok(ComponentsRef { entities, values })
    }

    pub fn components_mut<'a, T: 'static>(
        &'a self,
        entities: &'a ChunkEntities,
    ) -> Result<ComponentsMut<T>, Error> {
        let values = self
            .components_rwlock()?
            .try_write()
            .ok_or(Error::ComponentAlreadyBorrowedMutably(type_name::<T>()))?;
        Ok(ComponentsMut { entities, values })
    }

    pub fn push_none(&mut self) -> Result<(), Error> {
        for column in self.0.values_mut() {
            column.push_none()?;
        }
        Ok(())
    }

    pub fn swap_remove(&mut self, index: usize) -> Result<(), Error> {
        for column in self.0.values_mut() {
            column.swap_remove(index)?;
        }
        Ok(())
    }

    fn components_rwlock<T: 'static>(&self) -> Result<&RwLock<ComponentsImpl<T>>, Error> {
        Ok(self
            .0
            .get(&TypeId::of::<T>())
            .ok_or(Error::ComponentNotRegistered(type_name::<T>()))?
            .as_any()
            .downcast_ref::<RwLock<ComponentsImpl<T>>>()
            .ok_or(Error::InternalStorageError(type_name::<T>()))?)
    }
}

pub(crate) trait ComponentStorage {
    fn as_any(&self) -> &dyn Any;
    fn swap_remove(&mut self, index: usize) -> Result<(), Error>;
    fn push_none(&mut self) -> Result<(), Error>;
}

impl<T> ComponentStorage for RwLock<ComponentsImpl<T>>
where
    T: 'static,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn swap_remove(&mut self, index: usize) -> Result<(), Error> {
        self.try_write()
            .ok_or(Error::ComponentAlreadyBorrowedMutably(type_name::<T>()))?
            .swap_remove(index);
        Ok(())
    }

    fn push_none(&mut self) -> Result<(), Error> {
        self.try_write()
            .ok_or(Error::ComponentAlreadyBorrowedMutably(type_name::<T>()))?
            .push(None);
        Ok(())
    }
}

pub struct ComponentsImpl<T>(Vec<Option<T>>);

impl<T> ComponentsImpl<T> {
    fn new() -> Self {
        Self(Vec::new())
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = Option<&T>> + use<'_, T> {
        self.0.iter().map(|v| v.as_ref())
    }

    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = Option<&mut T>> + use<'_, T> {
        self.0.iter_mut().map(|v| v.as_mut())
    }

    #[inline]
    pub fn set(&mut self, index: usize, value: Option<T>) {
        self.0[index] = value;
    }

    #[inline]
    fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.0[index].as_mut()
    }

    #[inline]
    fn get(&self, index: usize) -> Option<&T> {
        self.0[index].as_ref()
    }

    #[inline]
    fn push(&mut self, value: Option<T>) {
        self.0.push(value);
    }

    #[inline]
    fn swap_remove(&mut self, index: usize) {
        self.0.swap_remove(index);
    }
}
