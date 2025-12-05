//! GreyEngine - A simple game engine built with Rust and wgpu.
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use grey_engine::prelude::*;
//!
//! struct MyGame {
//!     player: Entity,
//! }
//!
//! impl Application for MyGame {
//!     fn init(engine: &mut Engine) -> Self {
//!         let player = engine.world.spawn();
//!         engine.world.add(player, Transform2D::new(Vec2::ZERO));
//!         engine.world.add(player, Sprite::colored(Color::RED, Vec2::new(50.0, 50.0)));
//!         Self { player }
//!     }
//!
//!     fn update(&mut self, engine: &mut Engine, dt: f32) {
//!         let movement = engine.input.get_movement_input();
//!         if let Some(transform) = engine.world.get_mut::<Transform2D>(self.player) {
//!             transform.position += movement * 200.0 * dt;
//!         }
//!     }
//! }
//!
//! fn main() {
//!     grey_engine::run::<MyGame>().unwrap();
//! }
//! ```

pub mod core;
pub mod ecs;
pub mod input;
pub mod math;
pub mod render;

// Re-export commonly used types
pub mod prelude {
    pub use crate::core::{Time, EngineConfig};
    pub use crate::ecs::{Entity, World, Transform2D, Transform3D, Sprite, Velocity2D, Name};
    pub use crate::input::{Input, MouseButton};
    pub use crate::math::{Vec2, Vec3, Vec4, Mat4, Color, Rect};
    pub use crate::render::{Camera2D, Camera3D, Renderer2D, Texture};
    pub use crate::{Application, Engine};
}

use std::sync::Arc;
use anyhow::Result;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

use crate::core::Time;
use crate::ecs::World;
use crate::input::{Input, MouseButton};
use crate::math::Color;
use crate::render::{Camera2D, Renderer2D};

/// The main engine state accessible to the application
pub struct Engine {
    pub world: World,
    pub input: Input,
    pub time: Time,
    pub camera: Camera2D,
    pub clear_color: Color,
}

impl Engine {
    fn new(width: f32, height: f32) -> Self {
        Self {
            world: World::new(),
            input: Input::new(),
            time: Time::new(),
            camera: Camera2D::new(width, height),
            clear_color: Color::new(0.1, 0.1, 0.15, 1.0),
        }
    }
}

/// Trait that users implement to create their game
pub trait Application: Sized + 'static {
    /// Called once when the application starts
    fn init(engine: &mut Engine) -> Self;
    
    /// Called every frame to update game logic
    fn update(&mut self, engine: &mut Engine, dt: f32);
    
    /// Called every frame to render (optional - engine handles default sprite rendering)
    fn render(&mut self, _engine: &Engine, _renderer: &mut Renderer2D) {}
    
    /// Called when a key is pressed
    fn on_key_pressed(&mut self, _engine: &mut Engine, _key: KeyCode) {}
    
    /// Called when a key is released
    fn on_key_released(&mut self, _engine: &mut Engine, _key: KeyCode) {}
}

/// Internal render state
struct RenderState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
    renderer: Renderer2D,
    window: Arc<Window>,
}

impl RenderState {
    async fn new(window: Arc<Window>) -> Result<Self> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone())?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        let renderer = Renderer2D::new(&device, &queue, surface_format);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,
            renderer,
            window,
        })
    }

    fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;
        }
    }

    fn render<A: Application>(
        &mut self,
        engine: &Engine,
        app: &mut A,
    ) -> Result<(), wgpu::SurfaceError> {
        self.window.request_redraw();

        if !self.is_surface_configured {
            return Ok(());
        }

        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Begin rendering with camera
        self.renderer.begin(&engine.camera, &self.queue);

        // Render all sprites from ECS
        for (entity, sprite) in engine.world.query::<crate::ecs::Sprite>() {
            if let Some(transform) = engine.world.get::<crate::ecs::Transform2D>(entity) {
                self.renderer.draw_sprite(
                    transform.position,
                    sprite.size * transform.scale,
                    transform.rotation,
                    sprite.color,
                    sprite.uv_rect,
                );
            }
        }

        // Let app add custom rendering
        app.render(engine, &mut self.renderer);

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: engine.clear_color.r as f64,
                            g: engine.clear_color.g as f64,
                            b: engine.clear_color.b as f64,
                            a: engine.clear_color.a as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            self.renderer.flush_colored(&mut render_pass, &self.queue);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}

/// Internal app wrapper
struct AppRunner<A: Application> {
    render_state: Option<RenderState>,
    engine: Option<Engine>,
    app: Option<A>,
}

impl<A: Application> AppRunner<A> {
    fn new() -> Self {
        Self {
            render_state: None,
            engine: None,
            app: None,
        }
    }
}

impl<A: Application> ApplicationHandler for AppRunner<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes().with_title("GreyEngine");
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        let render_state = pollster::block_on(RenderState::new(window)).unwrap();
        
        let size = render_state.window.inner_size();
        let mut engine = Engine::new(size.width as f32, size.height as f32);
        let app = A::init(&mut engine);
        
        self.render_state = Some(render_state);
        self.engine = Some(engine);
        self.app = Some(app);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let (render_state, engine, app) = match (&mut self.render_state, &mut self.engine, &mut self.app) {
            (Some(rs), Some(e), Some(a)) => (rs, e, a),
            _ => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            
            WindowEvent::Resized(size) => {
                render_state.resize(size.width, size.height);
                engine.camera.set_viewport(size.width as f32, size.height as f32);
            }
            
            WindowEvent::RedrawRequested => {
                engine.input.begin_frame();
                engine.time.update();
                
                let dt = engine.time.delta();
                app.update(engine, dt);
                
                if let Err(e) = render_state.render(engine, app) {
                    match e {
                        wgpu::SurfaceError::Lost => {
                            render_state.resize(render_state.config.width, render_state.config.height);
                        }
                        wgpu::SurfaceError::OutOfMemory => event_loop.exit(),
                        _ => log::error!("Render error: {:?}", e),
                    }
                }
            }
            
            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    physical_key: PhysicalKey::Code(code),
                    state: key_state,
                    ..
                },
                ..
            } => {
                if key_state.is_pressed() {
                    engine.input.on_key_pressed(code);
                    app.on_key_pressed(engine, code);
                    
                    if code == KeyCode::Escape {
                        event_loop.exit();
                    }
                } else {
                    engine.input.on_key_released(code);
                    app.on_key_released(engine, code);
                }
            }
            
            WindowEvent::MouseInput { state, button, .. } => {
                let btn = match button {
                    winit::event::MouseButton::Left => MouseButton::Left,
                    winit::event::MouseButton::Right => MouseButton::Right,
                    winit::event::MouseButton::Middle => MouseButton::Middle,
                    _ => return,
                };
                
                if state.is_pressed() {
                    engine.input.on_mouse_button_pressed(btn);
                } else {
                    engine.input.on_mouse_button_released(btn);
                }
            }
            
            WindowEvent::CursorMoved { position, .. } => {
                engine.input.on_mouse_moved(position.x as f32, position.y as f32);
            }
            
            WindowEvent::MouseWheel { delta, .. } => {
                let (x, y) = match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => (x, y),
                    winit::event::MouseScrollDelta::PixelDelta(pos) => (pos.x as f32, pos.y as f32),
                };
                engine.input.on_scroll(x, y);
            }
            
            _ => {}
        }
    }
}

/// Run the engine with the given Application type
pub fn run<A: Application>() -> Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    env_logger::init();

    let event_loop = EventLoop::builder().build()?;
    let mut app = AppRunner::<A>::new();
    event_loop.run_app(&mut app)?;

    Ok(())
}

