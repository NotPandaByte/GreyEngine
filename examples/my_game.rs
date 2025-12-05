//! Your game goes here

use grey_engine::prelude::*;
use winit::keyboard::KeyCode;

// Define your game struct - holds all your game state
struct MyGame {
    player: Entity,
    bullets: Vec<Entity>,
    shoot_cooldown: f32,
}

impl Application for MyGame {
    // Called once at startup - create your entities here
    fn init(engine: &mut Engine) -> Self {
        engine.clear_color = Color::new(0.02, 0.02, 0.05, 1.0);
        
        // Spawn player
        let player = engine.world.spawn();
        engine.world.add(player, Transform2D::new(Vec2::new(0.0, -250.0)));
        engine.world.add(player, Sprite::colored(Color::from_hex(0x00FFAA), Vec2::new(50.0, 30.0)));
        engine.world.add(player, Name::new("Player"));
        
        Self {
            player,
            bullets: Vec::new(),
            shoot_cooldown: 0.0,
        }
    }
    
    // Called every frame - update your game logic here
    fn update(&mut self, engine: &mut Engine, dt: f32) {
        // Decrease cooldown
        self.shoot_cooldown -= dt;
        
        // Player movement (left/right only)
        let mut move_x = 0.0;
        if engine.input.key_down(KeyCode::KeyA) || engine.input.key_down(KeyCode::ArrowLeft) {
            move_x -= 1.0;
        }
        if engine.input.key_down(KeyCode::KeyD) || engine.input.key_down(KeyCode::ArrowRight) {
            move_x += 1.0;
        }
        
        if let Some(transform) = engine.world.get_mut::<Transform2D>(self.player) {
            transform.position.x += move_x * 400.0 * dt;
            transform.position.x = transform.position.x.clamp(-600.0, 600.0);
        }
        
        // Shooting
        if engine.input.key_down(KeyCode::Space) && self.shoot_cooldown <= 0.0 {
            self.shoot_cooldown = 0.15; // Fire rate
            
            let player_pos = engine.world.get::<Transform2D>(self.player)
                .map(|t| t.position)
                .unwrap_or(Vec2::ZERO);
            
            let bullet = engine.world.spawn();
            engine.world.add(bullet, Transform2D::new(player_pos + Vec2::new(0.0, 20.0)));
            engine.world.add(bullet, Sprite::colored(Color::YELLOW, Vec2::new(8.0, 20.0)));
            engine.world.add(bullet, Velocity2D { linear: Vec2::new(0.0, 500.0), angular: 0.0 });
            self.bullets.push(bullet);
        }
        
        // Move bullets
        self.bullets.retain(|&bullet| {
            if let Some(transform) = engine.world.get_mut::<Transform2D>(bullet) {
                if let Some(vel) = engine.world.get::<Velocity2D>(bullet) {
                    transform.position += vel.linear * dt;
                }
                
                // Remove if off screen
                if transform.position.y > 400.0 {
                    engine.world.despawn(bullet);
                    return false;
                }
            }
            true
        });
    }
    
    // Optional: Custom rendering (HUD, effects, etc)
    fn render(&mut self, engine: &Engine, renderer: &mut Renderer2D) {
        // Draw crosshair at player position
        if let Some(transform) = engine.world.get::<Transform2D>(self.player) {
            renderer.draw_quad(
                transform.position + Vec2::new(0.0, 50.0),
                Vec2::new(4.0, 10.0),
                0.0,
                Color::WHITE.with_alpha(0.5),
            );
        }
    }
    
    // Optional: Handle key press events
    fn on_key_pressed(&mut self, _engine: &mut Engine, key: KeyCode) {
        if key == KeyCode::KeyR {
            // Restart logic could go here
        }
    }
}

fn main() -> anyhow::Result<()> {
    grey_engine::run::<MyGame>()
}

