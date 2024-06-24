#![warn(clippy::pedantic)]

use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use itertools::Itertools;

const RECORD_SECONDS: usize = 5;

fn main() -> anyhow::Result<()> {
    let host = cpal::default_host();
    let usb_mic = host
        .input_devices()?
        .find(|d| d.name().is_ok_and(|name| name == "ATR4697-USB"))
        .ok_or_else(|| anyhow::anyhow!("No USB mic found"))?;
    let usb_mic_config = usb_mic.default_input_config()?.config();
    let output_device = host
        .output_devices()?
        .exactly_one()
        .map_err(|_| anyhow::anyhow!("Not exactly one output device"))?;
    let output_config = output_device.default_output_config()?.config();

    static INPUTTING: AtomicBool = AtomicBool::new(true);
    let buffer = Box::leak(Box::new(Mutex::new(VecDeque::<f32>::with_capacity(
        48000 * RECORD_SECONDS,
    ))));

    let input_stream = usb_mic.build_input_stream(
        &usb_mic_config,
        |data: &[f32], _: &_| {
            if INPUTTING.load(Ordering::Relaxed) {
                let mut buffer = buffer.lock().unwrap();
                if buffer.len() + data.len() > buffer.capacity() {
                    INPUTTING.store(false, Ordering::Relaxed);
                    println!("Recording done");
                } else {
                    buffer.extend(data.iter().copied());
                }
            }
        },
        |err| {
            eprintln!("An error occurred on the input stream: {}", err);
        },
        None,
    )?;
    input_stream.play()?;
    let output_stream = output_device.build_output_stream(
        &output_config,
        |data: &mut [f32], _| {
            if !INPUTTING.load(Ordering::Relaxed) {
                let mut buffer = buffer.lock().unwrap();
                if buffer.len() < data.len() {
                    INPUTTING.store(true, Ordering::Relaxed);
                    println!("Playback done");
                } else {
                    data.iter_mut().for_each(|d| {
                        *d = buffer.pop_front().unwrap();
                    });
                }
            }
        },
        |err| {
            eprintln!("An error occurred on the output stream: {}", err);
        },
        None,
    )?;
    output_stream.play()?;
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
