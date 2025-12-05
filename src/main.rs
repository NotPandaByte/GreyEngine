//! Example game using GreyEngine

use grey_engine::prelude::*;
use winit::keyboard::KeyCode;

struct Game {
    player: Entity,
    enemies: Vec<Entity>,
    score: u32,
    spawn_timer: f32,
}

impl Application for Game {
    fn init(engine: &mut Engine) -> Self {
        // Set a nice dark background
        engine.clear_color = Color::new(0.05, 0.05, 0.1, 1.0);
        
        // Create player
        let player = engine.world.spawn();
        engine.world.add(player, Transform2D::new(Vec2::ZERO));
        engine.world.add(player, Sprite::colored(Color::from_hex(0x4ECDC4), Vec2::new(40.0, 40.0)));
        engine.world.add(player, Name::new("Player"));
        
        // Create some initial enemies
        let mut enemies = Vec::new();
        for i in 0..5 {
            let enemy = engine.world.spawn();
            let angle = (i as f32 / 5.0) * std::f32::consts::TAU;
            let pos = Vec2::new(angle.cos() * 300.0, angle.sin() * 300.0);
            
            engine.world.add(enemy, Transform2D::new(pos));
            engine.world.add(enemy, Sprite::colored(Color::from_hex(0xFF6B6B), Vec2::new(30.0, 30.0)));
            engine.world.add(enemy, Velocity2D {
                linear: Vec2::ZERO,
                angular: 2.0,
            });
            
            enemies.push(enemy);
        }
        
        Self {
            player,
            enemies,
            score: 0,
            spawn_timer: 0.0,
        }
    }
    
    fn update(&mut self, engine: &mut Engine, dt: f32) {
        let speed = 300.0;
        
        // Player movement
        let movement = engine.input.get_movement_input();
        if let Some(transform) = engine.world.get_mut::<Transform2D>(self.player) {
            transform.position += movement * speed * dt;
            
            // Clamp to screen bounds
            let half_size = engine.camera.viewport_size() * 0.5;
            transform.position.x = transform.position.x.clamp(-half_size.x + 20.0, half_size.x - 20.0);
            transform.position.y = transform.position.y.clamp(-half_size.y + 20.0, half_size.y - 20.0);
        }
        
        // Get player position for enemy AI
        let player_pos = engine.world.get::<Transform2D>(self.player)
            .map(|t| t.position)
            .unwrap_or(Vec2::ZERO);
        
        // Update enemies - chase player
        for &enemy in &self.enemies {
            if let Some(transform) = engine.world.get_mut::<Transform2D>(enemy) {
                let dir = (player_pos - transform.position).normalize();
                transform.position += dir * 100.0 * dt;
                transform.rotation += 2.0 * dt;
            }
        }
        
        // Spawn new enemies periodically
        self.spawn_timer += dt;
        if self.spawn_timer > 2.0 {
            self.spawn_timer = 0.0;
            
            let angle = (self.enemies.len() as f32 * 1.5) % std::f32::consts::TAU;
            let spawn_dist = 400.0;
            let pos = Vec2::new(angle.cos() * spawn_dist, angle.sin() * spawn_dist);
            
            let enemy = engine.world.spawn();
            engine.world.add(enemy, Transform2D::new(pos));
            engine.world.add(enemy, Sprite::colored(
                Color::from_hex(0xFF6B6B),
                Vec2::new(30.0, 30.0)
            ));
            self.enemies.push(enemy);
        }
        
        // Check collisions with player
        let player_size = 40.0;
        let enemy_size = 30.0;
        let collision_dist = (player_size + enemy_size) / 2.0;
        
        self.enemies.retain(|&enemy| {
            if let Some(enemy_transform) = engine.world.get::<Transform2D>(enemy) {
                let dist = (enemy_transform.position - player_pos).length();
                if dist < collision_dist {
                    engine.world.despawn(enemy);
                    self.score += 10;
                    false
                } else {
                    true
                }
            } else {
                false
            }
        });
    }
    
    fn render(&mut self, engine: &Engine, renderer: &mut Renderer2D) {
        // Draw a simple HUD - score indicator as colored bars
        let bar_count = (self.score / 10).min(20);
        for i in 0..bar_count as usize {
            let x = -engine.camera.viewport_size().x / 2.0 + 20.0 + (i as f32 * 15.0);
            let y = engine.camera.viewport_size().y / 2.0 - 20.0;
            renderer.draw_quad(
                Vec2::new(x, y),
                Vec2::new(10.0, 20.0),
                0.0,
                Color::from_hex(0xFFE66D),
            );
        }
    }
    
    fn on_key_pressed(&mut self, engine: &mut Engine, key: KeyCode) {
        if key == KeyCode::Space {
            // Boost player size temporarily
            if let Some(sprite) = engine.world.get_mut::<Sprite>(self.player) {
                sprite.size = Vec2::new(60.0, 60.0);
            }
        }
    }
    
    fn on_key_released(&mut self, engine: &mut Engine, key: KeyCode) {
        if key == KeyCode::Space {
            // Reset player size
            if let Some(sprite) = engine.world.get_mut::<Sprite>(self.player) {
                sprite.size = Vec2::new(40.0, 40.0);
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    grey_engine::run::<Game>()
}
