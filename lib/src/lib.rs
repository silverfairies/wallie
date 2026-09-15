use std::{path::PathBuf, time::Duration};

use serde::{Deserialize, Serialize};

pub mod dataset;
pub mod rules;

#[derive(Debug, Serialize, Deserialize)]
pub struct Info {
    pub current_wallpaper: Option<PathBuf>,
    pub duration: Duration,
    pub time_left: Duration,
    pub playing: bool,
}

#[cfg(debug_assertions)]
pub fn dprint(print: String) {
    eprintln!("{}", print);
}
