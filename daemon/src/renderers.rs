use std::{
    env,
    io::{self, ErrorKind},
    path::{Path, PathBuf},
    process::{self, Command},
    str::FromStr,
    thread::sleep,
    time::Duration,
};

use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};

#[derive(Clone)]
pub enum Renderer {
    ///Infers the type of renderer at launch by either finding one, that already runs, or defaulting to `Renderer::Awww`.
    Auto, //TODO: Add a configuration option
    Awww,
    Swaybg,
    ///A not yet implemented interface to cosmic-bg
    #[allow(unused)]
    Cosmic, //TODO: something like `sed -i "s|source: Path(\"[^\"]*\"|source: Path(\"/home/argentum/Pictures/Wallpapers/wallpaper.png\"|" ~/.config/cosmic/com.system76.CosmicBackground/v1/all` but in Rust. Maybe use serde ron? Once the support for multiple outputs is added to wallie, use `output.[$name]` instead of `all` and requiering same-on-all set to true.
    ///Interface to KDE Plasma's background manager
    Plasma,
    #[allow(unused)]
    Other(PathBuf, Vec<String>),
}

impl Renderer {
    pub fn name(&self) -> String {
        match self {
            Renderer::Awww => String::from_str("awww").unwrap(),
            Renderer::Swaybg => String::from_str("swaybg").unwrap(),
            Renderer::Cosmic => String::from_str("COSMIC").unwrap(),
            Renderer::Plasma => String::from_str("KDE Plasma").unwrap(),
            _ => String::from_str("something is not right").unwrap(),
        }
    }

    pub fn auto(self, process_list: &mut System) -> Self {
        if let Renderer::Auto = self {
            if process_list
                .processes_by_exact_name("awww-daemon".as_ref())
                .next()
                .is_some()
            {
                Renderer::Awww
            } else if env::var("XDG_SESSION_DESKTOP").is_ok_and(|var| var == "KDE") {
                Renderer::Plasma
            } else {
                Renderer::Swaybg
            }
        } else {
            self
        }
    }

    pub fn change(
        &self,
        picture: &Path,
        process_list: &mut System,
    ) -> Result<process::ExitStatus, io::Error> {
        let _ = self.spawn(picture, process_list)?;
        match self {
            Renderer::Awww => spawn("awww", vec!["img", picture.to_str().unwrap()]),
            Renderer::Swaybg => {
                spawn_detached("swaybg", vec!["-i", picture.to_str().unwrap()])?;
                sleep(Duration::from_millis(100));
                if let Some(process) = process_list
                    .processes_by_exact_name("swaybg".as_ref())
                    .next()
                {
                    process.kill();
                }
                Ok(process::ExitStatus::default())
            }
            Renderer::Plasma => spawn(
                "plasma-apply-wallpaperimage",
                vec![picture.to_str().unwrap()],
            ),
            Renderer::Cosmic => Err(io::Error::new(
                ErrorKind::Unsupported,
                "COSMIC integration is not yet implemented!",
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
            Renderer::Auto => Err(io::Error::new(
                ErrorKind::Unsupported,
                "Renderer not initialized!",
            )),
        }
    }

    pub fn spawn(
        &self,
        picture: &Path,
        process_list: &mut System,
    ) -> Result<process::ExitStatus, io::Error> {
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
                    spawn_detached("awww-daemon", vec![])?;
                }
                Ok(process::ExitStatus::default())
            }
            Renderer::Swaybg => {
                if process_list
                    .processes_by_exact_name("swaybg".as_ref())
                    .next()
                    .is_none()
                {
                    spawn_detached("swaybg", vec!["-i", picture.to_str().unwrap()])?;
                }
                Ok(process::ExitStatus::default())
            }
            Renderer::Cosmic => Err(io::Error::new(
                ErrorKind::Unsupported,
                "Cosmic integration is not yet implemented!",
            )),
            Renderer::Plasma => {
                eprintln!(
                    "No need to initialize KDE Plasma. If it isn't started at this point, then I don't know."
                );
                Ok(process::ExitStatus::default())
            }
            Renderer::Other(_, _) => {
                Ok(process::ExitStatus::default())
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
            Renderer::Auto => Err(io::Error::new(
                ErrorKind::Unsupported,
                "Renderer not initialized!",
            )),
        }
    }

    #[allow(unused)]
    pub fn kill(&self) -> Result<process::ExitStatus, io::Error> {
        match self {
            Renderer::Awww => Ok(process::ExitStatus::default()), //Command::new("/bin/awww").arg("kill").status(),
            Renderer::Swaybg => Ok(process::ExitStatus::default()), //Command::new("/bin/pkill").arg("swaybg").status(),
            Renderer::Cosmic => {
                eprintln!(
                    "As cosmic-bg is a part of the COSMIC Desktop Enviernment, killing it is pointless. If you do actualy need to kill it, please do that manualy with pkill cosmic-bg."
                );
                Ok(process::ExitStatus::default())
            }
            Renderer::Plasma => {
                eprintln!(
                    "As wallpapers on KDE Plasma are a part of KDE Plasma, we have no control over their runtime."
                );
                Ok(process::ExitStatus::default())
            }
            Renderer::Other(path, _) => Command::new("/bin/pkill")
                .arg(path.to_str().unwrap().to_string().split_off(5))
                .status(),
            Renderer::Auto => Err(io::Error::new(
                ErrorKind::Unsupported,
                "Renderer not initialized!",
            )),
        }
    }
}

fn spawn(program: &str, args: Vec<&str>) -> Result<process::ExitStatus, io::Error> {
    match Command::new(format!("/bin/{}", program))
        .args(args.clone())
        .status()
    {
        Err(error) if error.kind() == ErrorKind::NotFound => {
            Command::new(format!("/usr/local/bin/{}", program))
                .args(args)
                .status()
        }
        smth => smth,
    }
}

fn spawn_detached(program: &str, args: Vec<&str>) -> Result<(), io::Error> {
    if let Err(error) = Command::new(format!("/bin/{}", program))
        .args(args.clone())
        .spawn()
    {
        if error.kind() == ErrorKind::NotFound {
            Command::new(format!("/usr/local/bin/{}", program))
                .args(args)
                .spawn()?;
        } else {
            Err(error)?;
        }
    }
    Ok(())
}
