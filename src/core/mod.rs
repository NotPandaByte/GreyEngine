//! Core engine module.

use std::time::{Duration, Instant};

/// Time tracking for the game loop
#[derive(Debug)]
pub struct Time {
    start_time: Instant,
    last_frame_time: Instant,
    delta_time: Duration,
    total_time: Duration,
    frame_count: u64,
}

impl Default for Time {
    fn default() -> Self {
        Self::new()
    }
}

impl Time {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start_time: now,
            last_frame_time: now,
            delta_time: Duration::ZERO,
            total_time: Duration::ZERO,
            frame_count: 0,
        }
    }

    /// Call at the start of each frame
    pub fn update(&mut self) {
        let now = Instant::now();
        self.delta_time = now - self.last_frame_time;
        self.total_time = now - self.start_time;
        self.last_frame_time = now;
        self.frame_count += 1;
    }

    /// Time since last frame in seconds
    pub fn delta(&self) -> f32 {
        self.delta_time.as_secs_f32()
    }

    /// Time since last frame as Duration
    pub fn delta_duration(&self) -> Duration {
        self.delta_time
    }

    /// Total time since engine start in seconds
    pub fn total(&self) -> f32 {
        self.total_time.as_secs_f32()
    }

    /// Total frames rendered
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Approximate frames per second (smoothed)
    pub fn fps(&self) -> f32 {
        if self.delta_time.as_secs_f32() > 0.0 {
            1.0 / self.delta_time.as_secs_f32()
        } else {
            0.0
        }
    }
}

/// Engine configuration
#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub window_title: String,
    pub window_width: u32,
    pub window_height: u32,
    pub vsync: bool,
    pub clear_color: [f32; 4],
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            window_title: "GreyEngine".to_string(),
            window_width: 1280,
            window_height: 720,
            vsync: true,
            clear_color: [0.1, 0.1, 0.15, 1.0],
        }
    }
}
