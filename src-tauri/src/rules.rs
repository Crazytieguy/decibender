use std::{
    env,
    sync::Arc,
    time::{Duration, Instant},
};

use rspotify::{clients::OAuthClient, AuthCodeSpotify};
use tauri::AppHandle;
use tokio::{sync::mpsc, time::sleep};

use crate::{
    audio::{self, PlayHandle},
    sound_files::SoundFiles,
    spotify,
};

pub struct RuleExecutor {
    spotify: AuthCodeSpotify,
    sound_files: SoundFiles,
    play_handle_tx: mpsc::Sender<Option<PlayHandle>>,
}

impl RuleExecutor {
    pub async fn new(app_handle: &AppHandle) -> anyhow::Result<Arc<Self>> {
        let sound_files = SoundFiles::resolve(&app_handle)?;
        let spotify = spotify::init().await?;
        let mut play_handle: Option<PlayHandle> = None;
        let (play_handle_tx, mut play_handle_rx) = mpsc::channel::<Option<PlayHandle>>(4);
        tokio::spawn(async move {
            while let Some(new_play_handle) = play_handle_rx.recv().await {
                drop(play_handle.take());
                play_handle = new_play_handle;
            }
        });
        Ok(Arc::new(Self {
            spotify,
            sound_files,
            play_handle_tx,
        }))
    }

    pub async fn announce_louder(self: Arc<Self>) {
        if let Err::<(), anyhow::Error>(e) = try {
            let play_handle = audio::play_file(&self.sound_files.random_louder_announcement())?;
            self.play_handle_tx.send(Some(play_handle)).await?;
        } {
            log::error!("{:?}", e.context("Announce louder failed"));
        }
    }

    pub async fn announce_quieter(self: Arc<Self>) {
        if let Err::<(), anyhow::Error>(e) = try {
            let play_handle = audio::play_file(&self.sound_files.random_quieter_announcement())?;
            self.play_handle_tx.send(Some(play_handle)).await?;
        } {
            log::error!("{:?}", e.context("Announce quieter failed"));
        }
    }

    pub async fn too_loud(self: Arc<Self>) {
        log::info!("Too loud");
        if let Err::<(), anyhow::Error>(e) = try {
            let play_handle = audio::play_file(&self.sound_files.too_loud_anouncement)?;
            let expect_done_at = play_handle.expect_done_at();
            self.play_handle_tx.send(Some(play_handle)).await?;
            sleep(expect_done_at - Instant::now()).await;

            let play_handle = audio::play_file(&self.sound_files.annoying)?;
            self.play_handle_tx.send(Some(play_handle)).await?;
            loop {
                annoying_lights_on().await?;
                sleep(Duration::from_secs(2)).await;
                annoying_lights_off().await?;
                sleep(Duration::from_secs(2)).await;
            }
        } {
            log::error!("{:?}", e.context("Too loud failed"));
        };
    }

    pub async fn acceptable(self: Arc<Self>) {
        log::info!("Acceptable");
        if let Err::<(), anyhow::Error>(e) = try {
            // TODO: this should be a "back to normal" anouncement
            let play_handle = audio::play_file(&self.sound_files.too_loud_anouncement)?;
            let expect_done_at = play_handle.expect_done_at();
            self.play_handle_tx.send(Some(play_handle)).await?;
            sleep(expect_done_at - Instant::now()).await;

            let (a, b, c) = tokio::join!(
                async move {
                    if let Err(e) = self.spotify.resume_playback(None, None).await {
                        if e.to_string().contains("403") {
                            // 403 is returned by spotify when already playing back
                            log::warn!(
                                "{:?}",
                                anyhow::Error::from(e).context("Failed to resume playback")
                            );
                        } else {
                            return Err::<(), anyhow::Error>(e.into());
                        }
                    }
                    Ok(())
                },
                nice_lights_on(),
                annoying_lights_off(),
            );
            for r in [a, b, c] {
                if let Err(e) = r {
                    log::error!("{:?}", e.context("Acceptable failed"));
                }
            }
        } {
            log::error!("{:?}", e.context("Acceptable failed"));
        };
    }
    pub async fn too_quiet(self: Arc<Self>) {
        log::info!("Too quiet");
        if let Err::<(), anyhow::Error>(e) = try {
            let play_handle = audio::play_file(&self.sound_files.too_quiet_anouncement)?;
            let expect_done_at = play_handle.expect_done_at();
            self.play_handle_tx.send(Some(play_handle)).await?;
            sleep(expect_done_at - Instant::now()).await;

            let (a, b) = tokio::join!(nice_lights_off(), async move {
                if let Err(e) = self.spotify.pause_playback(None).await {
                    if e.to_string().contains("403") {
                        // 403 is returned by spotify when spotify is already paused
                        log::warn!(
                            "{:?}",
                            anyhow::Error::from(e).context("Failed to resume playback")
                        );
                    } else {
                        return Err::<(), anyhow::Error>(e.into());
                    }
                };
                Ok(())
            });
            for r in [a, b] {
                if let Err(e) = r {
                    log::error!("{:?}", e.context("Too quiet failed"));
                }
            }
        } {
            log::error!("{:?}", e.context("Too quiet failed"));
        };
    }
}

async fn annoying_lights_on() -> anyhow::Result<()> {
    const ON_URL: &str = env!("ANNOYING_LIGHTS_ON_URL");
    reqwest::get(ON_URL).await?;
    Ok(())
}

async fn annoying_lights_off() -> anyhow::Result<()> {
    const OFF_URL: &str = env!("ANNOYING_LIGHTS_OFF_URL");
    reqwest::get(OFF_URL).await?;
    Ok(())
}

async fn nice_lights_on() -> anyhow::Result<()> {
    const ON_URL: &str = env!("NICE_LIGHTS_ON_URL");
    reqwest::get(ON_URL).await?;
    Ok(())
}

async fn nice_lights_off() -> anyhow::Result<()> {
    const OFF_URL: &str = env!("NICE_LIGHTS_OFF_URL");
    reqwest::get(OFF_URL).await?;
    Ok(())
}
