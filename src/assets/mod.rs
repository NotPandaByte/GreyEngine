//! Asset management and resource loading.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Handle to a loaded asset
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AssetHandle(pub u32);

/// Asset loading state
#[derive(Debug, Clone, PartialEq)]
pub enum AssetState {
    Loading,
    Loaded,
    Failed(String),
}

/// Asset manager for loading and caching resources
pub struct AssetManager {
    next_handle: u32,
    paths: HashMap<AssetHandle, PathBuf>,
    states: HashMap<AssetHandle, AssetState>,
    bytes: HashMap<AssetHandle, Vec<u8>>,
}

impl Default for AssetManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetManager {
    pub fn new() -> Self {
        Self {
            next_handle: 0,
            paths: HashMap::new(),
            states: HashMap::new(),
            bytes: HashMap::new(),
        }
    }

    /// Load an asset from a file path
    pub fn load(&mut self, path: impl AsRef<Path>) -> AssetHandle {
        let path = path.as_ref().to_path_buf();
        
        // Check if already loaded
        for (handle, existing_path) in &self.paths {
            if *existing_path == path {
                return *handle;
            }
        }

        let handle = AssetHandle(self.next_handle);
        self.next_handle += 1;
        
        self.paths.insert(handle, path.clone());
        self.states.insert(handle, AssetState::Loading);

        // Try to load immediately (blocking)
        match std::fs::read(&path) {
            Ok(data) => {
                self.bytes.insert(handle, data);
                self.states.insert(handle, AssetState::Loaded);
            }
            Err(e) => {
                self.states.insert(handle, AssetState::Failed(e.to_string()));
            }
        }

        handle
    }

    /// Get the state of an asset
    pub fn state(&self, handle: AssetHandle) -> Option<&AssetState> {
        self.states.get(&handle)
    }

    /// Get the bytes of a loaded asset
    pub fn get_bytes(&self, handle: AssetHandle) -> Option<&[u8]> {
        if self.states.get(&handle) == Some(&AssetState::Loaded) {
            self.bytes.get(&handle).map(|v| v.as_slice())
        } else {
            None
        }
    }

    /// Check if asset is loaded
    pub fn is_loaded(&self, handle: AssetHandle) -> bool {
        self.states.get(&handle) == Some(&AssetState::Loaded)
    }

    /// Unload an asset
    pub fn unload(&mut self, handle: AssetHandle) {
        self.paths.remove(&handle);
        self.states.remove(&handle);
        self.bytes.remove(&handle);
    }

    /// Clear all assets
    pub fn clear(&mut self) {
        self.paths.clear();
        self.states.clear();
        self.bytes.clear();
    }
}
