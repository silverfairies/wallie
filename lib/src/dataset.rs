use std::{fs::read_to_string, io::Error, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::rules::Rule;

#[derive(Serialize, Deserialize, Default)]
pub struct Database {
    tags: Vec<Tag>,
    wallpapers: Vec<Wallpaper>,
    rules: Vec<Rule>,
}

impl Database {
    pub fn from_file(path: PathBuf) -> Result<Database, Error> {
        ron::from_str(read_to_string(path)?.as_str()).map_err(Error::other)
    }
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct Tag {
    name: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Wallpaper {
    name: Option<String>,
    pub path: PathBuf,
    tags: Vec<Tag>,
    r#type: WallpaperType,
}

#[derive(Serialize, Deserialize, Clone)]
enum WallpaperType {
    Picture,
    Animation,
    Video,
}
