#![warn(clippy::pedantic)]

use decibender::audio::PlayHandle;
use indicatif::ProgressBar;
use tokio::spawn;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();
    // let device_name = "ATR4697-USB";
    let device_name = "MacBook Pro Microphone";
    let spotify = decibender::spotify::init().await?;
    let mut control_rx = decibender::controls::recieve_command();
    let mut loudness_rx = decibender::audio::watch_loudness(device_name)?;
    let mut loudness_rx_2 = loudness_rx.clone();
    let mut loudness_rx_3 = loudness_rx.clone();
    let mut loudness_rx_4 = loudness_rx.clone();
    let mut loudness_rx_pbar = loudness_rx.clone();
    let mut thresholds = decibender::controls::Thresholds::default();
    let mut state = State::Acceptable;
    let mut play_handle: Option<PlayHandle> = None;
    let loudness_bar = ProgressBar::new(100);
    spawn(async move {
        loop {
            loudness_rx_pbar.changed().await.unwrap();
            let loudness = *loudness_rx_pbar.borrow_and_update();
            loudness_bar.set_position((loudness + 100.0) as u64);
        }
    });
    println!("Tresholds: {thresholds:?}");
    loop {
        tokio::select! {
            res = loudness_rx.wait_for(thresholds.too_loud_pred()), if matches!(state, State::Acceptable) => {
                res?;
                play_handle = Some(decibender::rules::too_loud().await?);
                state = State::TooLoud;
            },
            res = loudness_rx_2.wait_for(thresholds.too_quite_pred()), if matches!(state, State::Acceptable) => {
                res?;
                decibender::rules::too_quite(&spotify).await?;
                state = State::TooQuite;
            },
            res = loudness_rx_3.wait_for(thresholds.acceptable_from_too_loud_pred()), if matches!(state, State::TooLoud) => {
                res?;
                decibender::rules::acceptable(&spotify, play_handle.take()).await?;
                state = State::Acceptable;
            },
            res = loudness_rx_4.wait_for(thresholds.acceptable_from_too_quite_pred()), if matches!(state, State::TooQuite) => {
                res?;
                decibender::rules::acceptable(&spotify, play_handle.take()).await?;
                state = State::Acceptable;
            },
            control_command = control_rx.recv() => {
                thresholds.update(control_command?);
                println!("Tresholds: {thresholds:?}");
            }
        }
    }
}

enum State {
    TooLoud,
    TooQuite,
    Acceptable,
}
