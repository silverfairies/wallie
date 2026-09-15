use rand::rngs::SmallRng;
use sysinfo::{ProcessRefreshKind, RefreshKind, System};
use wallie_lib::Info;

use crate::{
    dataset::{Heap, Simple, WallLoader},
    renderers::Renderer,
};

use std::{
    env::{args, home_dir},
    fs::read_to_string,
    io::Error,
    ops::Sub,
    path::PathBuf,
    str::FromStr,
    time::{Duration, Instant},
};

pub struct Manager {
    generator: SmallRng,
    last_change: State,
    current_wallpaper: Option<PathBuf>,
    mode: Box<dyn WallLoader>,
    sleep: Duration,
    renderer: Renderer,
    process_list: System,
}

#[derive(Clone, Copy)]
enum State {
    Playing(Instant),
    Paused(Duration),
}

impl State {
    pub fn pause(self) -> Self {
        if let State::Playing(instant) = self {
            Self::Paused(instant.elapsed())
        } else {
            self
        }
    }

    pub fn resume(self) -> Self {
        if let State::Paused(duration) = self {
            Self::Playing(Instant::now().sub(duration))
        } else {
            self
        }
    }
}

impl Manager {
    pub fn new_from_args() -> Result<Self, Error> {
        let (mode, sleep, renderer) = Self::parse_arguments()?;
        eprintln!("Constructs a new manager");
        Ok(Self {
            mode,
            sleep,
            renderer,
            ..Default::default()
        })
    }

    pub fn run(&mut self) -> Result<(), Error> {
        eprintln!("Manager initialization");
        self.renderer = self.renderer.clone().auto(&mut self.process_list);
        let picture = &self.mode.next(&mut self.generator)?;
        let _ = self.renderer.spawn(picture, &mut self.process_list)?;
        Ok(())
    }

    fn parse_arguments() -> Result<(Box<dyn WallLoader>, Duration, Renderer), Error> {
        let arguments = args().collect::<Vec<String>>();
        let timing = if arguments.contains(&"-d".to_string()) {
            Duration::from_secs(
                u64::from_str(
                    arguments[arguments
                        .iter()
                        .position(|entry| entry == &"-d".to_string())
                        .unwrap()
                        + 1]
                    .as_str(),
                )
                .expect("Invalid sleep duration!"),
            )
        } else {
            Duration::from_secs(300)
        };
        let renderer = if arguments.contains(&"-r".to_string()) {
            match arguments[arguments
                .iter()
                .position(|entry| entry == &"-r".to_string())
                .unwrap()
                + 1]
            .as_str()
            {
                "awww" => Renderer::Awww,
                "swaybg" => Renderer::Swaybg,
                "auto" => Renderer::Auto,
                renderer => {
                    eprintln!("Invalid renderer: {}", renderer);
                    Renderer::Auto
                }
            }
        } else {
            Renderer::Auto
        };

        let mode: Box<dyn WallLoader> = if arguments.contains(&"--simple".to_string()) {
            Box::new(Simple::from_directory(PathBuf::from(
                arguments.last().expect("No directory provided!"),
            ))?)
        } else {
            eprintln!("Heap");
            Box::new(Heap::generate(
                ron::from_str(
                    read_to_string(home_dir().unwrap().join(".config/wallie/database.ron"))?
                        .as_str(),
                )
                .unwrap_or_default(),
            ))
        };
        Ok((mode, timing, renderer))
    }

    fn next(&mut self) -> Result<(), Error> {
        self.mode.reload_static()?;
        let new_wallpaper = self.mode.next(&mut self.generator)?;
        let exit = self.renderer.change(&new_wallpaper, &mut self.process_list);
        if let Err(e) = exit {
            eprintln!("Some error while changing background happened: {}", e);
        } else if let Ok(e) = exit {
            if !e.success() {
                eprintln!(
                    "Some error while changing background happened: {:#?}",
                    e.code()
                );
            } else {
                self.current_wallpaper = Some(new_wallpaper);
            }
        }
        if let State::Playing(_) = self.last_change {
            self.last_change = State::Playing(Instant::now());
        }
        Ok(())
    }

    pub fn init_pictures(&mut self) -> Result<(), Error> {
        self.mode.reload()
    }

    pub fn next_picture(&mut self, instant: bool) -> Result<(), Error> {
        if instant {
            self.next()
        } else if let State::Playing(instant) = self.last_change {
            if !instant.elapsed().saturating_sub(self.sleep).is_zero() {
                self.next()
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    pub fn get_info(&self) -> Info {
        let time_elapsed = if let State::Playing(instant) = self.last_change {
            instant.elapsed()
        } else if let State::Paused(duration) = self.last_change {
            duration
        } else {
            Duration::ZERO
        };
        Info {
            current_wallpaper: self.current_wallpaper.clone(),
            duration: self.sleep,
            time_left: self.sleep.saturating_sub(time_elapsed),
            playing: matches!(self.last_change, State::Playing(_)),
        }
    }

    pub fn pause(&mut self) {
        self.last_change = self.last_change.pause();
    }

    pub fn resume(&mut self) {
        self.last_change = self.last_change.resume();
    }

    pub fn toggle(&mut self) {
        self.last_change = if let State::Playing(_) = self.last_change {
            self.last_change.pause()
        } else {
            self.last_change.resume()
        }
    }
}

impl Default for Manager {
    fn default() -> Self {
        Self {
            generator: rand::make_rng(),
            last_change: State::Playing(Instant::now()),
            current_wallpaper: None,
            mode: Box::new(Heap::default()),
            sleep: Duration::from_secs(300),
            renderer: Renderer::Auto,
            process_list: System::new_with_specifics(
                RefreshKind::nothing().with_processes(ProcessRefreshKind::nothing()),
            ),
        }
    }
}
