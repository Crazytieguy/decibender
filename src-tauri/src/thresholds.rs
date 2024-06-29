use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Thresholds {
    pub too_loud: f32,
    pub too_quiet: f32,
    pub grace: f32,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            too_loud: -30.0,
            too_quiet: -80.0,
            grace: 6.0,
        }
    }
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

    pub fn louder(&mut self) {
        log::info!("Louder");
        self.too_loud += 6.0;
        self.too_quiet += 6.0;
    }

    pub fn quieter(&mut self) {
        log::info!("Quieter");
        self.too_loud -= 6.0;
        self.too_quiet -= 6.0;
    }
}
