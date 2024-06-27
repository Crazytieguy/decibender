use std::{fmt::Display, thread::sleep, time::Duration};

use inquire::Select;
use tokio::{sync::broadcast, task::spawn_blocking};

#[derive(Debug)]
pub struct Thresholds {
    too_loud: f32,
    too_quite: f32,
    grace: f32,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            too_loud: -20.0,
            too_quite: -80.0,
            grace: 5.0,
        }
    }
}

impl Thresholds {
    pub fn update(&mut self, command: Command) {
        match command {
            Command::IncreaseTooLoud => {
                self.too_loud += 1.0;
            }
            Command::DecreaseTooLoud => {
                self.too_loud -= 1.0;
            }
            Command::IncreaseTooQuite => {
                self.too_quite += 1.0;
            }
            Command::DecreaseTooQuite => {
                self.too_quite -= 1.0;
            }
            Command::IncreaseGrace => {
                self.grace += 1.0;
            }
            Command::DecreaseGrace => {
                self.grace -= 1.0;
            }
        }
    }

    pub fn too_loud_pred(&self) -> impl Fn(&f32) -> bool + '_ {
        move |&loudness| loudness > self.too_loud
    }

    pub fn too_quite_pred(&self) -> impl Fn(&f32) -> bool + '_ {
        move |&loudness| loudness < self.too_quite
    }

    pub fn acceptable_from_too_loud_pred(&self) -> impl Fn(&f32) -> bool + '_ {
        move |&loudness| loudness < self.too_loud - self.grace
    }

    pub fn acceptable_from_too_quite_pred(&self) -> impl Fn(&f32) -> bool + '_ {
        move |&loudness| loudness > self.too_quite + self.grace
    }
}

#[derive(Clone, Debug)]
pub enum Command {
    IncreaseTooLoud,
    DecreaseTooLoud,
    IncreaseTooQuite,
    DecreaseTooQuite,
    IncreaseGrace,
    DecreaseGrace,
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::IncreaseTooLoud => write!(f, "Increase too loud"),
            Command::DecreaseTooLoud => write!(f, "Decrease too loud"),
            Command::IncreaseTooQuite => write!(f, "Increase too quite"),
            Command::DecreaseTooQuite => write!(f, "Decrease too quite"),
            Command::IncreaseGrace => write!(f, "Increase grace"),
            Command::DecreaseGrace => write!(f, "Decrease grace"),
        }
    }
}

pub fn recieve_command() -> broadcast::Receiver<Command> {
    let (tx, rx) = broadcast::channel(32);
    let options = vec![
        Command::IncreaseTooLoud,
        Command::DecreaseTooLoud,
        Command::IncreaseTooQuite,
        Command::DecreaseTooQuite,
        Command::IncreaseGrace,
        Command::DecreaseGrace,
    ];
    spawn_blocking(move || {
        sleep(Duration::from_millis(500));
        loop {
            let control = Select::new("Command:", options.clone()).prompt().unwrap();
            tx.send(control).unwrap();
        }
    });
    rx
}
