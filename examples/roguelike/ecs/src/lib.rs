pub use serde;
use serde::{Deserialize, Serialize};
use std::mem;
use std::slice;

type Id = u32;
type Index = u32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entity {
    id: Id,
    index: Index,
}

#[derive(Debug, Default)]
struct IndexToId {
    vec: Vec<Option<Id>>,
}

impl IndexToId {
    fn to_entities(&self) -> Vec<Entity> {
        self.vec
            .iter()
            .enumerate()
            .filter_map(|(index, maybe_id)| {
                maybe_id.map(|id| Entity {
                    id,
                    index: index as u32,
                })
            })
            .collect()
    }
    fn from_entities(entities: Vec<Entity>) -> Self {
        let vec = if let Some(max_index) = entities.iter().map(|e| e.index).max() {
            let mut vec = Vec::with_capacity(max_index as usize + 1);
            vec.resize(max_index as usize + 1, None);
            for entity in &entities {
                vec[entity.index as usize] = Some(entity.id);
            }
            vec
        } else {
            Vec::new()
        };
        Self { vec }
    }
}

impl Serialize for IndexToId {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.to_entities().serialize(s)
    }
}

impl<'a> Deserialize<'a> for IndexToId {
    fn deserialize<D: serde::Deserializer<'a>>(d: D) -> Result<Self, D::Error> {
        Deserialize::deserialize(d).map(Self::from_entities)
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct EntityAllocator {
    next_id: Id,
    next_index: Index,
    index_to_id: IndexToId,
    free_indices: Vec<Index>,
}

impl EntityAllocator {
    pub fn alloc(&mut self) -> Entity {
        let id = self.next_id;
        self.next_id += 1;
        let index = self.free_indices.pop().unwrap_or_else(|| {
            let index = self.next_index;
            self.next_index += 1;
            index
        });
        if index as usize >= self.index_to_id.vec.len() {
            self.index_to_id.vec.resize(index as usize, None);
            self.index_to_id.vec.push(Some(id));
        } else {
            assert_eq!(self.index_to_id.vec[index as usize], None);
            self.index_to_id.vec[index as usize] = Some(id);
        }
        Entity { id, index }
    }
    pub fn exists(&self, entity: Entity) -> bool {
        self.index_to_id.vec[entity.index as usize] == Some(entity.id)
    }
    pub fn free(&mut self, entity: Entity) {
        if self.exists(entity) {
            self.index_to_id.vec[entity.index as usize] = None;
            self.free_indices.push(entity.index);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentTableEntry<T> {
    data: T,
    entity: Entity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentTableEntries<T> {
    vec: Vec<ComponentTableEntry<T>>,
}

impl<T> Default for ComponentTableEntries<T> {
    fn default() -> Self {
        Self {
            vec: Default::default(),
        }
    }
}

impl<T> ComponentTableEntries<T> {
    fn entity_index_to_entry_index(&self) -> Vec<Option<Index>> {
        if let Some(max_index) = self.vec.iter().map(|entry| entry.entity.index).max() {
            let mut vec = Vec::with_capacity(max_index as usize + 1);
            vec.resize(max_index as usize + 1, None);
            for (index, entry) in self.vec.iter().enumerate() {
                vec[entry.entity.index as usize] = Some(index as u32);
            }
            vec
        } else {
            Vec::new()
        }
    }
    pub fn into_component_table(self) -> ComponentTable<T> {
        let entity_index_to_entry_index = self.entity_index_to_entry_index();
        ComponentTable {
            entries: self,
            entity_index_to_entry_index,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ComponentTable<T> {
    entries: ComponentTableEntries<T>,
    entity_index_to_entry_index: Vec<Option<Index>>,
}

impl<T> Default for ComponentTable<T> {
    fn default() -> Self {
        ComponentTableEntries::default().into_component_table()
    }
}

impl<T: Serialize> Serialize for ComponentTable<T> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.entries.serialize(s)
    }
}

impl<'a, T: Deserialize<'a>> Deserialize<'a> for ComponentTable<T> {
    fn deserialize<D: serde::Deserializer<'a>>(d: D) -> Result<Self, D::Error> {
        Deserialize::deserialize(d).map(ComponentTableEntries::into_component_table)
    }
}

impl<T> ComponentTable<T> {
    pub fn is_empty(&self) -> bool {
        self.entries.vec.is_empty()
    }
    pub fn len(&self) -> usize {
        self.entries.vec.len()
    }
    pub fn entries(&self) -> &ComponentTableEntries<T> {
        &self.entries
    }
    pub fn insert(&mut self, entity: Entity, data: T) -> Option<T> {
        if let Some(maybe_entry_index) = self.entity_index_to_entry_index.get_mut(entity.index as usize) {
            if let Some(entry_index) = maybe_entry_index {
                assert!((*entry_index as usize) < self.entries.vec.len());
                let entry = &mut self.entries.vec[*entry_index as usize];
                if entry.entity == entity {
                    Some(mem::replace(&mut entry.data, data))
                } else {
                    entry.entity = entity;
                    entry.data = data;
                    None
                }
            } else {
                *maybe_entry_index = Some(self.entries.vec.len() as u32);
                self.entries.vec.push(ComponentTableEntry { data, entity });
                None
            }
        } else {
            self.entity_index_to_entry_index.resize(entity.index as usize, None);
            self.entity_index_to_entry_index
                .push(Some(self.entries.vec.len() as u32));
            self.entries.vec.push(ComponentTableEntry { data, entity });
            None
        }
    }
    pub fn contains(&self, entity: Entity) -> bool {
        if let Some(Some(entry_index)) = self.entity_index_to_entry_index.get(entity.index as usize) {
            assert!((*entry_index as usize) < self.entries.vec.len());
            self.entries.vec[*entry_index as usize].entity.id == entity.id
        } else {
            false
        }
    }
    pub fn remove(&mut self, entity: Entity) -> Option<T> {
        if let Some(maybe_entry_index) = self.entity_index_to_entry_index.get_mut(entity.index as usize) {
            if let Some(entry_index) = maybe_entry_index.take() {
                assert!((entry_index as usize) < self.entries.vec.len());
                if entry_index as usize == self.entries.vec.len() - 1 {
                    self.entries.vec.pop().map(|entry| entry.data)
                } else {
                    let entry = self.entries.vec.swap_remove(entry_index as usize);
                    let moved_index = self.entries.vec[entry_index as usize].entity.index;
                    self.entity_index_to_entry_index[moved_index as usize] = Some(entry_index);
                    Some(entry.data)
                }
            } else {
                None
            }
        } else {
            None
        }
    }
    pub fn get(&self, entity: Entity) -> Option<&T> {
        if let Some(Some(entry_index)) = self.entity_index_to_entry_index.get(entity.index as usize) {
            assert!((*entry_index as usize) < self.entries.vec.len());
            let entry = &self.entries.vec[*entry_index as usize];
            if entry.entity.id == entity.id {
                Some(&entry.data)
            } else {
                None
            }
        } else {
            None
        }
    }
    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        if let Some(Some(entry_index)) = self.entity_index_to_entry_index.get(entity.index as usize) {
            assert!((*entry_index as usize) < self.entries.vec.len());
            let entry = &mut self.entries.vec[*entry_index as usize];
            if entry.entity.id == entity.id {
                Some(&mut entry.data)
            } else {
                None
            }
        } else {
            None
        }
    }
    pub fn iter(&self) -> ComponentTableIter<T> {
        ComponentTableIter {
            iter: self.entries.vec.iter(),
        }
    }
    pub fn iter_mut(&mut self) -> ComponentTableIterMut<T> {
        ComponentTableIterMut {
            iter: self.entries.vec.iter_mut(),
        }
    }
    pub fn entities(&self) -> impl '_ + Iterator<Item = Entity> {
        self.iter().map(|(entity, _)| entity)
    }
}

pub struct ComponentTableIter<'a, T> {
    iter: slice::Iter<'a, ComponentTableEntry<T>>,
}

pub struct ComponentTableIterMut<'a, T> {
    iter: slice::IterMut<'a, ComponentTableEntry<T>>,
}

impl<'a, T> Iterator for ComponentTableIter<'a, T> {
    type Item = (Entity, &'a T);
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|entry| (entry.entity, &entry.data))
    }
}

impl<'a, T> Iterator for ComponentTableIterMut<'a, T> {
    type Item = (Entity, &'a mut T);
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|entry| (entry.entity, &mut entry.data))
    }
}

#[macro_export]
macro_rules! ecs_components {
    { $module_name:ident { $($component_name:ident: $component_type:ty,)* } } => {
        mod $module_name {
            #[allow(unused_imports)]
            use super::*;

            #[derive(Debug, Clone, $crate::serde::Serialize, $crate::serde::Deserialize)]
            pub struct Components {
                $(pub $component_name: $crate::ComponentTable<$component_type>,)*
            }

            impl Default for Components {
                fn default() -> Self {
                    Self {
                        $($component_name: Default::default(),)*
                    }
                }
            }

            #[derive(Debug, Clone, $crate::serde::Serialize, $crate::serde::Deserialize)]
            pub struct EntityData {
                $(pub $component_name: Option<$component_type>,)*
            }

            impl Default for EntityData {
                fn default() -> Self {
                    Self {
                        $($component_name: None,)*
                    }
                }
            }

            impl Components {
                #[allow(unused)]
                pub fn remove_entity(&mut self, entity: $crate::Entity) {
                    $(self.$component_name.remove(entity);)*
                }
                #[allow(unused)]
                pub fn clone_entity_data(&self, entity: $crate::Entity) -> EntityData {
                    EntityData {
                        $($component_name: self.$component_name.get(entity).cloned(),)*
                    }
                }
                #[allow(unused)]
                pub fn remove_entity_data(&mut self, entity: $crate::Entity) -> EntityData {
                    EntityData {
                        $($component_name: self.$component_name.remove(entity),)*
                    }
                }
                #[allow(unused)]
                pub fn insert_entity_data(&mut self, entity: $crate::Entity, entity_data: EntityData) {
                    $(if let Some(field) = entity_data.$component_name {
                        self.$component_name.insert(entity, field);
                    })*
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn entity_alloc_remove() {
        let mut a = EntityAllocator::default();
        let e0 = a.alloc();
        let e1 = a.alloc();
        let e2 = a.alloc();
        a.free(e1);
        a.free(e1); // second free should be redundant
        let e3 = a.alloc();
        let e4 = a.alloc();
        assert_eq!([e0.id, e1.id, e2.id, e3.id, e4.id], [0, 1, 2, 3, 4]);
        assert_eq!([e0.index, e1.index, e2.index, e3.index, e4.index], [0, 1, 2, 1, 3]);
    }

    #[test]
    fn component_table_insert_remove() {
        let mut a = EntityAllocator::default();
        let e0 = a.alloc();
        let e1 = a.alloc();
        let e2 = a.alloc();
        let mut c = ComponentTable::default();
        c.insert(e0, "zero".to_string());
        c.insert(e1, "one".to_string());
        c.insert(e2, "two".to_string());
        assert_eq!(c.get(e0).unwrap(), "zero");
        assert_eq!(c.get(e1).unwrap(), "one");
        assert_eq!(c.get(e2).unwrap(), "two");
        c.remove(e0);
        assert_eq!(c.get(e0), None);
        assert_eq!(c.get(e1).unwrap(), "one");
        assert_eq!(c.get(e2).unwrap(), "two");
        c.insert(e0, "zero again".to_string());
        assert_eq!(c.get(e0).unwrap(), "zero again");
        assert_eq!(c.get(e1).unwrap(), "one");
        assert_eq!(c.get(e2).unwrap(), "two");
        c.insert(e1, "one again".to_string());
        *c.get_mut(e2).unwrap() = "two again".to_string();
        assert_eq!(c.get(e0).unwrap(), "zero again");
        assert_eq!(c.get(e1).unwrap(), "one again");
        assert_eq!(c.get(e2).unwrap(), "two again");
        a.free(e0);
        let raw_entries = c.entries.vec.iter().map(|e| e.data.clone()).collect::<Vec<_>>();
        assert_eq!(raw_entries, ["two again", "one again", "zero again"]);
        assert_eq!(c.entity_index_to_entry_index, [Some(2), Some(1), Some(0)]);
        let e3 = a.alloc();
        assert_eq!(e3.index, 0);
        assert_eq!(c.get(e3), None);
        c.insert(e3, "three".to_string());
        assert_eq!(c.get(e3).unwrap(), "three");
        let raw_entries = c.entries.vec.iter().map(|e| e.data.clone()).collect::<Vec<_>>();
        assert_eq!(raw_entries, ["two again", "one again", "three"]);
        assert_eq!(c.entity_index_to_entry_index, [Some(2), Some(1), Some(0)]);
    }

    #[test]
    fn ecs_components_macro() {
        ecs_components! {
            components {
                coord: (i32, i32),
                name: String,
                health: i32,
            }
        }
        use components::Components;
        let mut entity_allocator = EntityAllocator::default();
        let mut components = Components::default();
        let e0 = entity_allocator.alloc();
        let e1 = entity_allocator.alloc();
        components.coord.insert(e0, (12, 19));
        components.name.insert(e0, "Foo".to_string());
        components.health.insert(e0, 42);
        components.coord.insert(e1, (0, 0));
        components.remove_entity(e1);
        entity_allocator.free(e1);
        assert!(!components.coord.contains(e1));
        let e0_data = components.remove_entity_data(e0);
        let e2 = entity_allocator.alloc();
        components.insert_entity_data(e2, e0_data);
        assert_eq!(components.name.get(e2).unwrap(), "Foo");
    }

    #[test]
    fn serde() {
        ecs_components! {
            components {
                coord: (i32, i32),
                name: String,
            }
        }
        use components::{Components, EntityData};
        let mut entity_allocator = EntityAllocator::default();
        let mut components = Components::default();
        let e0 = entity_allocator.alloc();
        components.insert_entity_data(
            e0,
            EntityData {
                coord: Some((21, 42)),
                name: Some("foo".to_string()),
            },
        );
        let e1 = entity_allocator.alloc();
        components.insert_entity_data(
            e1,
            EntityData {
                coord: None,
                name: Some("bar".to_string()),
            },
        );
        let e2 = entity_allocator.alloc();
        components.insert_entity_data(
            e2,
            EntityData {
                coord: Some((2, 3)),
                name: Some("baz".to_string()),
            },
        );
        entity_allocator.free(e1);
        components.remove_entity(e1);
        let e3 = entity_allocator.alloc();
        components.insert_entity_data(
            e3,
            EntityData {
                coord: Some((11, 12)),
                name: Some("qux".to_string()),
            },
        );
        entity_allocator.free(e0);
        components.remove_entity(e0);
        let json = serde_json::to_string(&components).unwrap();
        let deserialized: Components = serde_json::from_str(&json).unwrap();
        assert_eq!(components.coord.get(e2).unwrap(), deserialized.coord.get(e2).unwrap());
        assert_eq!(components.coord.get(e3).unwrap(), deserialized.coord.get(e3).unwrap());
        assert_eq!(components.name.get(e2).unwrap(), deserialized.name.get(e2).unwrap());
        assert_eq!(components.name.get(e3).unwrap(), deserialized.name.get(e3).unwrap());
        assert!(!entity_allocator.exists(e0));
        assert!(!entity_allocator.exists(e1));
    }
}
