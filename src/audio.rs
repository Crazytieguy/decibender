use std::{
    collections::VecDeque,
    fs::File,
    io::BufReader,
    sync::mpsc::{self, Sender},
    thread,
};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    BufferSize,
};
use rodio::{Decoder, OutputStream, Sink};
use tokio::sync::watch;

const BUFFER_SIZE: u32 = 1000;
const RMS_SECONDS: usize = 3;

pub fn watch_loudness(device_name: &str) -> anyhow::Result<watch::Receiver<f32>> {
    let host = cpal::default_host();
    let mic = host
        .input_devices()?
        .find(|d| d.name().is_ok_and(|name| name == device_name))
        .ok_or_else(|| anyhow::anyhow!("Device {device_name} not found"))?;
    let mut mic_config = mic.default_input_config()?.config();
    mic_config.buffer_size = BufferSize::Fixed(BUFFER_SIZE);

    let (tx, rx) = mpsc::channel::<f32>();

    let input_stream = mic.build_input_stream(
        &mic_config,
        move |data: &[f32], _| {
            debug_assert_eq!(data.len(), BUFFER_SIZE as usize);
            let mean_square = data
                .iter()
                .map(|x| x.powi(2) / data.len() as f32)
                .sum::<f32>();
            if tx.send(mean_square).is_err() {
                panic!("Failed to send mean square to watch receiver");
            }
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
        loop {
            let Ok(mean_square) = rx.recv() else {
                panic!("Failed to receive mean square from input stream");
            };
            mean_square_buffer.push_back(mean_square);
            if mean_square_buffer.len()
                > (mic_config.sample_rate.0 / BUFFER_SIZE) as usize * RMS_SECONDS
            {
                mean_square_buffer.pop_front();
            }
            let mean_square_avg =
                mean_square_buffer.iter().copied().sum::<f32>() / mean_square_buffer.len() as f32;
            let rms = mean_square_avg.sqrt().max(0.0).min(1.0);
            let decibels = 20.0 * rms.log10();
            watch_tx.send(decibels).ok();
        }
    });

    Ok(watch_rx)
}

pub fn play_file(file: &str) -> anyhow::Result<PlayHandle> {
    let file = BufReader::new(File::open(file)?);
    let source = Decoder::new(file)?;
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        sink.append(source);
        // we expect the sender to be dropped
        rx.recv().ok();
    });
    Ok(PlayHandle { _tx: tx })
}

/// To stop playback, drop the handle
pub struct PlayHandle {
    _tx: Sender<()>,
}
