use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Thresholds {
    pub too_loud: f32,
    pub too_quite: f32,
    pub grace: f32,
}

impl Thresholds {
    pub fn too_loud(&self, loudness: f32) -> bool {
        loudness > self.too_loud
    }

    pub fn too_quite(&self, loudness: f32) -> bool {
        loudness < self.too_quite
    }

    pub fn acceptable_from_too_loud(&self, loudness: f32) -> bool {
        loudness < self.too_loud - self.grace
    }

    pub fn acceptable_from_too_quite(&self, loudness: f32) -> bool {
        loudness > self.too_quite + self.grace
    }
}
