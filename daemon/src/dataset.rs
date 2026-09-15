use std::{io::Error, path::PathBuf};

use rand::{
    distr::{Distribution, weighted::WeightedIndex},
    rngs::SmallRng,
};
use wallie_lib::{
    dataset::{Database, Wallpaper},
    dprint,
    rules::Rule,
};

#[derive(Clone)]
pub struct Heap {
    wallpapers: Vec<Wallpaper>,
    distribution: Option<WeightedIndex<u32>>,
    database: Database,
    current_rules: Vec<Rule>,
}

impl WallLoader for Heap {
    fn next(&mut self, generator: &mut SmallRng) -> Result<PathBuf, Error> {
        eprintln!("Fetching a new wallpaper");
        if let Some(distribution) = self.distribution.to_owned() {
            let mut new_wallpaper = self.wallpapers[distribution.sample(generator)].to_owned();
            for _ in 0..24 {
                if new_wallpaper.path.is_file() {
                    break;
                } else {
                    eprintln!(
                        "Some files were deleted or moved. Not found: {}. Please regenerate the dabase for this warning to go away.",
                        new_wallpaper.path.to_str().unwrap()
                    );
                    new_wallpaper = self.wallpapers[distribution.sample(generator)].to_owned();
                }
            }
            Ok(new_wallpaper.path)
        } else {
            self.reload()?;
            Ok(self.wallpapers.first().unwrap().path.to_path_buf())
        }
    }

    fn reload(&mut self) -> Result<(), Error> {
        if let Some((wallpapers, distr, current_rules)) =
            self.database.get_walls(&self.current_rules)
        {
            eprintln!("reloaded!");
            self.wallpapers = wallpapers;
            self.current_rules = current_rules;
            self.distribution = WeightedIndex::new(distr).ok();
        } else {
            dprint(format!(
                "No reload happened! Current state:\n{:#?}\n{:#?}",
                self.wallpapers, self.current_rules
            ));
        }
        Ok(())
    }

    fn reload_static(&mut self) -> Result<(), Error> {
        self.reload()
    }
}

impl Heap {
    pub fn generate(database: Database) -> Self {
        let mut out = Self {
            database,
            ..Default::default()
        };
        let _ = out.reload();
        out
    }
}

#[allow(clippy::derivable_impls)]
impl Default for Heap {
    fn default() -> Self {
        Self {
            wallpapers: Vec::new(),
            distribution: None,
            database: Database::default(),
            current_rules: Vec::new(),
        }
    }
}

pub trait WallLoader {
    fn next(&mut self, generator: &mut SmallRng) -> Result<PathBuf, Error>;
    fn reload(&mut self) -> Result<(), Error>;
    fn reload_static(&mut self) -> Result<(), Error>;
}

pub struct Simple {
    wallpaper_directories: Vec<PathBuf>,
    wallpapers: Vec<PathBuf>,
    distribution: WeightedIndex<u32>,
}

impl WallLoader for Simple {
    fn next(&mut self, generator: &mut SmallRng) -> Result<PathBuf, Error> {
        let new_wallpaper = self.wallpapers[self.distribution.sample(generator)].to_owned();
        if !new_wallpaper.is_file() {
            eprintln!(
                "Some files were deleted or moved. Not found: {}",
                new_wallpaper.to_str().unwrap()
            );
            self.reload()?;
            self.next(generator)
        } else {
            Ok(new_wallpaper)
        }
    }

    fn reload(&mut self) -> Result<(), Error> {
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

    fn reload_static(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

impl Simple {
    pub fn from_directory(path: PathBuf) -> Result<Self, Error> {
        let mut output = Self {
            wallpaper_directories: vec![path],
            wallpapers: Vec::new(),
            distribution: WeightedIndex::new([1_u32].iter()).unwrap(),
        };
        output.reload()?;
        Ok(output)
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
}
