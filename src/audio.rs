use std::{collections::VecDeque, sync::mpsc};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    BufferSize,
};
use tokio::sync::watch;

const BUFFER_SIZE: u32 = 1000;
const RMS_SECONDS: usize = 2;

pub fn watch_loudness(device_name: &str) -> anyhow::Result<watch::Receiver<f32>> {
    let host = cpal::default_host();
    let usb_mic = host
        .input_devices()?
        .find(|d| d.name().is_ok_and(|name| name == device_name))
        .ok_or_else(|| anyhow::anyhow!("Device {device_name} not found"))?;
    let mut usb_mic_config = usb_mic.default_input_config()?.config();
    usb_mic_config.buffer_size = BufferSize::Fixed(BUFFER_SIZE);

    let (tx, rx) = mpsc::channel::<f32>();

    let input_stream = usb_mic.build_input_stream(
        &usb_mic_config,
        move |data: &[f32], _| {
            debug_assert_eq!(data.len(), BUFFER_SIZE as usize);
            let mean_square = data
                .iter()
                .map(|x| x.powi(2) / data.len() as f32)
                .sum::<f32>();
            tx.send(mean_square).unwrap();
        },
        |err| {
            eprintln!("An error occurred on the input stream: {}", err);
        },
        None,
    )?;
    input_stream.play()?;

    let (watch_tx, watch_rx) = watch::channel(-60.0);

    std::thread::spawn(move || {
        let mut mean_square_buffer = VecDeque::new();
        while let Ok(mean_square) = rx.recv() {
            mean_square_buffer.push_back(mean_square);
            if mean_square_buffer.len()
                > (usb_mic_config.sample_rate.0 / BUFFER_SIZE) as usize * RMS_SECONDS
            {
                mean_square_buffer.pop_front();
            }
            let mean_square_avg =
                mean_square_buffer.iter().copied().sum::<f32>() / mean_square_buffer.len() as f32;
            let rms = mean_square_avg.sqrt().max(0.0).min(1.0);
            let decibels = 20.0 * rms.log10();
            watch_tx.send(decibels).unwrap();
        }
    });

    Ok(watch_rx)
}

