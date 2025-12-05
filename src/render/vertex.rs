//! Vertex types and mesh data.

use crate::math::{Vec2, Vec3, Color};

/// Vertex for 2D rendering with position, UV, and color
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex2D {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

impl Vertex2D {
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex2D>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x2,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x2,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                shader_location: 2,
                format: wgpu::VertexFormat::Float32x4,
            },
        ],
    };

    pub fn new(pos: Vec2, uv: Vec2, color: Color) -> Self {
        Self {
            position: [pos.x, pos.y],
            uv: [uv.x, uv.y],
            color: color.to_array(),
        }
    }
}

/// Vertex for 3D rendering
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex3D {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

impl Vertex3D {
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex3D>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                shader_location: 2,
                format: wgpu::VertexFormat::Float32x2,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                shader_location: 3,
                format: wgpu::VertexFormat::Float32x4,
            },
        ],
    };

    pub fn new(pos: Vec3, normal: Vec3, uv: Vec2, color: Color) -> Self {
        Self {
            position: [pos.x, pos.y, pos.z],
            normal: [normal.x, normal.y, normal.z],
            uv: [uv.x, uv.y],
            color: color.to_array(),
        }
    }
}

/// A mesh containing vertices and indices
#[derive(Debug, Clone)]
pub struct Mesh2D {
    pub vertices: Vec<Vertex2D>,
    pub indices: Vec<u16>,
}

impl Mesh2D {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Create a quad mesh
    pub fn quad(size: Vec2, color: Color) -> Self {
        let half = size * 0.5;
        Self {
            vertices: vec![
                Vertex2D::new(Vec2::new(-half.x, -half.y), Vec2::new(0.0, 1.0), color),
                Vertex2D::new(Vec2::new(half.x, -half.y), Vec2::new(1.0, 1.0), color),
                Vertex2D::new(Vec2::new(half.x, half.y), Vec2::new(1.0, 0.0), color),
                Vertex2D::new(Vec2::new(-half.x, half.y), Vec2::new(0.0, 0.0), color),
            ],
            indices: vec![0, 1, 2, 0, 2, 3],
        }
    }
}

impl Default for Mesh2D {
    fn default() -> Self {
        Self::new()
    }
}

/// A 3D mesh
#[derive(Debug, Clone)]
pub struct Mesh3D {
    pub vertices: Vec<Vertex3D>,
    pub indices: Vec<u32>,
}

impl Mesh3D {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Create a cube mesh
    pub fn cube(size: f32, color: Color) -> Self {
        let h = size / 2.0;
        let vertices = vec![
            // Front face
            Vertex3D::new(Vec3::new(-h, -h, h), Vec3::BACK, Vec2::new(0.0, 1.0), color),
            Vertex3D::new(Vec3::new(h, -h, h), Vec3::BACK, Vec2::new(1.0, 1.0), color),
            Vertex3D::new(Vec3::new(h, h, h), Vec3::BACK, Vec2::new(1.0, 0.0), color),
            Vertex3D::new(Vec3::new(-h, h, h), Vec3::BACK, Vec2::new(0.0, 0.0), color),
            // Back face
            Vertex3D::new(Vec3::new(h, -h, -h), Vec3::FORWARD, Vec2::new(0.0, 1.0), color),
            Vertex3D::new(Vec3::new(-h, -h, -h), Vec3::FORWARD, Vec2::new(1.0, 1.0), color),
            Vertex3D::new(Vec3::new(-h, h, -h), Vec3::FORWARD, Vec2::new(1.0, 0.0), color),
            Vertex3D::new(Vec3::new(h, h, -h), Vec3::FORWARD, Vec2::new(0.0, 0.0), color),
            // Top face
            Vertex3D::new(Vec3::new(-h, h, h), Vec3::UP, Vec2::new(0.0, 1.0), color),
            Vertex3D::new(Vec3::new(h, h, h), Vec3::UP, Vec2::new(1.0, 1.0), color),
            Vertex3D::new(Vec3::new(h, h, -h), Vec3::UP, Vec2::new(1.0, 0.0), color),
            Vertex3D::new(Vec3::new(-h, h, -h), Vec3::UP, Vec2::new(0.0, 0.0), color),
            // Bottom face
            Vertex3D::new(Vec3::new(-h, -h, -h), Vec3::DOWN, Vec2::new(0.0, 1.0), color),
            Vertex3D::new(Vec3::new(h, -h, -h), Vec3::DOWN, Vec2::new(1.0, 1.0), color),
            Vertex3D::new(Vec3::new(h, -h, h), Vec3::DOWN, Vec2::new(1.0, 0.0), color),
            Vertex3D::new(Vec3::new(-h, -h, h), Vec3::DOWN, Vec2::new(0.0, 0.0), color),
            // Right face
            Vertex3D::new(Vec3::new(h, -h, h), Vec3::RIGHT, Vec2::new(0.0, 1.0), color),
            Vertex3D::new(Vec3::new(h, -h, -h), Vec3::RIGHT, Vec2::new(1.0, 1.0), color),
            Vertex3D::new(Vec3::new(h, h, -h), Vec3::RIGHT, Vec2::new(1.0, 0.0), color),
            Vertex3D::new(Vec3::new(h, h, h), Vec3::RIGHT, Vec2::new(0.0, 0.0), color),
            // Left face
            Vertex3D::new(Vec3::new(-h, -h, -h), Vec3::LEFT, Vec2::new(0.0, 1.0), color),
            Vertex3D::new(Vec3::new(-h, -h, h), Vec3::LEFT, Vec2::new(1.0, 1.0), color),
            Vertex3D::new(Vec3::new(-h, h, h), Vec3::LEFT, Vec2::new(1.0, 0.0), color),
            Vertex3D::new(Vec3::new(-h, h, -h), Vec3::LEFT, Vec2::new(0.0, 0.0), color),
        ];

        let indices = vec![
            0, 1, 2, 0, 2, 3,       // front
            4, 5, 6, 4, 6, 7,       // back
            8, 9, 10, 8, 10, 11,    // top
            12, 13, 14, 12, 14, 15, // bottom
            16, 17, 18, 16, 18, 19, // right
            20, 21, 22, 20, 22, 23, // left
        ];

        Self { vertices, indices }
    }
}

impl Default for Mesh3D {
    fn default() -> Self {
        Self::new()
    }
}

