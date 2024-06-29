#![warn(clippy::pedantic)]
// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    sync::OnceLock,
    time::{Duration, Instant},
};

use decibender::{
    audio::{self},
    rules::RuleExecutor,
    thresholds::Thresholds,
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tokio::{
    sync::{broadcast, watch},
    task::JoinHandle,
};

#[derive(Clone, Serialize)]
enum State {
    TooLoud,
    TooQuiet,
    Acceptable,
}

#[derive(Serialize)]
struct AppError(String);

impl<T: ToString> From<T> for AppError {
    fn from(e: T) -> Self {
        Self(e.to_string())
    }
}

#[derive(Serialize, Clone, Copy)]
struct Loudness {
    loudness: f32,
}

#[derive(Deserialize, Clone)]
struct RmsSeconds {
    rms_seconds: f32,
}

static INITIALIZED: OnceLock<()> = OnceLock::new();

#[tauri::command]
async fn init(
    app_handle: AppHandle,
    initial_rms_seconds: f32,
    initial_thresholds: Thresholds,
) -> Result<(), AppError> {
    if let Err(_) = INITIALIZED.set(()) {
        // We've already initialized
        return Ok(());
    }
    log::info!("Initializing");

    let rule_executor = RuleExecutor::new(&app_handle).await?;

    app_handle.emit_all("thresholds", initial_thresholds)?;
    tokio::spawn(rule_executor.clone().adjust_volume(initial_thresholds));

    let mut state = State::Acceptable;
    let mut end_grace_period_at = std::time::Instant::now();
    let mut current_task: Option<JoinHandle<()>> = None;
    let mut set_current_task = |task| {
        if let Some(task) = current_task.take() {
            task.abort();
        }
        current_task = Some(task);
    };

    let (louder_tx, mut louder_rx) = broadcast::channel::<()>(4);
    let (quieter_tx, mut quieter_rx) = broadcast::channel::<()>(4);
    let (thresholds_tx, mut thresholds_rx) = watch::channel::<Thresholds>(initial_thresholds);
    let (rms_seconds_tx, rms_seconds) = watch::channel::<f32>(initial_rms_seconds);
    let mut loudness_rx = audio::watch_loudness(rms_seconds)?;

    app_handle.listen_global("louder", move |_event| {
        louder_tx.send(()).ok();
    });

    app_handle.listen_global("quieter", move |_event| {
        quieter_tx.send(()).ok();
    });

    app_handle.listen_global("rms-seconds", move |event| {
        let Some(payload) = event.payload() else {
            log::error!("No payload in rms_seconds event");
            return;
        };
        let Ok(RmsSeconds { rms_seconds }) = serde_json::from_str(payload) else {
            log::error!("Failed to parse rms_seconds payload: {}", payload);
            return;
        };
        log::info!("Updating rms_seconds: {}", rms_seconds);
        rms_seconds_tx.send(rms_seconds).ok();
    });
    app_handle.listen_global("thresholds", move |event| {
        let Some(payload) = event.payload() else {
            log::error!("No payload in thresholds event");
            return;
        };
        let Ok(thresholds) = serde_json::from_str(payload) else {
            log::error!("Failed to parse thresholds payload: {}", payload);
            return;
        };
        log::info!("Updating thresholds: {:?}", thresholds);
        thresholds_tx.send(thresholds).ok();
    });

    loop {
        tokio::select! {
            _ = louder_rx.recv() => {
                end_grace_period_at = Instant::now() + Duration::from_secs(7);
                tokio::spawn(rule_executor.clone().louder());
                continue;
            }
            _ = quieter_rx.recv() => {
                end_grace_period_at = Instant::now() + Duration::from_secs(7);
                tokio::spawn(rule_executor.clone().quieter());
                continue;
            }
            _ = thresholds_rx.changed() => {
                let thresholds = *thresholds_rx.borrow_and_update();
                app_handle.emit_all("thresholds", thresholds)?;
                tokio::spawn(rule_executor.clone().adjust_volume(thresholds));
            }
            _ = loudness_rx.changed() => {}
        };
        let thresholds = thresholds_rx.borrow();
        let loudness = *loudness_rx.borrow_and_update();
        app_handle.emit_all("loudness", Loudness { loudness })?;
        if end_grace_period_at > Instant::now() {
            continue;
        }
        match state {
            State::Acceptable if thresholds.too_loud(loudness) => {
                set_current_task(tokio::spawn(rule_executor.clone().too_loud()));
                state = State::TooLoud;
            }
            State::Acceptable if thresholds.too_quiet(loudness) => {
                set_current_task(tokio::spawn(rule_executor.clone().too_quiet()));
                state = State::TooQuiet;
            }
            State::TooLoud if thresholds.acceptable_from_too_loud(loudness) => {
                set_current_task(tokio::spawn(rule_executor.clone().acceptable()));
                state = State::Acceptable;
            }
            State::TooQuiet if thresholds.acceptable_from_too_quiet(loudness) => {
                set_current_task(tokio::spawn(rule_executor.clone().acceptable()));
                state = State::Acceptable;
            }
            _ => continue,
        };
        app_handle.emit_all("state", state.clone())?;
    }
}

fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("warn"));
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![init])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
