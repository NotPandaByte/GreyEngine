//! Scene and hierarchy management.

use crate::ecs::{Entity, World};
use crate::math::Vec2;

/// A node in the scene hierarchy
#[derive(Debug, Clone)]
pub struct SceneNode {
    pub entity: Entity,
    pub parent: Option<Entity>,
    pub children: Vec<Entity>,
    pub local_position: Vec2,
}

impl SceneNode {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            parent: None,
            children: Vec::new(),
            local_position: Vec2::ZERO,
        }
    }
}

/// Scene graph for managing entity hierarchies
#[derive(Debug, Default)]
pub struct SceneGraph {
    nodes: std::collections::HashMap<Entity, SceneNode>,
    roots: Vec<Entity>,
}

impl SceneGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an entity as a root node
    pub fn add_root(&mut self, entity: Entity) {
        self.nodes.insert(entity, SceneNode::new(entity));
        self.roots.push(entity);
    }

    /// Set parent-child relationship
    pub fn set_parent(&mut self, child: Entity, parent: Entity) {
        // Remove from old parent or roots
        if let Some(node) = self.nodes.get(&child) {
            if let Some(old_parent) = node.parent {
                if let Some(parent_node) = self.nodes.get_mut(&old_parent) {
                    parent_node.children.retain(|&e| e != child);
                }
            } else {
                self.roots.retain(|&e| e != child);
            }
        }

        // Add to new parent
        if let Some(parent_node) = self.nodes.get_mut(&parent) {
            parent_node.children.push(child);
        }

        if let Some(child_node) = self.nodes.get_mut(&child) {
            child_node.parent = Some(parent);
        }
    }

    /// Get root entities
    pub fn roots(&self) -> &[Entity] {
        &self.roots
    }

    /// Get children of an entity
    pub fn children(&self, entity: Entity) -> &[Entity] {
        self.nodes.get(&entity).map(|n| n.children.as_slice()).unwrap_or(&[])
    }

    /// Remove an entity and all its children
    pub fn remove(&mut self, entity: Entity) {
        if let Some(node) = self.nodes.remove(&entity) {
            // Remove from parent's children
            if let Some(parent) = node.parent {
                if let Some(parent_node) = self.nodes.get_mut(&parent) {
                    parent_node.children.retain(|&e| e != entity);
                }
            } else {
                self.roots.retain(|&e| e != entity);
            }

            // Recursively remove children
            for child in node.children {
                self.remove(child);
            }
        }
    }
}
