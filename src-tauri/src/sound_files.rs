use rand::seq::SliceRandom;
use std::path::PathBuf;

use tauri::AppHandle;

pub struct SoundFiles {
    pub annoying: PathBuf,
    pub too_loud_anouncement: PathBuf,
    pub too_quiet_anouncement: PathBuf,
    pub louder_anouncements: Vec<PathBuf>,
    pub quieter_anouncements: Vec<PathBuf>,
}

impl SoundFiles {
    pub fn random_louder_announcement(&self) -> &PathBuf {
        self.louder_anouncements
            .choose(&mut rand::thread_rng())
            .expect("At least one louder announcement file")
    }

    pub fn random_quieter_announcement(&self) -> &PathBuf {
        self.quieter_anouncements
            .choose(&mut rand::thread_rng())
            .expect("At least one quieter announcement file")
    }

    pub fn resolve(app_handle: &AppHandle) -> anyhow::Result<Self> {
        Ok(SoundFiles {
            annoying: app_handle
                .path_resolver()
                .resolve_resource(env!("ANNOYING_FILE"))
                .ok_or_else(|| anyhow::anyhow!("Failed to resolve annoying file"))?,
            too_loud_anouncement: app_handle
                .path_resolver()
                .resolve_resource(env!("TOO_LOUD_ANNOUNCEMENT_FILE"))
                .ok_or_else(|| anyhow::anyhow!("Failed to resolve too loud announcement file"))?,
            too_quiet_anouncement: app_handle
                .path_resolver()
                .resolve_resource(env!("TOO_QUIET_ANNOUNCEMENT_FILE"))
                .ok_or_else(|| anyhow::anyhow!("Failed to resolve too quiet announcement file"))?,
            louder_anouncements: env!("LOUDER_ANNOUNCEMENT_FILES")
                .split(",")
                .map(|name| {
                    app_handle
                        .path_resolver()
                        .resolve_resource(name)
                        .ok_or_else(|| {
                            anyhow::anyhow!("Failed to resolve louder announcement file")
                        })
                })
                .collect::<anyhow::Result<Vec<PathBuf>>>()?,
            quieter_anouncements: env!("QUIETER_ANNOUNCEMENT_FILES")
                .split(",")
                .map(|name| {
                    app_handle
                        .path_resolver()
                        .resolve_resource(name)
                        .ok_or_else(|| {
                            anyhow::anyhow!("Failed to resolve quieter announcement file")
                        })
                })
                .collect::<anyhow::Result<Vec<PathBuf>>>()?,
        })
    }
}
