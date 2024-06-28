use std::env;
use std::path::PathBuf;

use rspotify::{clients::OAuthClient, AuthCodeSpotify};

use crate::audio::PlayHandle;

pub async fn too_loud(file_path: &PathBuf) -> anyhow::Result<PlayHandle> {
    lights_on().await?;
    crate::audio::play_file(file_path)
}

pub async fn acceptable(
    spotify: &AuthCodeSpotify,
    _play_handle: Option<PlayHandle>, // We just drop this to stop playback
) -> anyhow::Result<()> {
    // TODO: Turn on the nice lights
    if let Err(e) = spotify.resume_playback(None, None).await {
        if e.to_string().contains("403") {
            // 403 is returned by spotify when already playing back
            log::warn!("Failed to resume playback: {}", e)
        } else {
            return Err(e.into());
        }
    };
    lights_off().await?;
    Ok(())
}

pub async fn too_quite(spotify: &AuthCodeSpotify) -> anyhow::Result<()> {
    // TODO: Turn off the nice lights
    if let Err(e) = spotify.pause_playback(None).await {
        if e.to_string().contains("403") {
            // 403 is returned by spotify when spotify is already paused
            log::warn!("Failed to resume playback: {}", e)
        } else {
            return Err(e.into());
        }
    };
    Ok(())
}

async fn lights_on() -> anyhow::Result<()> {
    const ON_URL: &str = env!("LIGHTS_ON_URL");
    reqwest::get(ON_URL).await?;
    Ok(())
}

async fn lights_off() -> anyhow::Result<()> {
    const OFF_URL: &str = env!("LIGHTS_OFF_URL");
    reqwest::get(OFF_URL).await?;
    Ok(())
}
