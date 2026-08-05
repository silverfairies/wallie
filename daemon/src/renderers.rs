use std::{
    io::{Error, ErrorKind},
    path::PathBuf,
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
        picture: &PathBuf,
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
        picture: &PathBuf,
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
                        .arg(picture.to_str().unwrap().to_string())
                        .spawn()?;
                }
                Ok(ExitStatus::default())
            }
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
