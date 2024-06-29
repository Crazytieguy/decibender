use std::{
    collections::VecDeque,
    f32::consts::PI,
    fs::File,
    io::BufReader,
    path::PathBuf,
    sync::mpsc::{self, Sender},
    thread,
    time::Instant,
};

use anyhow::Context;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    BufferSize,
};
use rodio::{Decoder, OutputStream, Sink, Source};
use tokio::sync::watch;

const BUFFER_SIZE: u32 = 4000;

pub fn watch_loudness(
    mut rms_seconds: watch::Receiver<f32>,
) -> anyhow::Result<watch::Receiver<f32>> {
    let device_name = env!("INPUT_DEVICE");
    let host = cpal::default_host();
    let mic = host
        .input_devices()?
        .find(|d| d.name().is_ok_and(|name| name == device_name))
        .ok_or_else(|| anyhow::anyhow!("Device {device_name} not found"))?;
    let mut mic_config = mic.default_input_config()?.config();
    mic_config.buffer_size = BufferSize::Fixed(BUFFER_SIZE);

    let (tx, rx) = mpsc::channel::<f32>();
    let mut filter = Filter {
        mode: FilterMode::HighPass,
        cutoff: 100.0,
        resonance: 0.0,
        ic1eq: 0.0,
        ic2eq: 0.0,
        sample_rate: mic_config.sample_rate.0 as f32,
    };

    let input_stream = mic.build_input_stream(
        &mic_config,
        move |data: &[f32], _| {
            debug_assert_eq!(data.len(), BUFFER_SIZE as usize);
            let filtered = data.iter().map(|&x| filter.process(x));
            let mean_square = filtered.map(|x| x.powi(2) / data.len() as f32).sum::<f32>();
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

    thread::spawn(move || {
        let mut mean_square_buffer = VecDeque::new();
        loop {
            let Ok(mean_square) = rx.recv() else {
                panic!("Failed to receive mean square from input stream");
            };
            mean_square_buffer.push_back(mean_square);
            let target_len = (mic_config.sample_rate.0 as f32 / BUFFER_SIZE as f32
                * *rms_seconds.borrow_and_update())
            .round() as usize;
            if mean_square_buffer.len() > target_len {
                mean_square_buffer.drain(..mean_square_buffer.len() - target_len);
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

#[allow(dead_code)]
enum FilterMode {
    /// Removes frequencies above the cutoff frequency.
    LowPass,
    /// Removes frequencies above and below the cutoff frequency.
    BandPass,
    /// Removes frequencies below the cutoff frequency.
    HighPass,
    /// Removes frequencies around the cutoff frequency.
    Notch,
}

struct Filter {
    mode: FilterMode,
    cutoff: f32,
    resonance: f32,
    ic1eq: f32,
    ic2eq: f32,
    sample_rate: f32,
}

impl Filter {
    // copied from https://github.com/tesselode/kira/blob/main/crates/kira/src/effect/filter.rs
    fn process(&mut self, sample: f32) -> f32 {
        let g = (PI * (self.cutoff / self.sample_rate)).tan();
        let k = 2.0 - (1.9 * self.resonance.min(1.0).max(0.0));
        let a1 = 1.0 / (1.0 + (g * (g + k)));
        let a2 = g * a1;
        let a3 = g * a2;
        let v3 = sample - self.ic2eq;
        let v1 = (self.ic1eq * (a1 as f32)) + (v3 * (a2 as f32));
        let v2 = self.ic2eq + (self.ic1eq * (a2 as f32)) + (v3 * (a3 as f32));
        self.ic1eq = (v1 * 2.0) - self.ic1eq;
        self.ic2eq = (v2 * 2.0) - self.ic2eq;
        match self.mode {
            FilterMode::LowPass => v2,
            FilterMode::BandPass => v1,
            FilterMode::HighPass => sample - v1 * (k as f32) - v2,
            FilterMode::Notch => sample - v1 * (k as f32),
        }
    }
}

pub fn play_file(file_path: &PathBuf) -> anyhow::Result<PlayHandle> {
    let file = BufReader::new(
        File::open(file_path).with_context(|| format!("Failed to open file {file_path:?}"))?,
    );
    let source =
        Decoder::new(file).with_context(|| format!("Failed to decode file {file_path:?}"))?;
    let total_duration = source
        .total_duration()
        .ok_or_else(|| anyhow::anyhow!("Failed to get total duration for file {file_path:?}"))?;
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let Ok((_stream, stream_handle)) = OutputStream::try_default() else {
            log::error!("Failed to get default output stream");
            return;
        };
        let Ok(sink) = Sink::try_new(&stream_handle) else {
            log::error!("Failed to get sink from stream handle");
            return;
        };
        sink.append(source);
        // we expect the sender to be dropped
        rx.recv_timeout(total_duration).ok();
    });
    Ok(PlayHandle {
        expect_done_at: Instant::now() + total_duration,
        _tx: tx,
    })
}

/// To stop playback, drop the handle
pub struct PlayHandle {
    expect_done_at: Instant,
    _tx: Sender<()>,
}

impl PlayHandle {
    pub fn expect_done_at(&self) -> Instant {
        self.expect_done_at
    }
}
