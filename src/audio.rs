//! Audio output lives on one worker. Device creation, file opening and seeking
//! cannot interrupt controller navigation or the renderer.
use crate::message;
use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player};
use std::{path::PathBuf, sync::mpsc, time::Duration};

pub enum Command {
    Open(PathBuf),
    Toggle,
    Seek(f64),
    Volume(f32),
    Stop,
}
#[derive(Default, Clone)]
pub struct Status {
    pub position: f64,
    pub playing: bool,
    pub ended: bool,
    pub error: Option<String>,
}
pub struct Audio {
    tx: mpsc::Sender<(u64, Command)>,
    rx: mpsc::Receiver<(u64, Status)>,
    pub status: Status,
    generation: u64,
}
impl Audio {
    pub fn new() -> Self {
        let (tx, commands) = mpsc::channel();
        let (reports, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let mut device: Option<MixerDeviceSink> = None;
            let mut player: Option<Player> = None;
            let mut volume = 0.75;
            let mut status = Status::default();
            let mut generation = 0;
            loop {
                match commands.recv_timeout(Duration::from_millis(100)) {
                    Ok((next_generation, command)) => {
                        generation = next_generation;
                        match command {
                            Command::Open(path) => {
                                player = None;
                                status = Status::default();
                                let result = (|| -> Result<Player, String> {
                                    if device.is_none() {
                                        let mut opened = DeviceSinkBuilder::open_default_sink()
                                            .map_err(|error| {
                                                message!("error-device", "error" => error.to_string())
                                            })?;
                                        opened.log_on_drop(false);
                                        device = Some(opened);
                                    }
                                    let file = std::fs::File::open(&path).map_err(|error| {
                                        message!(
                                            "error-path",
                                            "name" => path.display().to_string(),
                                            "error" => error.to_string(),
                                        )
                                    })?;
                                    let decoder = Decoder::try_from(file).map_err(|error| {
                                        message!(
                                            "error-open",
                                            "name" => path.display().to_string(),
                                            "error" => error.to_string(),
                                        )
                                    })?;
                                    let sink =
                                        Player::connect_new(device.as_ref().unwrap().mixer());
                                    sink.set_volume(volume);
                                    sink.append(decoder);
                                    Ok(sink)
                                })();
                                match result {
                                    Ok(p) => player = Some(p),
                                    Err(e) => status.error = Some(e),
                                }
                            }
                            Command::Toggle => {
                                if let Some(p) = &player {
                                    if p.is_paused() {
                                        p.play();
                                    } else {
                                        p.pause();
                                    }
                                }
                            }
                            Command::Seek(at) => {
                                if let Some(p) = &player {
                                    if let Err(error) =
                                        p.try_seek(Duration::from_secs_f64(at.max(0.0)))
                                    {
                                        status.error = Some(
                                            message!("error-seek", "error" => error.to_string()),
                                        );
                                    }
                                }
                            }
                            Command::Volume(v) => {
                                volume = v;
                                if let Some(p) = &player {
                                    p.set_volume(v);
                                }
                            }
                            Command::Stop => {
                                player = None;
                                status = Status::default();
                            }
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
                if let Some(p) = &player {
                    status.position = p.get_pos().as_secs_f64();
                    status.playing = !p.is_paused() && !p.empty();
                    status.ended = p.empty();
                }
                if reports.send((generation, status.clone())).is_err() {
                    break;
                }
            }
        });
        Self {
            tx,
            rx,
            status: Status::default(),
            generation: 0,
        }
    }
    pub fn send(&mut self, command: Command) {
        if matches!(command, Command::Open(_) | Command::Stop) {
            // Reports carry a generation: a late end-of-track event cannot
            // advance the queue again after another song has already opened.
            self.generation = self.generation.wrapping_add(1);
            while self.rx.try_recv().is_ok() {}
            self.status = Status::default();
        }
        let _ = self.tx.send((self.generation, command));
    }
    pub fn poll(&mut self) {
        while let Ok((generation, status)) = self.rx.try_recv() {
            if generation == self.generation {
                self.status = status;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn late_reports_from_previous_song_cannot_end_the_new_song() {
        let (tx, _commands) = mpsc::channel();
        let (reports, rx) = mpsc::channel();
        let mut audio = Audio {
            tx,
            rx,
            status: Status::default(),
            generation: 1,
        };
        reports
            .send((
                0,
                Status {
                    ended: true,
                    ..Status::default()
                },
            ))
            .unwrap();
        audio.poll();
        assert!(!audio.status.ended);
        reports
            .send((
                1,
                Status {
                    playing: true,
                    position: 2.0,
                    ..Status::default()
                },
            ))
            .unwrap();
        audio.poll();
        assert!(audio.status.playing);
        assert_eq!(audio.status.position, 2.0);
    }
}
