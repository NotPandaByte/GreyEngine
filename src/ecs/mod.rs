//! Entity-Component-System (ECS) module.

use std::any::{Any, TypeId};
use std::collections::HashMap;

// ============================================================================
// Entity
// ============================================================================

/// Unique identifier for an entity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(pub u32);

impl Entity {
    pub fn id(&self) -> u32 {
        self.0
    }
}

// ============================================================================
// Component Storage
// ============================================================================

/// Trait for component storage operations
trait ComponentStorage: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn remove(&mut self, entity: Entity);
    fn has(&self, entity: Entity) -> bool;
}

/// Stores components of type T mapped to entities
struct Storage<T: 'static> {
    data: HashMap<Entity, T>,
}

impl<T: 'static> Storage<T> {
    fn new() -> Self {
        Self { data: HashMap::new() }
    }

    fn insert(&mut self, entity: Entity, component: T) {
        self.data.insert(entity, component);
    }

    fn get(&self, entity: Entity) -> Option<&T> {
        self.data.get(&entity)
    }

    fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        self.data.get_mut(&entity)
    }

    #[allow(dead_code)]
    fn iter(&self) -> impl Iterator<Item = (Entity, &T)> {
        self.data.iter().map(|(&e, c)| (e, c))
    }

    #[allow(dead_code)]
    fn iter_mut(&mut self) -> impl Iterator<Item = (Entity, &mut T)> {
        self.data.iter_mut().map(|(&e, c)| (e, c))
    }
}

impl<T: 'static> ComponentStorage for Storage<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn remove(&mut self, entity: Entity) {
        self.data.remove(&entity);
    }

    fn has(&self, entity: Entity) -> bool {
        self.data.contains_key(&entity)
    }
}

// ============================================================================
// World
// ============================================================================

/// The ECS world that holds all entities and components
pub struct World {
    next_entity_id: u32,
    entities: Vec<Entity>,
    dead_entities: Vec<Entity>,
    components: HashMap<TypeId, Box<dyn ComponentStorage>>,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    pub fn new() -> Self {
        Self {
            next_entity_id: 0,
            entities: Vec::new(),
            dead_entities: Vec::new(),
            components: HashMap::new(),
        }
    }

    /// Spawn a new entity
    pub fn spawn(&mut self) -> Entity {
        let entity = if let Some(recycled) = self.dead_entities.pop() {
            recycled
        } else {
            let id = self.next_entity_id;
            self.next_entity_id += 1;
            Entity(id)
        };
        self.entities.push(entity);
        entity
    }

    /// Despawn an entity and remove all its components
    pub fn despawn(&mut self, entity: Entity) {
        if let Some(pos) = self.entities.iter().position(|&e| e == entity) {
            self.entities.swap_remove(pos);
            self.dead_entities.push(entity);
            
            // Remove from all component storages
            for storage in self.components.values_mut() {
                storage.remove(entity);
            }
        }
    }

    /// Check if entity exists
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.entities.contains(&entity)
    }

    /// Get all entities
    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    /// Add a component to an entity
    pub fn add<T: 'static>(&mut self, entity: Entity, component: T) {
        let type_id = TypeId::of::<T>();
        
        if !self.components.contains_key(&type_id) {
            self.components.insert(type_id, Box::new(Storage::<T>::new()));
        }
        
        let storage = self.components
            .get_mut(&type_id)
            .unwrap()
            .as_any_mut()
            .downcast_mut::<Storage<T>>()
            .unwrap();
        
        storage.insert(entity, component);
    }

    /// Get a component from an entity
    pub fn get<T: 'static>(&self, entity: Entity) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.components
            .get(&type_id)?
            .as_any()
            .downcast_ref::<Storage<T>>()?
            .get(entity)
    }

    /// Get a mutable component from an entity
    pub fn get_mut<T: 'static>(&mut self, entity: Entity) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.components
            .get_mut(&type_id)?
            .as_any_mut()
            .downcast_mut::<Storage<T>>()?
            .get_mut(entity)
    }

    /// Check if entity has a component
    pub fn has<T: 'static>(&self, entity: Entity) -> bool {
        let type_id = TypeId::of::<T>();
        self.components
            .get(&type_id)
            .map(|s| s.has(entity))
            .unwrap_or(false)
    }

    /// Remove a component from an entity
    pub fn remove<T: 'static>(&mut self, entity: Entity) {
        let type_id = TypeId::of::<T>();
        if let Some(storage) = self.components.get_mut(&type_id) {
            storage.remove(entity);
        }
    }

    /// Query all entities with component T
    pub fn query<T: 'static>(&self) -> QueryIter<'_, T> {
        QueryIter {
            inner: self.components
                .get(&TypeId::of::<T>())
                .and_then(|s| s.as_any().downcast_ref::<Storage<T>>())
                .map(|s| s.data.iter()),
        }
    }

    /// Query all entities with component T (mutable)
    pub fn query_mut<T: 'static>(&mut self) -> QueryIterMut<'_, T> {
        QueryIterMut {
            inner: self.components
                .get_mut(&TypeId::of::<T>())
                .and_then(|s| s.as_any_mut().downcast_mut::<Storage<T>>())
                .map(|s| s.data.iter_mut()),
        }
    }
}

// ============================================================================
// Query Iterators
// ============================================================================

pub struct QueryIter<'a, T: 'static> {
    inner: Option<std::collections::hash_map::Iter<'a, Entity, T>>,
}

impl<'a, T: 'static> Iterator for QueryIter<'a, T> {
    type Item = (Entity, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.as_mut()?.next().map(|(&e, c)| (e, c))
    }
}

pub struct QueryIterMut<'a, T: 'static> {
    inner: Option<std::collections::hash_map::IterMut<'a, Entity, T>>,
}

impl<'a, T: 'static> Iterator for QueryIterMut<'a, T> {
    type Item = (Entity, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.as_mut()?.next().map(|(&e, c)| (e, c))
    }
}

// ============================================================================
// Common Components
// ============================================================================

use crate::math::{Vec2, Vec3, Color};

/// 2D Transform component
#[derive(Debug, Clone, Copy)]
pub struct Transform2D {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
}

impl Default for Transform2D {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }
}

impl Transform2D {
    pub fn new(position: Vec2) -> Self {
        Self { position, ..Default::default() }
    }

    pub fn with_scale(mut self, scale: Vec2) -> Self {
        self.scale = scale;
        self
    }

    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }
}

/// 3D Transform component
#[derive(Debug, Clone, Copy)]
pub struct Transform3D {
    pub position: Vec3,
    pub rotation: Vec3, // Euler angles
    pub scale: Vec3,
}

impl Default for Transform3D {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
        }
    }
}

/// Sprite component for 2D rendering
#[derive(Debug, Clone)]
pub struct Sprite {
    pub color: Color,
    pub size: Vec2,
    pub texture_id: Option<u32>,
    pub uv_rect: [f32; 4], // x, y, w, h in UV coords
}

impl Default for Sprite {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            size: Vec2::new(100.0, 100.0),
            texture_id: None,
            uv_rect: [0.0, 0.0, 1.0, 1.0],
        }
    }
}

impl Sprite {
    pub fn colored(color: Color, size: Vec2) -> Self {
        Self { color, size, ..Default::default() }
    }
}

/// Velocity component for movement
#[derive(Debug, Clone, Copy, Default)]
pub struct Velocity2D {
    pub linear: Vec2,
    pub angular: f32,
}

/// Tag component for naming entities
#[derive(Debug, Clone)]
pub struct Name(pub String);

impl Name {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}
