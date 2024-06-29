use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct Thresholds {
    pub too_loud: f32,
    pub too_quiet: f32,
    pub grace: f32,
}

impl Thresholds {
    pub fn too_loud(&self, loudness: f32) -> bool {
        loudness > self.too_loud
    }

    pub fn too_quiet(&self, loudness: f32) -> bool {
        loudness < self.too_quiet
    }

    pub fn acceptable_from_too_loud(&self, loudness: f32) -> bool {
        loudness < self.too_loud - self.grace
    }

    pub fn acceptable_from_too_quiet(&self, loudness: f32) -> bool {
        loudness > self.too_quiet + self.grace
    }
}
