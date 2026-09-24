// Scene-stage component: the single owner of every `Material`. Blocks store
// only a `MaterialId`; the renderer resolves it here per hit.
#![allow(dead_code)]

use std::collections::HashMap;

use crate::core::material::{Material, MaterialId};

/// `MaterialId -> Material` registry. Centralizing materials here keeps
/// per-block memory to one small id, lets several blocks share a material,
/// and allows optical tuning in a single place.
#[derive(Debug, Default)]
pub struct MaterialLibrary {
    materials: HashMap<MaterialId, Material>,
}

impl MaterialLibrary {
    pub fn new() -> Self {
        Self {
            materials: HashMap::new(),
        }
    }

    /// Registers `material` under `id`, returning the previous material for
    /// that id if one existed.
    pub fn insert(&mut self, id: MaterialId, material: Material) -> Option<Material> {
        self.materials.insert(id, material)
    }

    /// Safe lookup: an unknown id is `None`, never a panic.
    pub fn get(&self, id: MaterialId) -> Option<&Material> {
        self.materials.get(&id)
    }

    pub fn contains(&self, id: MaterialId) -> bool {
        self.materials.contains_key(&id)
    }

    pub fn len(&self) -> usize {
        self.materials.len()
    }

    pub fn is_empty(&self) -> bool {
        self.materials.is_empty()
    }
}
