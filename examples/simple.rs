//! Simple example with FPS counter, settings menu, and inventory

use grey_engine::prelude::*;
use winit::keyboard::KeyCode;

// ============================================================================
// Inventory Item
// ============================================================================

#[derive(Debug, Clone)]
struct Item {
    name: String,
    color: Color,
    count: u32,
}

impl Item {
    fn new(name: &str, color: Color) -> Self {
        Self { name: name.to_string(), color, count: 1 }
    }
}

// ============================================================================
// Game State
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
enum GameState {
    Playing,
    Settings,
    Inventory,
}

struct Settings {
    move_speed: f32,
    show_fps: bool,
    volume: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            move_speed: 200.0,
            show_fps: true,
            volume: 0.8,
        }
    }
}

// ============================================================================
// Main Game
// ============================================================================

struct SimpleGame {
    square: Entity,
    state: GameState,
    settings: Settings,
    inventory: Vec<Item>,
    selected_slot: usize,
    fps_samples: Vec<f32>,
    current_fps: f32,
}

impl Application for SimpleGame {
    fn init(engine: &mut Engine) -> Self {
        engine.clear_color = Color::new(0.08, 0.08, 0.12, 1.0);
        
        // Create player square
        let square = engine.world.spawn();
        engine.world.add(square, Transform2D::new(Vec2::ZERO));
        engine.world.add(square, Sprite::colored(Color::from_hex(0x6C5CE7), Vec2::new(60.0, 60.0)));
        
        // Starting inventory
        let inventory = vec![
            Item::new("Sword", Color::from_hex(0xE74C3C)),
            Item::new("Shield", Color::from_hex(0x3498DB)),
            Item::new("Potion", Color::from_hex(0x2ECC71)),
            Item::new("Key", Color::from_hex(0xF1C40F)),
        ];
        
        Self {
            square,
            state: GameState::Playing,
            settings: Settings::default(),
            inventory,
            selected_slot: 0,
            fps_samples: Vec::with_capacity(60),
            current_fps: 0.0,
        }
    }
    
    fn update(&mut self, engine: &mut Engine, dt: f32) {
        // Update FPS counter
        if dt > 0.0 {
            self.fps_samples.push(1.0 / dt);
            if self.fps_samples.len() > 60 {
                self.fps_samples.remove(0);
            }
            self.current_fps = self.fps_samples.iter().sum::<f32>() / self.fps_samples.len() as f32;
        }
        
        match self.state {
            GameState::Playing => self.update_playing(engine, dt),
            GameState::Settings => self.update_settings(engine),
            GameState::Inventory => self.update_inventory(engine),
        }
    }
    
    fn render(&mut self, engine: &Engine, renderer: &mut Renderer2D) {
        let vp = engine.camera.viewport_size();
        let half_w = vp.x / 2.0;
        let half_h = vp.y / 2.0;
        
        match self.state {
            GameState::Playing => {
                // FPS Counter (top right)
                if self.settings.show_fps {
                    self.draw_fps(renderer, half_w, half_h);
                }
                
                // Controls hint (bottom left)
                self.draw_text_bar(renderer, Vec2::new(-half_w + 10.0, -half_h + 30.0), 
                    "[I] Inventory  [ESC] Settings", Color::GRAY.with_alpha(0.5));
            }
            GameState::Settings => {
                self.draw_settings_menu(renderer, half_w, half_h);
            }
            GameState::Inventory => {
                self.draw_inventory(renderer, half_w, half_h);
            }
        }
    }
    
    fn on_key_pressed(&mut self, engine: &mut Engine, key: KeyCode) {
        match self.state {
            GameState::Playing => {
                match key {
                    KeyCode::Escape => self.state = GameState::Settings,
                    KeyCode::KeyI | KeyCode::Tab => self.state = GameState::Inventory,
                    KeyCode::KeyE => {
                        // Pick up random item
                        let colors = [0xE91E63, 0x9C27B0, 0x00BCD4, 0xFF9800];
                        let names = ["Gem", "Crystal", "Orb", "Rune"];
                        let idx = (engine.time.frame_count() % 4) as usize;
                        self.inventory.push(Item::new(names[idx], Color::from_hex(colors[idx])));
                    }
                    _ => {}
                }
            }
            GameState::Settings => {
                match key {
                    KeyCode::Escape => self.state = GameState::Playing,
                    KeyCode::ArrowUp | KeyCode::KeyW => {
                        self.settings.move_speed = (self.settings.move_speed + 50.0).min(500.0);
                    }
                    KeyCode::ArrowDown | KeyCode::KeyS => {
                        self.settings.move_speed = (self.settings.move_speed - 50.0).max(50.0);
                    }
                    KeyCode::KeyF => {
                        self.settings.show_fps = !self.settings.show_fps;
                    }
                    KeyCode::ArrowLeft | KeyCode::KeyA => {
                        self.settings.volume = (self.settings.volume - 0.1).max(0.0);
                    }
                    KeyCode::ArrowRight | KeyCode::KeyD => {
                        self.settings.volume = (self.settings.volume + 0.1).min(1.0);
                    }
                    _ => {}
                }
            }
            GameState::Inventory => {
                match key {
                    KeyCode::Escape | KeyCode::KeyI | KeyCode::Tab => self.state = GameState::Playing,
                    KeyCode::ArrowLeft | KeyCode::KeyA => {
                        if self.selected_slot > 0 {
                            self.selected_slot -= 1;
                        }
                    }
                    KeyCode::ArrowRight | KeyCode::KeyD => {
                        if self.selected_slot < self.inventory.len().saturating_sub(1) {
                            self.selected_slot += 1;
                        }
                    }
                    KeyCode::KeyX | KeyCode::Delete => {
                        // Drop item
                        if !self.inventory.is_empty() {
                            self.inventory.remove(self.selected_slot);
                            if self.selected_slot > 0 && self.selected_slot >= self.inventory.len() {
                                self.selected_slot = self.inventory.len().saturating_sub(1);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

impl SimpleGame {
    fn update_playing(&mut self, engine: &mut Engine, dt: f32) {
        let movement = engine.input.get_movement_input();
        
        if let Some(transform) = engine.world.get_mut::<Transform2D>(self.square) {
            transform.position += movement * self.settings.move_speed * dt;
            transform.rotation += dt * 0.5;
        }
    }
    
    fn update_settings(&mut self, _engine: &mut Engine) {
        // Settings controls handled in on_key_pressed
    }
    
    fn update_inventory(&mut self, _engine: &mut Engine) {
        // Inventory controls handled in on_key_pressed
    }
    
    // ========================================================================
    // Drawing helpers
    // ========================================================================
    
    fn draw_fps(&self, renderer: &mut Renderer2D, half_w: f32, half_h: f32) {
        let fps = self.current_fps as i32;
        let color = if fps >= 55 {
            Color::from_hex(0x2ECC71) // Green
        } else if fps >= 30 {
            Color::from_hex(0xF1C40F) // Yellow  
        } else {
            Color::from_hex(0xE74C3C) // Red
        };
        
        // Background
        renderer.draw_quad(
            Vec2::new(half_w - 55.0, half_h - 22.0),
            Vec2::new(100.0, 36.0),
            0.0,
            Color::BLACK.with_alpha(0.7),
        );
        
        // "FPS:" label
        renderer.draw_text(
            Vec2::new(half_w - 100.0, half_h - 22.0),
            "FPS:",
            1.0,
            Color::WHITE.with_alpha(0.7),
        );
        
        // FPS number
        renderer.draw_number(
            Vec2::new(half_w - 40.0, half_h - 22.0),
            fps,
            1.2,
            color,
        );
    }
    
    fn draw_settings_menu(&self, renderer: &mut Renderer2D, half_w: f32, half_h: f32) {
        // Dim background
        renderer.draw_quad(Vec2::ZERO, Vec2::new(half_w * 2.0, half_h * 2.0), 0.0, Color::BLACK.with_alpha(0.8));
        
        // Menu panel
        let panel_w = 400.0;
        let panel_h = 300.0;
        renderer.draw_quad(Vec2::ZERO, Vec2::new(panel_w, panel_h), 0.0, Color::from_hex(0x2C3E50));
        renderer.draw_quad(Vec2::ZERO, Vec2::new(panel_w - 4.0, panel_h - 4.0), 0.0, Color::from_hex(0x34495E));
        
        // Title bar
        renderer.draw_quad(Vec2::new(0.0, panel_h / 2.0 - 25.0), Vec2::new(panel_w - 4.0, 40.0), 0.0, Color::from_hex(0x6C5CE7));
        
        // Speed setting (visual bar)
        let y = 50.0;
        renderer.draw_quad(Vec2::new(-100.0, y), Vec2::new(150.0, 20.0), 0.0, Color::from_hex(0x1A1A2E));
        let speed_pct = (self.settings.move_speed - 50.0) / 450.0;
        renderer.draw_quad(
            Vec2::new(-100.0 - 75.0 + speed_pct * 75.0, y),
            Vec2::new(150.0 * speed_pct, 16.0),
            0.0,
            Color::from_hex(0x00CEC9),
        );
        // Label indicator
        self.draw_text_bar(renderer, Vec2::new(-170.0, y), "Speed [W/S]", Color::WHITE);
        
        // FPS toggle
        let y = 0.0;
        let fps_color = if self.settings.show_fps { Color::from_hex(0x2ECC71) } else { Color::from_hex(0xE74C3C) };
        renderer.draw_quad(Vec2::new(50.0, y), Vec2::new(60.0, 30.0), 0.0, fps_color);
        self.draw_text_bar(renderer, Vec2::new(-170.0, y), "Show FPS [F]", Color::WHITE);
        
        // Volume slider
        let y = -50.0;
        renderer.draw_quad(Vec2::new(-100.0, y), Vec2::new(150.0, 20.0), 0.0, Color::from_hex(0x1A1A2E));
        renderer.draw_quad(
            Vec2::new(-100.0 - 75.0 + self.settings.volume * 75.0, y),
            Vec2::new(150.0 * self.settings.volume, 16.0),
            0.0,
            Color::from_hex(0xFDCB6E),
        );
        self.draw_text_bar(renderer, Vec2::new(-170.0, y), "Volume [A/D]", Color::WHITE);
        
        // Close hint
        self.draw_text_bar(renderer, Vec2::new(0.0, -panel_h / 2.0 + 30.0), "[ESC] Close", Color::GRAY);
    }
    
    fn draw_inventory(&self, renderer: &mut Renderer2D, half_w: f32, half_h: f32) {
        // Dim background
        renderer.draw_quad(Vec2::ZERO, Vec2::new(half_w * 2.0, half_h * 2.0), 0.0, Color::BLACK.with_alpha(0.85));
        
        // Inventory panel
        let panel_w = 500.0;
        let panel_h = 200.0;
        renderer.draw_quad(Vec2::ZERO, Vec2::new(panel_w, panel_h), 0.0, Color::from_hex(0x2C3E50));
        renderer.draw_quad(Vec2::ZERO, Vec2::new(panel_w - 4.0, panel_h - 4.0), 0.0, Color::from_hex(0x1A1A2E));
        
        // Title
        renderer.draw_quad(Vec2::new(0.0, panel_h / 2.0 - 25.0), Vec2::new(panel_w - 4.0, 40.0), 0.0, Color::from_hex(0xE17055));
        
        // Inventory slots
        let slot_size = 60.0;
        let slot_spacing = 70.0;
        let max_visible = 6;
        let start_x = -(max_visible as f32 * slot_spacing) / 2.0 + slot_spacing / 2.0;
        
        for i in 0..max_visible {
            let x = start_x + i as f32 * slot_spacing;
            let y = 0.0;
            
            // Slot background
            let is_selected = i == self.selected_slot;
            let slot_color = if is_selected {
                Color::from_hex(0x6C5CE7)
            } else {
                Color::from_hex(0x34495E)
            };
            
            renderer.draw_quad(Vec2::new(x, y), Vec2::new(slot_size + 4.0, slot_size + 4.0), 0.0, slot_color);
            renderer.draw_quad(Vec2::new(x, y), Vec2::new(slot_size, slot_size), 0.0, Color::from_hex(0x2C3E50));
            
            // Item in slot
            if let Some(item) = self.inventory.get(i) {
                let item_size = slot_size - 16.0;
                renderer.draw_quad(Vec2::new(x, y), Vec2::new(item_size, item_size), 0.0, item.color);
                
                // Stack count indicator
                if item.count > 1 {
                    renderer.draw_quad(
                        Vec2::new(x + slot_size / 2.0 - 8.0, y - slot_size / 2.0 + 8.0),
                        Vec2::new(16.0, 16.0),
                        0.0,
                        Color::BLACK.with_alpha(0.7),
                    );
                }
            }
        }
        
        // Selected item name indicator
        if let Some(item) = self.inventory.get(self.selected_slot) {
            self.draw_text_bar(renderer, Vec2::new(0.0, -60.0), &item.name, item.color);
        }
        
        // Controls
        self.draw_text_bar(renderer, Vec2::new(0.0, -panel_h / 2.0 + 20.0), 
            "[A/D] Select  [X] Drop  [ESC] Close", Color::GRAY);
    }
    
    fn draw_text_bar(&self, renderer: &mut Renderer2D, pos: Vec2, _text: &str, color: Color) {
        // Visual indicator bar (since we can't render text directly)
        renderer.draw_quad(pos, Vec2::new(8.0, 8.0), 0.0, color);
    }
}

fn main() -> anyhow::Result<()> {
    grey_engine::run::<SimpleGame>()
}
