use std::io::Error;

use rand::{
    distr::{Distribution, weighted::WeightedIndex},
    rngs::SmallRng,
};
use wallie_lib::dataset::Wallpaper;

#[derive(Clone)]
pub struct Heap {
    wallpapers: Vec<Wallpaper>,
    distribution: Option<WeightedIndex<u32>>,
}

impl Heap {
    pub fn next(&self, generator: &mut SmallRng) -> Result<Wallpaper, Error> {
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
            Ok(new_wallpaper)
        } else {
            Err(Error::other(
                "No valid database provided or no pictures following the curent rules found!",
            ))
        }
    }
}

impl Default for Heap {
    fn default() -> Self {
        Self {
            wallpapers: Vec::new(),
            distribution: None,
        }
    }
}
