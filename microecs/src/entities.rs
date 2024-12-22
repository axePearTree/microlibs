use alloc::vec::Vec;
use hashbrown::HashMap;

use crate::{components::ChunkComponents, Error};

#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Entity(pub(crate) u64);

pub struct Entities<'a>(pub(crate) &'a ChunkEntities);

pub(crate) struct ChunkEntities {
    indexes: HashMap<Entity, usize>,
    id: Vec<Entity>,
    entity_id_generator: Entity,
}

impl ChunkEntities {
    pub fn new() -> Self {
        Self {
            indexes: HashMap::new(),
            id: Vec::new(),
            entity_id_generator: Entity(0),
        }
    }

    pub fn spawn(&mut self, components: &mut ChunkComponents) -> Result<Entity, Error> {
        let id = self.entity_id_generator;
        let index = self.id.len();
        self.id.push(id);
        components.push_none()?;
        self.indexes.insert(id, index);
        self.entity_id_generator = Entity(self.entity_id_generator.0 + 1);
        Ok(id)
    }

    pub fn destroy(&mut self, components: &mut ChunkComponents, id: Entity) -> Result<(), Error> {
        let Some(index) = self.indexes.remove(&id) else {
            return Err(Error::InvalidEntity(id));
        };
        let last_row = self.id.last().cloned().unwrap();
        self.id.swap_remove(index);
        components.swap_remove(index)?;
        if !self.id.is_empty() && last_row != id {
            self.indexes.insert(last_row, index);
        }
        Ok(())
    }

    pub fn index(&self, entity: Entity) -> Option<usize> {
        self.indexes.get(&entity).copied()
    }

    pub fn iter(&self) -> impl Iterator<Item = Entity> + use<'_> {
        self.id.iter().copied()
    }
}
