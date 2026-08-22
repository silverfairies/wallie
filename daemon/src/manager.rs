use rand::{
    distr::{Distribution, weighted::WeightedIndex},
    rngs::SmallRng,
};
use sysinfo::{ProcessRefreshKind, RefreshKind, System};
use wallie_lib::Info;

use crate::{dataset::Heap, renderers::Renderer};

use std::{
    env::args,
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
    mode: Mode,
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

#[derive(Clone)]
enum Mode {
    Simple {
        wallpaper_directories: Vec<PathBuf>,
        wallpapers: Vec<PathBuf>,
        distribution: WeightedIndex<u32>,
    },
    Heap {
        heap: Heap,
    },
}

impl Mode {
    fn init_pictures(&mut self) -> Result<(), Error> {
        match self {
            Mode::Simple {
                wallpaper_directories,
                wallpapers,
                distribution,
            } => {
                wallpapers.clear();
                let mut distr = Vec::new();
                for directory in wallpaper_directories.clone() {
                    let (mut picbuf, mut distbuf) = Self::parse_directory_recursive(directory)?;
                    wallpapers.append(&mut picbuf);
                    distr.append(&mut distbuf);
                }
                *distribution = WeightedIndex::new(distr.iter()).unwrap();
            }
            Mode::Heap { .. } => {
                //TODO: Heap logic
            }
        }
        Ok(())
    }

    fn parse_directory_recursive(directory: PathBuf) -> Result<(Vec<PathBuf>, Vec<u32>), Error> {
        let entries = directory.read_dir()?;
        let mut pictures = Vec::<PathBuf>::new();
        let mut distribution = Vec::<u32>::new();
        for entry in entries {
            let picture = entry?.path();
            if picture.is_file() {
                pictures.push(picture);
                distribution.push(1);
            } else {
                let (mut picbuf, mut distbuf) = Self::parse_directory_recursive(picture)?;
                pictures.append(&mut picbuf);
                distribution.append(&mut distbuf);
            }
        }
        Ok((pictures, distribution))
    }

    fn random_picture(&mut self, generator: &mut SmallRng) -> Result<PathBuf, Error> {
        match self {
            Self::Simple {
                wallpapers,
                distribution,
                ..
            } => {
                let new_wallpaper = wallpapers[distribution.sample(generator)].to_owned();
                if !new_wallpaper.is_file() {
                    eprintln!(
                        "Some files were deleted or moved. Not found: {}",
                        new_wallpaper.to_str().unwrap()
                    );
                    self.init_pictures()?;
                    self.random_picture(generator)
                } else {
                    Ok(new_wallpaper)
                }
            }
            Self::Heap { heap } => Ok(heap.next(generator)?.path),
        }
    }
}

impl Manager {
    pub fn new_from_args() -> Result<Self, Error> {
        let (mode, sleep, renderer) = Self::parse_arguments()?;
        Ok(Self {
            mode,
            sleep,
            renderer,
            ..Default::default()
        })
    }

    pub fn run(&mut self) -> Result<(), Error> {
        self.renderer = self.renderer.clone().auto(&mut self.process_list);
        let picture = &self.random_picture()?;
        let _ = self.renderer.spawn(picture, &mut self.process_list)?;
        Ok(())
    }

    fn parse_arguments() -> Result<(Mode, Duration, Renderer), Error> {
        let arguments = args().collect::<Vec<String>>();
        if arguments.contains(&"--simple".to_string()) {
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
                        Renderer::Awww
                    }
                }
            } else {
                Renderer::Awww
            };
            Ok((
                Mode::Simple {
                    wallpaper_directories: vec![PathBuf::from(
                        arguments.last().expect("No directory provided!"),
                    )],
                    wallpapers: Vec::new(),
                    distribution: WeightedIndex::new([1_u32].iter()).unwrap(),
                },
                timing,
                renderer,
            ))
        } else {
            Err(Error::new(
                std::io::ErrorKind::InvalidInput,
                "--simple not specified, exiting",
            ))
        }
    }

    pub fn init_pictures(&mut self) -> Result<(), Error> {
        self.mode.init_pictures()
    }

    fn next(&mut self) -> Result<(), Error> {
        let new_wallpaper = self.random_picture()?;
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

    fn random_picture(&mut self) -> Result<PathBuf, Error> {
        self.mode.random_picture(&mut self.generator)
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
            mode: Mode::Heap { heap: Heap::default() },
            sleep: Duration::from_secs(300),
            renderer: Renderer::Auto,
            process_list: System::new_with_specifics(
                RefreshKind::nothing().with_processes(ProcessRefreshKind::nothing()),
            ),
        }
    }
}
