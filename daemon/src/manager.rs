use rand::{
    distr::{Distribution, weighted::WeightedIndex},
    rngs::SmallRng,
};

use std::{
    env::args,
    io::Error,
    path::PathBuf,
    process::Command,
    str::FromStr,
    time::{Duration, Instant},
};

pub struct Manager {
    generator: SmallRng,
    last_change: Instant,
    wallpaper_directories: Vec<PathBuf>,
    wallpapers: Vec<PathBuf>,
    distribution: WeightedIndex<u8>,
    sleep: Duration,
    wallpaper_renderer: PathBuf,
    wallpaper_renderer_arguments: Vec<String>,
}

impl Manager {
    pub fn new_from_args() -> Self {
        let (wallpaper_directories, sleep) = Self::parse_arguments();
        Self {
            wallpaper_directories,
            sleep,
            ..Default::default()
        }
    }

    fn parse_arguments() -> (Vec<PathBuf>, Duration) {
        let arguments = args().collect::<Vec<String>>();
        let mut timing = Duration::from_secs(300);
        if arguments.contains(&"-d".to_string()) {
            timing = Duration::from_secs(
                u64::from_str(
                    arguments[arguments
                        .iter()
                        .position(|entry| entry == &"-d".to_string())
                        .unwrap()
                        + 1]
                    .as_str(),
                )
                .expect("Invalid sleep duration!"),
            );
        }
        (
            vec![PathBuf::from(
                arguments.last().expect("No directory provided!"),
            )],
            timing,
        )
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

    fn parse_directory_recursive(directory: PathBuf) -> Result<(Vec<PathBuf>, Vec<u8>), Error> {
        let entries = directory.read_dir()?;
        let mut pictures = Vec::<PathBuf>::new();
        let mut distribution = Vec::<u8>::new();
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
            let mut new_wallpaper = &self.wallpapers[self.distribution.sample(&mut self.generator)];
            if !new_wallpaper.is_file() {
                eprintln!("Some files were deleted or moved. Not found: {}", new_wallpaper.to_str().unwrap());
                self.init_pictures()?;
                new_wallpaper = &self.wallpapers[self.distribution.sample(&mut self.generator)];
            }
            Command::new(self.wallpaper_renderer.to_str().unwrap())
                .args(
                    [
                        self.wallpaper_renderer_arguments.as_slice(),
                        vec![
                            new_wallpaper
                                .to_str()
                                .unwrap()
                                .to_string(),
                        ]
                        .as_slice(),
                    ]
                    .concat(),
                )
                .output()?;
            self.last_change = Instant::now();
        }
        Ok(())
    }
}

impl Default for Manager {
    fn default() -> Self {
        Self {
            generator: rand::make_rng(),
            last_change: Instant::now(),
            wallpaper_directories: Vec::new(),
            wallpapers: Vec::new(),
            distribution: WeightedIndex::new([1].iter()).unwrap(),
            sleep: Duration::from_secs(300),
            wallpaper_renderer: PathBuf::from_str("/bin/awww").unwrap(),
            wallpaper_renderer_arguments: vec!["img".to_string()],
        }
    }
}
