use std::{
    io::{Error, ErrorKind},
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
    thread::sleep,
    time::Duration,
};

use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};

#[derive(Clone)]
pub enum Renderer {
    Auto,
    Awww,
    Swaybg,
    #[allow(unused)]
    Cosmic, //TODO: something like `sed -i "s|source: Path(\"[^\"]*\"|source: Path(\"/home/argentum/Pictures/Wallpapers/wallpaper.png\"|" ~/.config/cosmic/com.system76.CosmicBackground/v1/all` but in Rust. Maybe use serde ron? Once the support for multiple outputs is added to wallie, use `output.[$name]` instead of `all` and requiering same-on-all set to true.
    #[allow(unused)]
    Other(PathBuf, Vec<String>),
}

impl Renderer {
    pub fn auto(self, process_list: &mut System) -> Self {
        if let Renderer::Auto = self {
            if process_list
                .processes_by_exact_name("swaybg".as_ref())
                .next()
                .is_some()
            {
                Renderer::Swaybg
            } else {
                Renderer::Awww
            }
        } else {
            self
        }
    }

    pub fn change(
        &self,
        picture: &Path,
        process_list: &mut System,
    ) -> Result<std::process::ExitStatus, std::io::Error> {
        let _ = self.spawn(picture, process_list)?;
        match self {
            Renderer::Awww => Command::new("/bin/awww")
                .args(["img", picture.to_str().unwrap()])
                .status(),
            Renderer::Swaybg => {
                let _ = Command::new("/bin/swaybg")
                    .arg("-i")
                    .arg(picture.to_str().unwrap())
                    .spawn()?;
                sleep(Duration::from_millis(10));
                if let Some(process) = process_list
                    .processes_by_exact_name("swaybg".as_ref())
                    .next()
                {
                    process.kill();
                }
                Ok(ExitStatus::default())
            }
            Renderer::Cosmic => Err(Error::new(
                ErrorKind::Unsupported,
                "Cosmic integration is not yet implemented!",
            )),
            Renderer::Other(path, args) => Command::new(path.to_str().unwrap())
                .args(
                    [
                        args.as_slice(),
                        vec![picture.to_str().unwrap().to_string()].as_slice(),
                    ]
                    .concat(),
                )
                .status(),
            Renderer::Auto => Err(Error::new(
                ErrorKind::Unsupported,
                "Renderer not initialized!",
            )),
        }
    }

    pub fn spawn(
        &self,
        picture: &Path,
        process_list: &mut System,
    ) -> Result<std::process::ExitStatus, std::io::Error> {
        process_list.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing(),
        );
        match self {
            Renderer::Awww => {
                if process_list
                    .processes_by_exact_name("awww-daemon".as_ref())
                    .next()
                    .is_none()
                {
                    let _ = Command::new("/bin/awww-daemon").spawn()?;
                }
                Ok(ExitStatus::default())
            }
            Renderer::Swaybg => {
                if process_list
                    .processes_by_exact_name("swaybg".as_ref())
                    .next()
                    .is_none()
                {
                    let _ = Command::new("/bin/swaybg")
                        .arg("-i")
                        .arg(picture.to_str().unwrap())
                        .spawn()?;
                }
                Ok(ExitStatus::default())
            }
            Renderer::Cosmic => Err(Error::new(
                ErrorKind::Unsupported,
                "Cosmic integration is not yet implemented!",
            )),
            Renderer::Other(_, _) => {
                Ok(ExitStatus::default())
                /*
                Command::new(path.to_str().unwrap())
                    .args(
                        [
                            args.as_slice(),
                            vec![picture.to_str().unwrap().to_string()].as_slice(),
                        ]
                        .concat(),
                    )
                    .status()
                */
            }
            Renderer::Auto => Err(Error::new(
                ErrorKind::Unsupported,
                "Renderer not initialized!",
            )),
        }
    }

    #[allow(unused)]
    pub fn kill(&self) -> Result<std::process::ExitStatus, std::io::Error> {
        match self {
            Renderer::Awww => Command::new("/bin/awww").arg("kill").status(),
            Renderer::Swaybg => Command::new("/bin/pkill").arg("swaybg").status(),
            Renderer::Cosmic => {
                eprintln!(
                    "As cosmic-bg is a part of the COSMIC Desktop Enviernment, killing it is pointless. If you do actualy need to kill it, please do that manualy with pkill cosmic-bg."
                );
                Ok(ExitStatus::default())
            }
            Renderer::Other(path, _) => Command::new("/bin/pkill")
                .arg(path.to_str().unwrap().to_string().split_off(5))
                .status(),
            Renderer::Auto => Err(Error::new(
                ErrorKind::Unsupported,
                "Renderer not initialized!",
            )),
        }
    }
}
