//! 2D Batch Renderer for sprites and shapes.

use crate::math::{Vec2, Color, Mat4};
use super::vertex::Vertex2D;
use super::camera::Camera2D;
use super::texture::Texture;
use wgpu::util::DeviceExt;

const MAX_QUADS: usize = 10000;
const MAX_VERTICES: usize = MAX_QUADS * 4;
const MAX_INDICES: usize = MAX_QUADS * 6;

/// Uniform buffer for camera data
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

/// 2D Batch Renderer
pub struct Renderer2D {
    pipeline_textured: wgpu::RenderPipeline,
    pipeline_colored: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    #[allow(dead_code)]
    white_texture: Texture,
    #[allow(dead_code)]
    white_texture_bind_group: wgpu::BindGroup,
    
    vertices: Vec<Vertex2D>,
    quad_count: usize,
}

impl Renderer2D {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("2D Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        // Camera bind group layout
        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera_bind_group_layout"),
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

        // Texture bind group layout
        let texture_bind_group_layout = Texture::bind_group_layout(device);

        // Pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("2D Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Colored pipeline (no texture)
        let pipeline_layout_colored = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("2D Colored Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline_textured = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("2D Textured Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main_2d"),
                buffers: &[Vertex2D::LAYOUT],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main_2d"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let pipeline_colored = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("2D Colored Pipeline"),
            layout: Some(&pipeline_layout_colored),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main_2d"),
                buffers: &[Vertex2D::LAYOUT],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main_2d_color"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Vertex buffer
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("2D Vertex Buffer"),
            size: (MAX_VERTICES * std::mem::size_of::<Vertex2D>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Index buffer (pre-generate indices for quads)
        let mut indices: Vec<u16> = Vec::with_capacity(MAX_INDICES);
        for i in 0..MAX_QUADS {
            let base = (i * 4) as u16;
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("2D Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Camera buffer
        let camera_uniform = CameraUniform {
            view_proj: Mat4::IDENTITY.cols,
        };
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        // White texture for colored quads
        let white_texture = Texture::white_pixel(device, queue);
        let white_texture_bind_group = white_texture.create_bind_group(device, &texture_bind_group_layout);

        Self {
            pipeline_textured,
            pipeline_colored,
            vertex_buffer,
            index_buffer,
            camera_buffer,
            camera_bind_group,
            texture_bind_group_layout,
            white_texture,
            white_texture_bind_group,
            vertices: Vec::with_capacity(MAX_VERTICES),
            quad_count: 0,
        }
    }

    /// Begin a new frame
    pub fn begin(&mut self, camera: &Camera2D, queue: &wgpu::Queue) {
        self.vertices.clear();
        self.quad_count = 0;

        // Update camera uniform
        let view_proj = camera.view_projection();
        let camera_uniform = CameraUniform {
            view_proj: view_proj.cols,
        };
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[camera_uniform]));
    }

    /// Draw a colored quad
    pub fn draw_quad(&mut self, position: Vec2, size: Vec2, rotation: f32, color: Color) {
        if self.quad_count >= MAX_QUADS {
            return;
        }

        let half = size * 0.5;
        let (sin_r, cos_r) = (rotation.sin(), rotation.cos());

        // Rotate and translate corners
        let corners = [
            Vec2::new(-half.x, -half.y),
            Vec2::new(half.x, -half.y),
            Vec2::new(half.x, half.y),
            Vec2::new(-half.x, half.y),
        ];

        for (i, corner) in corners.iter().enumerate() {
            let rotated = Vec2::new(
                corner.x * cos_r - corner.y * sin_r,
                corner.x * sin_r + corner.y * cos_r,
            );
            let pos = position + rotated;
            let uv = match i {
                0 => Vec2::new(0.0, 1.0),
                1 => Vec2::new(1.0, 1.0),
                2 => Vec2::new(1.0, 0.0),
                _ => Vec2::new(0.0, 0.0),
            };
            self.vertices.push(Vertex2D::new(pos, uv, color));
        }

        self.quad_count += 1;
    }

    /// Draw a sprite with texture coordinates
    pub fn draw_sprite(&mut self, position: Vec2, size: Vec2, rotation: f32, color: Color, uv_rect: [f32; 4]) {
        if self.quad_count >= MAX_QUADS {
            return;
        }

        let half = size * 0.5;
        let (sin_r, cos_r) = (rotation.sin(), rotation.cos());

        let corners = [
            Vec2::new(-half.x, -half.y),
            Vec2::new(half.x, -half.y),
            Vec2::new(half.x, half.y),
            Vec2::new(-half.x, half.y),
        ];

        let uvs = [
            Vec2::new(uv_rect[0], uv_rect[1] + uv_rect[3]),
            Vec2::new(uv_rect[0] + uv_rect[2], uv_rect[1] + uv_rect[3]),
            Vec2::new(uv_rect[0] + uv_rect[2], uv_rect[1]),
            Vec2::new(uv_rect[0], uv_rect[1]),
        ];

        for (i, corner) in corners.iter().enumerate() {
            let rotated = Vec2::new(
                corner.x * cos_r - corner.y * sin_r,
                corner.x * sin_r + corner.y * cos_r,
            );
            let pos = position + rotated;
            self.vertices.push(Vertex2D::new(pos, uvs[i], color));
        }

        self.quad_count += 1;
    }

    /// Flush and render all batched quads (colored only)
    pub fn flush_colored<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, queue: &wgpu::Queue) {
        if self.quad_count == 0 {
            return;
        }

        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.vertices));

        render_pass.set_pipeline(&self.pipeline_colored);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..(self.quad_count * 6) as u32, 0, 0..1);
    }

    /// Flush and render all batched quads with texture
    pub fn flush_textured<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        queue: &wgpu::Queue,
        texture_bind_group: &'a wgpu::BindGroup,
    ) {
        if self.quad_count == 0 {
            return;
        }

        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.vertices));

        render_pass.set_pipeline(&self.pipeline_textured);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.set_bind_group(1, texture_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..(self.quad_count * 6) as u32, 0, 0..1);
    }

    /// Get the texture bind group layout for creating texture bind groups
    pub fn texture_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.texture_bind_group_layout
    }

    /// Draw a number at the given position
    /// Returns the width of the drawn number
    pub fn draw_number(&mut self, position: Vec2, number: i32, scale: f32, color: Color) -> f32 {
        let digit_width = 12.0 * scale;
        let spacing = 2.0 * scale;
        
        let s = if number < 0 {
            format!("{}", number)
        } else {
            format!("{}", number)
        };
        
        let mut x = position.x;
        for ch in s.chars() {
            if ch == '-' {
                // Minus sign
                self.draw_quad(
                    Vec2::new(x + digit_width * 0.5, position.y),
                    Vec2::new(digit_width * 0.6, 2.0 * scale),
                    0.0,
                    color,
                );
            } else if let Some(digit) = ch.to_digit(10) {
                self.draw_digit_segments(Vec2::new(x, position.y), digit as u8, scale, color);
            }
            x += digit_width + spacing;
        }
        
        x - position.x - spacing
    }

    /// Draw a single digit using 7-segment style
    fn draw_digit_segments(&mut self, pos: Vec2, digit: u8, scale: f32, color: Color) {
        let w = 12.0 * scale;
        let h = 20.0 * scale;
        let t = 2.5 * scale; // segment thickness
        
        // Segment positions:
        //  AAA
        // F   B
        //  GGG
        // E   C
        //  DDD
        
        let segments: [bool; 7] = match digit {
            0 => [true,  true,  true,  true,  true,  true,  false],
            1 => [false, true,  true,  false, false, false, false],
            2 => [true,  true,  false, true,  true,  false, true],
            3 => [true,  true,  true,  true,  false, false, true],
            4 => [false, true,  true,  false, false, true,  true],
            5 => [true,  false, true,  true,  false, true,  true],
            6 => [true,  false, true,  true,  true,  true,  true],
            7 => [true,  true,  true,  false, false, false, false],
            8 => [true,  true,  true,  true,  true,  true,  true],
            9 => [true,  true,  true,  true,  false, true,  true],
            _ => [false; 7],
        };
        
        let cx = pos.x + w * 0.5;
        let cy = pos.y;
        
        // A - top horizontal
        if segments[0] {
            self.draw_quad(Vec2::new(cx, cy + h * 0.5 - t * 0.5), Vec2::new(w - t * 2.0, t), 0.0, color);
        }
        // B - top right vertical
        if segments[1] {
            self.draw_quad(Vec2::new(cx + w * 0.5 - t * 0.5, cy + h * 0.25), Vec2::new(t, h * 0.5 - t), 0.0, color);
        }
        // C - bottom right vertical
        if segments[2] {
            self.draw_quad(Vec2::new(cx + w * 0.5 - t * 0.5, cy - h * 0.25), Vec2::new(t, h * 0.5 - t), 0.0, color);
        }
        // D - bottom horizontal
        if segments[3] {
            self.draw_quad(Vec2::new(cx, cy - h * 0.5 + t * 0.5), Vec2::new(w - t * 2.0, t), 0.0, color);
        }
        // E - bottom left vertical
        if segments[4] {
            self.draw_quad(Vec2::new(cx - w * 0.5 + t * 0.5, cy - h * 0.25), Vec2::new(t, h * 0.5 - t), 0.0, color);
        }
        // F - top left vertical
        if segments[5] {
            self.draw_quad(Vec2::new(cx - w * 0.5 + t * 0.5, cy + h * 0.25), Vec2::new(t, h * 0.5 - t), 0.0, color);
        }
        // G - middle horizontal
        if segments[6] {
            self.draw_quad(Vec2::new(cx, cy), Vec2::new(w - t * 2.0, t), 0.0, color);
        }
    }

    /// Draw text-like label using simple block letters (limited charset: A-Z, 0-9, space, colon)
    pub fn draw_text(&mut self, position: Vec2, text: &str, scale: f32, color: Color) -> f32 {
        let char_width = 12.0 * scale;
        let spacing = 3.0 * scale;
        
        let mut x = position.x;
        for ch in text.chars() {
            match ch {
                '0'..='9' => {
                    let digit = ch.to_digit(10).unwrap() as u8;
                    self.draw_digit_segments(Vec2::new(x, position.y), digit, scale, color);
                }
                ':' => {
                    let t = 2.5 * scale;
                    self.draw_quad(Vec2::new(x + char_width * 0.3, position.y + 5.0 * scale), Vec2::new(t, t), 0.0, color);
                    self.draw_quad(Vec2::new(x + char_width * 0.3, position.y - 5.0 * scale), Vec2::new(t, t), 0.0, color);
                }
                ' ' => {}
                'F' | 'f' => {
                    self.draw_letter_f(Vec2::new(x, position.y), scale, color);
                }
                'P' | 'p' => {
                    self.draw_letter_p(Vec2::new(x, position.y), scale, color);
                }
                'S' | 's' => {
                    // S looks like 5
                    self.draw_digit_segments(Vec2::new(x, position.y), 5, scale, color);
                }
                _ => {
                    // Unknown char - draw a box
                    self.draw_quad(Vec2::new(x + char_width * 0.5, position.y), Vec2::new(char_width * 0.8, 20.0 * scale), 0.0, color.with_alpha(0.3));
                }
            }
            x += char_width + spacing;
        }
        
        x - position.x - spacing
    }

    fn draw_letter_f(&mut self, pos: Vec2, scale: f32, color: Color) {
        let w = 12.0 * scale;
        let h = 20.0 * scale;
        let t = 2.5 * scale;
        let cx = pos.x + w * 0.5;
        let cy = pos.y;
        
        // Top horizontal
        self.draw_quad(Vec2::new(cx, cy + h * 0.5 - t * 0.5), Vec2::new(w - t, t), 0.0, color);
        // Left vertical (full height)
        self.draw_quad(Vec2::new(cx - w * 0.5 + t * 0.5, cy), Vec2::new(t, h), 0.0, color);
        // Middle horizontal
        self.draw_quad(Vec2::new(cx - t * 0.5, cy), Vec2::new(w * 0.6, t), 0.0, color);
    }

    fn draw_letter_p(&mut self, pos: Vec2, scale: f32, color: Color) {
        let w = 12.0 * scale;
        let h = 20.0 * scale;
        let t = 2.5 * scale;
        let cx = pos.x + w * 0.5;
        let cy = pos.y;
        
        // Top horizontal
        self.draw_quad(Vec2::new(cx, cy + h * 0.5 - t * 0.5), Vec2::new(w - t, t), 0.0, color);
        // Left vertical (full height)
        self.draw_quad(Vec2::new(cx - w * 0.5 + t * 0.5, cy), Vec2::new(t, h), 0.0, color);
        // Right vertical (top half)
        self.draw_quad(Vec2::new(cx + w * 0.5 - t * 0.5, cy + h * 0.25), Vec2::new(t, h * 0.5 - t), 0.0, color);
        // Middle horizontal
        self.draw_quad(Vec2::new(cx, cy), Vec2::new(w - t, t), 0.0, color);
    }
}

