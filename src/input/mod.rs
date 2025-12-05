//! Input handling module.

use std::collections::HashSet;
use winit::keyboard::KeyCode;
use crate::math::Vec2;

/// Mouse button identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Input state tracking for keyboard and mouse
#[derive(Debug, Default)]
pub struct Input {
    // Keyboard
    keys_down: HashSet<KeyCode>,
    keys_pressed: HashSet<KeyCode>,
    keys_released: HashSet<KeyCode>,
    
    // Mouse
    mouse_buttons_down: HashSet<MouseButton>,
    mouse_buttons_pressed: HashSet<MouseButton>,
    mouse_buttons_released: HashSet<MouseButton>,
    mouse_position: Vec2,
    mouse_delta: Vec2,
    scroll_delta: Vec2,
    
    // Internal tracking
    prev_mouse_position: Vec2,
}

impl Input {
    pub fn new() -> Self {
        Self::default()
    }

    /// Call at the start of each frame to update state
    pub fn begin_frame(&mut self) {
        self.keys_pressed.clear();
        self.keys_released.clear();
        self.mouse_buttons_pressed.clear();
        self.mouse_buttons_released.clear();
        self.scroll_delta = Vec2::ZERO;
        self.mouse_delta = self.mouse_position - self.prev_mouse_position;
        self.prev_mouse_position = self.mouse_position;
    }

    // ========================================================================
    // Keyboard
    // ========================================================================

    pub fn on_key_pressed(&mut self, key: KeyCode) {
        if !self.keys_down.contains(&key) {
            self.keys_pressed.insert(key);
        }
        self.keys_down.insert(key);
    }

    pub fn on_key_released(&mut self, key: KeyCode) {
        self.keys_down.remove(&key);
        self.keys_released.insert(key);
    }

    /// Returns true if the key is currently held down
    pub fn key_down(&self, key: KeyCode) -> bool {
        self.keys_down.contains(&key)
    }

    /// Returns true only on the frame the key was first pressed
    pub fn key_pressed(&self, key: KeyCode) -> bool {
        self.keys_pressed.contains(&key)
    }

    /// Returns true only on the frame the key was released
    pub fn key_released(&self, key: KeyCode) -> bool {
        self.keys_released.contains(&key)
    }

    /// Get movement input from WASD or arrow keys as a normalized vector
    pub fn get_movement_input(&self) -> Vec2 {
        let mut dir = Vec2::ZERO;
        
        if self.key_down(KeyCode::KeyW) || self.key_down(KeyCode::ArrowUp) {
            dir.y += 1.0;
        }
        if self.key_down(KeyCode::KeyS) || self.key_down(KeyCode::ArrowDown) {
            dir.y -= 1.0;
        }
        if self.key_down(KeyCode::KeyA) || self.key_down(KeyCode::ArrowLeft) {
            dir.x -= 1.0;
        }
        if self.key_down(KeyCode::KeyD) || self.key_down(KeyCode::ArrowRight) {
            dir.x += 1.0;
        }
        
        if dir.length_squared() > 0.0 {
            dir = dir.normalize();
        }
        
        dir
    }

    // ========================================================================
    // Mouse
    // ========================================================================

    pub fn on_mouse_button_pressed(&mut self, button: MouseButton) {
        if !self.mouse_buttons_down.contains(&button) {
            self.mouse_buttons_pressed.insert(button);
        }
        self.mouse_buttons_down.insert(button);
    }

    pub fn on_mouse_button_released(&mut self, button: MouseButton) {
        self.mouse_buttons_down.remove(&button);
        self.mouse_buttons_released.insert(button);
    }

    pub fn on_mouse_moved(&mut self, x: f32, y: f32) {
        self.mouse_position = Vec2::new(x, y);
    }

    pub fn on_scroll(&mut self, x: f32, y: f32) {
        self.scroll_delta = Vec2::new(x, y);
    }

    /// Returns true if the mouse button is currently held down
    pub fn mouse_button_down(&self, button: MouseButton) -> bool {
        self.mouse_buttons_down.contains(&button)
    }

    /// Returns true only on the frame the mouse button was first pressed
    pub fn mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons_pressed.contains(&button)
    }

    /// Returns true only on the frame the mouse button was released
    pub fn mouse_button_released(&self, button: MouseButton) -> bool {
        self.mouse_buttons_released.contains(&button)
    }

    /// Current mouse position in screen coordinates
    pub fn mouse_position(&self) -> Vec2 {
        self.mouse_position
    }

    /// Mouse movement since last frame
    pub fn mouse_delta(&self) -> Vec2 {
        self.mouse_delta
    }

    /// Scroll wheel delta this frame
    pub fn scroll_delta(&self) -> Vec2 {
        self.scroll_delta
    }
}
