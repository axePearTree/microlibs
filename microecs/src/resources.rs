use core::any::{type_name, Any, TypeId};

use alloc::boxed::Box;
use hashbrown::HashMap;
use spin::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::Error;

#[derive(Default)]
pub struct Resources {
    values: HashMap<TypeId, Box<dyn Any>>,
}

impl Resources {
    pub fn add_resource<T: 'static>(&mut self, value: T) {
        self.values
            .insert(TypeId::of::<T>(), Box::new(RwLock::new(value)));
    }

    pub fn remove_resource<T: 'static>(&mut self) {
        self.values.remove(&TypeId::of::<T>());
    }

    pub fn resource_ref<T: 'static>(&self) -> Result<ResourceRef<T>, Error> {
        self.resource_rw_lock::<T>()?
            .try_read()
            .ok_or(Error::ResourceAlreadyBorrowedMutably(type_name::<T>()))
            .map(ResourceRef)
    }

    pub fn resource_mut<T: 'static>(&self) -> Result<ResourceMut<T>, Error> {
        self.resource_rw_lock::<T>()?
            .try_write()
            .ok_or(Error::ResourceAlreadyBorrowedMutably(type_name::<T>()))
            .map(ResourceMut)
    }

    pub(crate) fn resource_rw_lock<T: 'static>(&self) -> Result<&RwLock<T>, Error> {
        self.values
            .get(&TypeId::of::<T>())
            .ok_or(Error::ResourceNotFound(type_name::<T>()))?
            .downcast_ref::<RwLock<T>>()
            .ok_or(Error::CorruptedResource(type_name::<T>()))
    }
}

pub struct ResourceRef<'a, T>(pub(crate) RwLockReadGuard<'a, T>);

pub struct ResourceMut<'a, T>(pub(crate) RwLockWriteGuard<'a, T>);

pub struct ItemRef<'a, T>(pub(crate) RwLockReadGuard<'a, T>);

pub struct ItemMut<'a, T>(pub(crate) RwLockWriteGuard<'a, T>);

impl<'a, T> ResourceRef<'a, T> {
    pub fn get(&self) -> &T {
        &self.0
    }

    pub(crate) fn into_item(self) -> ItemRef<'a, T> {
        ItemRef(self.0)
    }
}

impl<'a, T> ResourceMut<'a, T> {
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.0
    }

    pub fn get(&self) -> &T {
        &self.0
    }

    pub(crate) fn into_item_mut(self) -> ItemMut<'a, T> {
        ItemMut(self.0)
    }
}

impl<'a, T> ItemRef<'a, T> {
    pub fn get(&self) -> &T {
        &self.0
    }
}

impl<'a, T> ItemMut<'a, T> {
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.0
    }

    pub fn get(&self) -> &T {
        &self.0
    }
}

#[derive(Default)]
pub struct ResourcesBuilder {
    resources: Resources,
}

impl ResourcesBuilder {
    pub fn with_resource<T: 'static>(mut self, value: T) -> Self {
        self.resources.add_resource(value);
        self
    }

    pub fn build(self) -> Resources {
        self.resources
    }
}
