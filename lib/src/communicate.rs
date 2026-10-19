use std::{path::PathBuf, time::Duration};

use serde::{Deserialize, Serialize};

//Communication protocol for all the different processes.
#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Kill,
    Next,
    Reload,
    InfoRequest,
    InfoAnswer(Info),
    SetControlFlow(bool),
    ToggleControlFlow,
}

#[derive(Debug, Serialize, Deserialize)] //TODO: impl Display
pub struct Info {
    pub current_wallpaper: Option<PathBuf>,
    pub duration: Duration,
    pub time_left: Duration,
    pub playing: bool,
}
