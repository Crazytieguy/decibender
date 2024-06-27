use rspotify::{clients::OAuthClient, AuthCodeSpotify};

use crate::audio::PlayHandle;

pub async fn too_loud() -> anyhow::Result<PlayHandle> {
    crate::lights::on().await?;
    crate::audio::play_file("assets/man-screaming-6373.mp3")
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
    crate::lights::off().await?;
    Ok(())
}

pub async fn too_quite(spotify: &AuthCodeSpotify) -> anyhow::Result<()> {
    // TODO: Turn off the nice lights
    spotify.pause_playback(None).await?;
    Ok(())
}
