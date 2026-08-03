use std::{env::args, io::Error, path::PathBuf, time::Duration};
use serde::{Deserialize, Serialize};

use {
    interprocess::local_socket::{GenericFilePath, GenericNamespaced, Stream, prelude::*},
    std::io::{BufReader, prelude::*},
};

const COMMANDS: [&str; 11] = ["next", "kill", "reload", "info", "img", "help", "pause", "unpause", "help", "-h", "--help"];
const HELP: &str = "Wallie 0.1.0-alpha

wallie command [options]

Commands:
    help, -h, --help
        Show this help message

    next
        Skip the timer and show new wallpaper

    kill
        Kill the daemon

    reload
        Reparses the wallpaper directory

    info
        Get curent wallpaper information
        Takes in arguments:
            -p, --pretty
                Prettyprint
            -j, --json
                Formated JSON, ignores all  other arguments besides -p
            -w, --wallpaper
                Path to curent wallpaper or none
            -d, --duration
                Duration between wallpapers
            -t, --time-left
                Time left till next wallpaper
            -e, --exact
                Specify timings in nanoseconds instead of whole seconds";

fn main() {
    let mut unproper = true;
    let mut arg = args().skip_while(|arg| !COMMANDS.contains(&arg.as_str()));

    if let Some(entry) = arg.next() {
        match entry.trim() {
            "info" => info(arg).unwrap(),
            "help" | "-h" | "--help" => println!("{}", HELP),
            _ => call_server(entry.trim()).unwrap(),
        }
        unproper = false;
    }
    if unproper {
        eprintln!("Unproper arguments!");
    }
}

fn info <I: Iterator<Item = String>> (arguments: I) -> Result<(), Error> {
    let info_args = [
        ("wallpaper", 'w'),
        ("duration", 'd'),
        ("time-left",'t'),
        ("exact", 'e'),
        ("pretty", 'p'),
        ("json", 'j'),
    ];
    let mut request = Vec::new();
    for arg in arguments {
        request.append(&mut parser(arg.strip_prefix('-').ok_or(Error::new(std::io::ErrorKind::InvalidInput, format!("Invalid argument: {}", arg)))?.to_string(), &info_args)?);
    }
    call_server("info")?;

    let response_deser: Info = ron::from_str(&read_server()?).unwrap();

    if request.contains(&"json".to_string()) {
        if request.contains(&"pretty".to_string()) {
            println!("{}", serde_json::to_string_pretty(&response_deser)?);
        } else {
            println!("{}", serde_json::to_string(&response_deser)?);
        }
    } else {
        if request.contains(&"pretty".to_string()) {
            if request.contains(&"wallpaper".to_string()) || request.contains(&"duration".to_string()) || request.contains(&"time-left".to_string()) {
                if request.contains(&"wallpaper".to_string()) {
                    if let Some(wallpaper) = response_deser.current_wallpaper {
                        println!("Curent wallpaper path: {}", wallpaper.to_str().unwrap());
                    } else {
                        println!("Wallie does not track current wallpaper!");
                    }
                }
                if request.contains(&"duration".to_string()) {
                    if request.contains(&"exact".to_string()) {
                        println!("Time between wallpapers in nanoseconds: {}ns", response_deser.duration.as_nanos());
                    } else {
                        println!("Time between wallpapers: {}s", response_deser.duration.as_secs());
                    }
                }
                if request.contains(&"time-left".to_string()) {
                    if request.contains(&"exact".to_string()) {
                        println!("Time till next wallaper in nanoseconds: {}ns", response_deser.time_left.as_nanos());
                    } else {
                        println!("Time till next wallpaper: {}s", response_deser.time_left.as_secs());
                    }
                }
            } else {
                if let Some(wallpaper) = response_deser.current_wallpaper {
                    println!("Curent wallpaper path: {}", wallpaper.to_str().unwrap());
                } else {
                    println!("Wallie does not track current wallpaper!");
                }
                if request.contains(&"exact".to_string()) {
                    println!("Time between wallpapers in nanoseconds: {}ns", response_deser.duration.as_nanos());
                    println!("Time till next wallaper in nanoseconds: {}ns", response_deser.time_left.as_nanos());
                } else {
                    println!("Time between wallpapers: {}s", response_deser.duration.as_secs());
                    println!("Time till next wallpaper: {}s", response_deser.time_left.as_secs());
                }
            }
        } else {
            if request.contains(&"wallpaper".to_string()) || request.contains(&"duration".to_string()) || request.contains(&"time-left".to_string()) {
                if request.contains(&"wallpaper".to_string()) {
                    if let Some(wallpaper) = response_deser.current_wallpaper {
                        println!("{}", wallpaper.to_str().unwrap());
                    } else {
                        println!("none");
                    }
                }
                if request.contains(&"duration".to_string()) {
                    if request.contains(&"exact".to_string()) {
                        println!("{}", response_deser.duration.as_nanos());
                    } else {
                        println!("{}", response_deser.duration.as_secs());
                    }
                }
                if request.contains(&"time-left".to_string()) {
                    if request.contains(&"exact".to_string()) {
                        println!("{}", response_deser.time_left.as_nanos());
                    } else {
                        println!("{}", response_deser.time_left.as_secs());
                    }
                }
            } else {
                if let Some(wallpaper) = response_deser.current_wallpaper {
                    println!("{}", wallpaper.to_str().unwrap());
                } else {
                    println!("none");
                }
                if request.contains(&"exact".to_string()) {
                    println!("{}", response_deser.duration.as_nanos());
                    println!("{}", response_deser.time_left.as_nanos());
                } else {
                    println!("{}", response_deser.duration.as_secs());
                    println!("{}", response_deser.time_left.as_secs());
                }
            }
        }
    }
    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Info {
    current_wallpaper: Option<PathBuf>,
    duration: Duration,
    time_left: Duration,
}

fn parser(arg: String, possible_arguments: &[(&str, char)]) -> Result<Vec<String>, Error> {
    let mut parsed = Vec::new();
    if arg.starts_with('-') {
        let argument = arg.strip_prefix('-').unwrap();
        Ok(vec![possible_arguments.iter().find(|possible| possible.0 == argument).ok_or(Error::new(std::io::ErrorKind::InvalidInput, format!("Invalid argument: {}", argument)))?.0.to_owned().to_string()])
    } else {
        for each in arg.chars() {
            parsed.push(possible_arguments.iter().find(|possible| possible.1 == each).ok_or(Error::new(std::io::ErrorKind::InvalidInput, format!("Invalid argument: {}", each)))?.0.to_owned().to_string());
        }
        Ok(parsed)
    }
}

fn call_server(request: &str) -> Result<(), Error> {
    let name = if GenericNamespaced::is_supported() {
        "wallie.sock".to_ns_name::<GenericNamespaced>()?
    } else {
        "/tmp/wallie.sock".to_fs_name::<GenericFilePath>()?
    };

    //let mut buffer = String::with_capacity(128);

    // Will fail immediately if the server hasn't started yet.
    let mut conn = BufReader::new(Stream::connect(name)?);

    // BufReader doesn't pass Write through, so we use get_mut.
    conn.get_mut()
        .write_all(request.as_bytes())?;

    // We now employ the buffer we allocated prior and receive a single line,
    // interpreting a newline character as an end-of-file (because local
    // sockets cannot be portably shut down), verifying validity of UTF-8 on
    // the fly.
    //conn.read_line(&mut buffer)?;

    // Avoid holding up resources.
    drop(conn);

    // read_line keeps the line feed at the end.
    //print!("Server answered: {buffer}");
    Ok(())
}

fn read_server() -> Result<String, Error> {
    let name = if GenericNamespaced::is_supported() {
        "wallie.sock".to_ns_name::<GenericNamespaced>()?
    } else {
        "/tmp/wallie.sock".to_fs_name::<GenericFilePath>()?
    };

    let mut buffer = String::new();

    // Will fail immediately if the server hasn't started yet.
    let mut conn = BufReader::new(Stream::connect(name)?);

    conn.read_to_string(&mut buffer)?;

    // Avoid holding up resources.
    drop(conn);

    Ok(buffer)
}
