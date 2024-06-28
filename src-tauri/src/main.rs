// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex, OnceLock};

use decibender::{audio::PlayHandle, rules, thresholds::Thresholds};
use serde::Serialize;
use tauri::{AppHandle, Window};

#[derive(Clone, Serialize)]
enum State {
    TooLoud,
    TooQuite,
    Acceptable,
}

#[derive(Serialize, Clone, Copy)]
struct Loudness {
    loudness: f32,
}

static INITIALIZED: OnceLock<()> = OnceLock::new();

#[tauri::command]
async fn init(
    window: Window,
    app_handle: AppHandle,
    initial_thresholds: Thresholds,
) -> Result<(), String> {
    if let Err(_) = INITIALIZED.set(()) {
        // We've already initialized
        return Ok(());
    }
    log::info!("Initializing");
    // let device_name = "ATR4697-USB";
    let device_name = "MacBook Pro Microphone";
    let annoying_file = app_handle
        .path_resolver()
        .resolve_resource(env!("ANNOYING_FILE"))
        .ok_or_else(|| "Failed to resolve annoying file".to_string())?;
    let spotify = decibender::spotify::init()
        .await
        .map_err(|e| e.to_string())?;
    let thresholds = Arc::new(Mutex::new(initial_thresholds));
    let thresholds_listener = thresholds.clone();
    window.listen("thresholds", move |event| {
        let Some(payload) = event.payload() else {
            log::error!("No payload in thresholds event");
            return;
        };
        let Ok(new_thresholds) = serde_json::from_str(payload) else {
            log::error!("Failed to parse thresholds payload: {}", payload);
            return;
        };
        log::info!("Updating thresholds: {:?}", new_thresholds);
        *thresholds_listener.lock().unwrap() = new_thresholds;
    });
    let mut loudness_rx =
        decibender::audio::watch_loudness(device_name).map_err(|e| e.to_string())?;
    let mut state = State::Acceptable;
    let mut play_handle: Option<PlayHandle> = None;
    loop {
        loudness_rx.changed().await.map_err(|e| e.to_string())?;
        let loudness = *loudness_rx.borrow_and_update();
        window
            .emit("loudness", Loudness { loudness })
            .map_err(|e| e.to_string())?;
        let thresholds = thresholds.lock().map_err(|e| e.to_string())?.clone();
        match state {
            State::Acceptable if thresholds.too_loud(loudness) => {
                log::info!("Too loud");
                play_handle = Some(
                    rules::too_loud(&annoying_file)
                        .await
                        .map_err(|e| e.to_string())?,
                );
                state = State::TooLoud;
            }
            State::Acceptable if thresholds.too_quite(loudness) => {
                log::info!("Too quite");
                rules::too_quite(&spotify)
                    .await
                    .map_err(|e| e.to_string())?;
                state = State::TooQuite;
            }
            State::TooLoud if thresholds.acceptable_from_too_loud(loudness) => {
                log::info!("Acceptable");
                rules::acceptable(&spotify, play_handle.take())
                    .await
                    .map_err(|e| e.to_string())?;
                state = State::Acceptable;
            }
            State::TooQuite if thresholds.acceptable_from_too_quite(loudness) => {
                log::info!("Acceptable");
                rules::acceptable(&spotify, play_handle.take())
                    .await
                    .map_err(|e| e.to_string())?;
                state = State::Acceptable;
            }
            _ => continue,
        };
        window
            .emit("state", state.clone())
            .map_err(|e| e.to_string())?;
    }
}

fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("warn"));
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![init])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
