# GreyEngine

A simple 2D/3D game engine built with Rust and wgpu.

## Features

- **ECS (Entity Component System)**: Simple but effective entity management
- **2D Rendering**: Batch sprite rendering with camera support
- **3D Ready**: Camera and mesh primitives for 3D development
- **Input Handling**: Keyboard and mouse input with press/release detection
- **Math Library**: Vec2, Vec3, Vec4, Mat4, Color, Rect, Transform types
- **Asset Management**: Basic asset loading and caching
- **Scene Graph**: Hierarchical entity management
- **Cross-Platform**: Runs on Windows, macOS, Linux, and Web (via wasm)

## Quick Start

```rust
use grey_engine::prelude::*;

struct MyGame {
    player: Entity,
}

impl Application for MyGame {
    fn init(engine: &mut Engine) -> Self {
        let player = engine.world.spawn();
        engine.world.add(player, Transform2D::new(Vec2::ZERO));
        engine.world.add(player, Sprite::colored(Color::RED, Vec2::new(50.0, 50.0)));
        
        Self { player }
    }
    
    fn update(&mut self, engine: &mut Engine, dt: f32) {
        let movement = engine.input.get_movement_input();
        
        if let Some(transform) = engine.world.get_mut::<Transform2D>(self.player) {
            transform.position += movement * 200.0 * dt;
        }
    }
}

fn main() -> anyhow::Result<()> {
    grey_engine::run::<MyGame>()
}
```

## Controls

- **WASD / Arrow Keys**: Movement
- **Escape**: Exit

## Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run demo
cargo run --bin grey_engine_demo

# Run example
cargo run --example simple
```

## Architecture

```
src/
  lib.rs          - Main engine entry point and Application trait
  core/           - Time, configuration
  ecs/            - Entity Component System
  input/          - Keyboard and mouse handling
  math/           - Vector, matrix, color types
  render/         - wgpu rendering, cameras, textures
  scene/          - Scene graph
  platform/       - OS utilities
  assets/         - Asset loading
```

## Dependencies

- `wgpu` - Graphics API
- `winit` - Windowing
- `bytemuck` - Safe transmutes for GPU buffers
- `anyhow` - Error handling

