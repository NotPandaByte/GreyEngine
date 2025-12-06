//! Rendering and graphics module.
//!
//! Typical submodules you might add:
//! - `backend` for the low-level graphics API wrapper (wgpu, vulkan, etc.)
//! - `camera` for view/projection handling
//! - `mesh` / `material` / `texture` resources
//! - `renderer2d` / `renderer3d` high-level drawing logic

mod app;
pub mod context;
pub mod pipeline;
pub mod state;

use anyhow::Result;
use winit::event_loop::EventLoop;

pub fn run() -> Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    }
    #[cfg(target_arch = "wasm32")]
    {
        console_log::init_with_level(log::Level::Info).unwrap_throw();
    }

    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = app::App::new(
        #[cfg(target_arch = "wasm32")]
        &event_loop,
    );
    event_loop.run_app(&mut app)?;

    Ok(())
}