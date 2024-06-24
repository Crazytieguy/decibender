#![warn(clippy::pedantic)]

use rspotify::{clients::OAuthClient, model::PlaylistId};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let device_name = "ATR4697-USB";
    let spotify = decibender::spotify::init().await?;
    let mut loudness_rx = decibender::audio::watch_loudness(device_name)?;
    let mut state = State::Paused;
    loop {
        match state {
            State::Paused => {
                loudness_rx.wait_for(|loudness| *loudness > -30.0).await?;
                spotify
                    .start_context_playback(
                        PlaylistId::from_id("5ICnfTOofjViwY7LHTSJUh")?.into(),
                        None,
                        None,
                        None,
                    )
                    .await?;
                state = State::Playing;
            }
            State::Playing => {
                loudness_rx.wait_for(|loudness| *loudness < -40.0).await?;
                spotify.pause_playback(None).await?;
                state = State::Paused;
            }
        }
    }
}

enum State {
    Playing,
    Paused,
}
