use rand::{
    distr::{Distribution, weighted::WeightedIndex},
    rngs::SmallRng,
};
use serde::Serialize;
use sysinfo::{ProcessRefreshKind, RefreshKind, System};

use crate::renderers::Renderer;

use std::{
    env::args,
    io::Error,
    path::PathBuf,
    str::FromStr,
    time::{Duration, Instant},
};

pub struct Manager {
    generator: SmallRng,
    last_change: Instant,
    current_wallpaper: Option<PathBuf>,
    wallpaper_directories: Vec<PathBuf>,
    wallpapers: Vec<PathBuf>,
    distribution: WeightedIndex<u32>,
    sleep: Duration,
    renderer: Renderer,
    process_list: System,
}

impl Manager {
    pub fn new_from_args() -> Result<Self, Error> {
        let (wallpaper_directories, sleep, renderer) = Self::parse_arguments()?;
        Ok(Self {
            wallpaper_directories,
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

    fn parse_arguments() -> Result<(Vec<PathBuf>, Duration, Renderer), Error> {
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
                vec![PathBuf::from(
                    arguments.last().expect("No directory provided!"),
                )],
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
        self.wallpapers.clear();
        let mut distr = Vec::new();
        for directory in self.wallpaper_directories.clone() {
            let (mut picbuf, mut distbuf) = Self::parse_directory_recursive(directory)?;
            self.wallpapers.append(&mut picbuf);
            distr.append(&mut distbuf);
        }
        self.distribution = WeightedIndex::new(distr.iter()).unwrap();
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

    pub fn next_picture(&mut self, instant: bool) -> Result<(), Error> {
        if !self
            .last_change
            .elapsed()
            .saturating_sub(self.sleep)
            .is_zero()
            || instant
        {
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
            self.last_change = Instant::now();
        }
        Ok(())
    }

    fn random_picture(&mut self) -> Result<PathBuf, Error> {
        let mut new_wallpaper =
            self.wallpapers[self.distribution.sample(&mut self.generator)].to_owned();
        if !new_wallpaper.is_file() {
            eprintln!(
                "Some files were deleted or moved. Not found: {}",
                new_wallpaper.to_str().unwrap()
            );
            self.init_pictures()?;
            new_wallpaper =
                self.wallpapers[self.distribution.sample(&mut self.generator)].to_owned();
        }
        Ok(new_wallpaper)
    }

    pub fn get_info(&self) -> Info {
        Info {
            current_wallpaper: self.current_wallpaper.clone(),
            duration: self.sleep,
            time_left: self.sleep.saturating_sub(self.last_change.elapsed()),
        }
    }
}

impl Default for Manager {
    fn default() -> Self {
        Self {
            generator: rand::make_rng(),
            last_change: Instant::now(),
            current_wallpaper: None,
            wallpaper_directories: Vec::new(),
            wallpapers: Vec::new(),
            distribution: WeightedIndex::new([1].iter()).unwrap(),
            sleep: Duration::from_secs(300),
            renderer: Renderer::Auto,
            process_list: System::new_with_specifics(
                RefreshKind::nothing().with_processes(ProcessRefreshKind::nothing()),
            ),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Info {
    current_wallpaper: Option<PathBuf>,
    duration: Duration,
    time_left: Duration,
}
