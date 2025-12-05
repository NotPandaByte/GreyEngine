//! Camera systems for 2D and 3D rendering.

use crate::math::{Mat4, Vec2, Vec3};

/// 2D Camera for orthographic projection
#[derive(Debug, Clone)]
pub struct Camera2D {
    pub position: Vec2,
    pub zoom: f32,
    pub rotation: f32,
    viewport_size: Vec2,
}

impl Default for Camera2D {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            zoom: 1.0,
            rotation: 0.0,
            viewport_size: Vec2::new(1280.0, 720.0),
        }
    }
}

impl Camera2D {
    pub fn new(viewport_width: f32, viewport_height: f32) -> Self {
        Self {
            viewport_size: Vec2::new(viewport_width, viewport_height),
            ..Default::default()
        }
    }

    pub fn set_viewport(&mut self, width: f32, height: f32) {
        self.viewport_size = Vec2::new(width, height);
    }

    pub fn viewport_size(&self) -> Vec2 {
        self.viewport_size
    }

    /// Get the view-projection matrix
    pub fn view_projection(&self) -> Mat4 {
        let half_w = self.viewport_size.x / 2.0 / self.zoom;
        let half_h = self.viewport_size.y / 2.0 / self.zoom;

        let projection = Mat4::orthographic(
            -half_w, half_w,
            -half_h, half_h,
            -1.0, 1.0,
        );

        let view = Mat4::translation(Vec3::new(-self.position.x, -self.position.y, 0.0))
            * Mat4::rotation_z(-self.rotation);

        projection * view
    }

    /// Convert screen coordinates to world coordinates
    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        let normalized = Vec2::new(
            (screen_pos.x / self.viewport_size.x) * 2.0 - 1.0,
            1.0 - (screen_pos.y / self.viewport_size.y) * 2.0,
        );

        let half_w = self.viewport_size.x / 2.0 / self.zoom;
        let half_h = self.viewport_size.y / 2.0 / self.zoom;

        Vec2::new(
            normalized.x * half_w + self.position.x,
            normalized.y * half_h + self.position.y,
        )
    }

    /// Convert world coordinates to screen coordinates
    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        let half_w = self.viewport_size.x / 2.0 / self.zoom;
        let half_h = self.viewport_size.y / 2.0 / self.zoom;

        let normalized = Vec2::new(
            (world_pos.x - self.position.x) / half_w,
            (world_pos.y - self.position.y) / half_h,
        );

        Vec2::new(
            (normalized.x + 1.0) * 0.5 * self.viewport_size.x,
            (1.0 - normalized.y) * 0.5 * self.viewport_size.y,
        )
    }
}

/// 3D Camera for perspective projection
#[derive(Debug, Clone)]
pub struct Camera3D {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    aspect_ratio: f32,
}

impl Default for Camera3D {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 5.0),
            target: Vec3::ZERO,
            up: Vec3::UP,
            fov: 60.0_f32.to_radians(),
            near: 0.1,
            far: 1000.0,
            aspect_ratio: 16.0 / 9.0,
        }
    }
}

impl Camera3D {
    pub fn new(position: Vec3, target: Vec3) -> Self {
        Self {
            position,
            target,
            ..Default::default()
        }
    }

    pub fn set_aspect(&mut self, width: f32, height: f32) {
        self.aspect_ratio = width / height;
    }

    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at(self.position, self.target, self.up)
    }

    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective(self.fov, self.aspect_ratio, self.near, self.far)
    }

    pub fn view_projection(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    /// Get forward direction
    pub fn forward(&self) -> Vec3 {
        (self.target - self.position).normalize()
    }

    /// Get right direction
    pub fn right(&self) -> Vec3 {
        self.forward().cross(self.up).normalize()
    }
}

