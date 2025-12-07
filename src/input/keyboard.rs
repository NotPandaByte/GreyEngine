use winit::keyboard::KeyCode;
use std::collections::HashSet;

pub struct Keyboard {
    pressed_keys: HashSet<KeyCode>,
    keys_just_pressed: HashSet<KeyCode>,
    keys_just_released: HashSet<KeyCode>,
}

impl Keyboard {
    pub fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            keys_just_pressed: HashSet::new(),
            keys_just_released: HashSet::new(),
        }
    }

    pub fn handle_key_event(&mut self, key: KeyCode, is_pressed: bool) {
        if is_pressed {
            if !self.pressed_keys.contains(&key) {
                self.keys_just_pressed.insert(key);
                self.pressed_keys.insert(key);
            }
        } else {
            if self.pressed_keys.contains(&key) {
                self.keys_just_released.insert(key);
                self.pressed_keys.remove(&key);
            }
        }
    }

    pub fn is_pressed(&self, key: KeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }

    pub fn was_just_pressed(&self, key: KeyCode) -> bool {
        self.keys_just_pressed.contains(&key)
    }

    pub fn was_just_released(&self, key: KeyCode) -> bool {
        self.keys_just_released.contains(&key)
    }

    pub fn clear_just_pressed(&mut self) {
        self.keys_just_pressed.clear();
    }

    pub fn clear_just_released(&mut self) {
        self.keys_just_released.clear();
    }

    pub fn clear_frame_state(&mut self) {
        self.keys_just_pressed.clear();
        self.keys_just_released.clear();
    }
}