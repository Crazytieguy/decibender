// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};

use decibender::{audio::PlayHandle, rules, thresholds::Thresholds};
use tauri::Window;

enum State {
    TooLoud,
    TooQuite,
    Acceptable,
}

#[tauri::command]
async fn init(window: Window, initial_thresholds: Thresholds) -> Result<(), String> {
    // let device_name = "ATR4697-USB";
    let device_name = "MacBook Pro Microphone";
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
            .emit("loudness", loudness)
            .map_err(|e| e.to_string())?;
        let thresholds = thresholds.lock().map_err(|e| e.to_string())?.clone();
        match state {
            State::Acceptable if thresholds.too_loud(loudness) => {
                play_handle = Some(rules::too_loud().await.map_err(|e| e.to_string())?);
                state = State::TooLoud;
            }
            State::Acceptable if thresholds.too_quite(loudness) => {
                rules::too_quite(&spotify)
                    .await
                    .map_err(|e| e.to_string())?;
                state = State::TooQuite;
            }
            State::TooLoud if thresholds.acceptable_from_too_loud(loudness) => {
                rules::acceptable(&spotify, play_handle.take())
                    .await
                    .map_err(|e| e.to_string())?;
                state = State::Acceptable;
            }
            State::TooQuite if thresholds.acceptable_from_too_quite(loudness) => {
                rules::acceptable(&spotify, play_handle.take())
                    .await
                    .map_err(|e| e.to_string())?;
                state = State::Acceptable;
            }
            _ => {}
        };
    }
}

fn main() {
    dotenvy::dotenv().ok();
    env_logger::init();
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![init])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
