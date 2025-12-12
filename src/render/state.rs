use std::{sync::Arc, time::SystemTime};

use anyhow::Result;
use winit::{
    event_loop::ActiveEventLoop,
    keyboard::KeyCode,
    window::Window,
};

use crate::{render::{context::RenderContext, pipeline::create_render_pipeline}};

pub struct State {
    context: RenderContext,
    is_surface_configured: bool,
    render_pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    position: [f32; 3],
    start_time: SystemTime,
    keyboard: Keyboard,
    window: Arc<Window>,
}

impl State {
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        let context = RenderContext::new(window.clone()).await?;

        // vec3<f32> in WGSL uniform buffers is aligned to 16 bytes (like vec4)
        let uniform_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: 16, // vec3<f32> requires 16 bytes alignment
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });
        let render_pipeline = create_render_pipeline(&context.device, &context.config, &bind_group_layout);
        let position = [0.0, 0.0, 0.0];
        let start_time = SystemTime::now();

        Ok(Self {
            context,
            is_surface_configured: false,
            render_pipeline,
            bind_group,
            uniform_buffer,
            position,
            start_time,
            window,
            keyboard: Keyboard::new(),
        })
    }


    
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.context.resize(width, height);
            self.is_surface_configured = true;
        }
    }

    pub fn handle_key(&self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        self.keyboard.handle_key_event(code, is_pressed);
        match (code, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            _ => {}
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.window.request_redraw();

        // we cant render unless the surface is configured
        if !self.is_surface_configured {
            return Ok(());
        }

        let output = self.context.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Pad position to 16 bytes (vec3<f32> alignment requirement)
        let padded_position = [self.position[0], self.position[1], self.position[2], 0.0f32];
        self.context.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&padded_position),
        );

        let mut encoder = self.context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
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
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // Use the render pipeline so it is not considered dead code,
            // and draw a simple triangle using the vertex_index trick in the shader.
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        // submit will accept anything that implements IntoIter<CommandBuffer>
        self.context.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    pub fn update(&mut self) {
        // Calculate time since app started for smooth animation
        let elapsed = SystemTime::now()
            .duration_since(self.start_time)
            .unwrap()
            .as_secs_f32();

        // Animate in a circle - position changes smoothly over time
        self.position[0] = elapsed.sin() * 0.3;
        self.position[1] = elapsed.cos() * 0.3;
        self.position[2] = 0.0;
    }
}