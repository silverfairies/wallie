use constcat::concat;
use ron::ser::PrettyConfig;
use std::{
    env::args,
    io::{self, ErrorKind},
    path::PathBuf,
    str::FromStr,
};
use wallie_lib::{
    Error,
    communicate::{Message, send_message, send_message_await},
    dataset::{Database, Tag, Wallpaper},
    rules::{Condition, Rule, Time, TimeRange},
};

const COMMANDS: [&str; 13] = [
    "next",
    "kill",
    "reload",
    "info",
    "img",
    "help",
    "pause",
    "resume",
    "toggle",
    "help",
    "-h",
    "--help",
    "generate-config",
];
const HELP: &str = concat!(env!("CARGO_PKG_NAME"), " v", env!("CARGO_PKG_VERSION"), "

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

    info [options]
        Get curent wallpaper information
        Takes in options:
            -p, --pretty
                Prettyprint
            -j, --json
                Formated as JSON, ignores all other arguments besides -p. Takes precedence over --ron
            -r, --ron
                Formated as RON, ignores all other arguments besides -p
            -w, --wallpaper
                Path to curent wallpaper or none
            -d, --duration
                Duration between wallpapers
            -t, --time-left
                Time left till next wallpaper
            -e, --exact
                Specify timings in nanoseconds instead of whole seconds
");

fn main() {
    let mut unproper = true;
    let mut arg = args().skip_while(|arg| !COMMANDS.contains(&arg.as_str()));

    if let Some(entry) = arg.next() {
        match entry.trim() {
            "info" => info(arg).unwrap(),
            "help" | "-h" | "--help" => println!("{}", HELP),
            "generate-config" => generate_config(),
            _ => send_message(convert_str_msg(entry.trim()).unwrap()).unwrap(),
        }
        unproper = false;
    }
    if unproper {
        eprintln!("Unproper arguments!");
    }
}

fn info<I: Iterator<Item = String>>(arguments: I) -> Result<(), Error> {
    let info_args = [
        ("wallpaper", 'w'),
        ("duration", 'd'),
        ("time-left", 't'),
        ("state", 's'),
        ("exact", 'e'),
        ("pretty", 'p'),
        ("json", 'j'),
        ("ron", 'r'),
    ];
    let mut request = Vec::new();
    for arg in arguments {
        request.append(&mut parser(
            arg.strip_prefix('-')
                .ok_or(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Invalid argument: {}", arg),
                ))?
                .to_string(),
            &info_args,
        )?);
    }
    let response = send_message_await(Message::InfoRequest)?;

    if let Message::InfoAnswer(response) = response {
        if request.contains(&"json".to_string()) {
            if request.contains(&"pretty".to_string()) {
                println!("{}", serde_json::to_string_pretty(&response).unwrap());
            } else {
                println!("{}", serde_json::to_string(&response).unwrap());
            }
        } else if request.contains(&"ron".to_string()) {
            if request.contains(&"pretty".to_string()) {
                println!(
                    "{}",
                    ron::ser::to_string_pretty(&response, PrettyConfig::default()).unwrap()
                );
            } else {
                println!("{}", ron::to_string(&response).unwrap());
            }
        } else {
            if request.contains(&"pretty".to_string()) {
                if request.contains(&"wallpaper".to_string())
                    || request.contains(&"duration".to_string())
                    || request.contains(&"time-left".to_string())
                    || request.contains(&"state".to_string())
                {
                    if request.contains(&"wallpaper".to_string()) {
                        if let Some(wallpaper) = response.current_wallpaper {
                            println!("Curent wallpaper path: {}", wallpaper.to_str().unwrap());
                        } else {
                            println!("Wallie does not track current wallpaper!");
                        }
                    }
                    if request.contains(&"duration".to_string()) {
                        if request.contains(&"exact".to_string()) {
                            println!(
                                "Time between wallpapers in nanoseconds: {}ns",
                                response.duration.as_nanos()
                            );
                        } else {
                            println!("Time between wallpapers: {}s", response.duration.as_secs());
                        }
                    }
                    if request.contains(&"time-left".to_string()) {
                        if request.contains(&"exact".to_string()) {
                            println!(
                                "Time till next wallaper in nanoseconds: {}ns",
                                response.time_left.as_nanos()
                            );
                        } else {
                            println!(
                                "Time till next wallpaper: {}s",
                                response.time_left.as_secs()
                            );
                        }
                    }
                    if request.contains(&"state".to_string()) {
                        println!("Playing: {}", if response.playing { "yes" } else { "no" })
                    }
                } else {
                    if let Some(wallpaper) = response.current_wallpaper {
                        println!("Curent wallpaper path: {}", wallpaper.to_str().unwrap());
                    } else {
                        println!("Wallie does not track current wallpaper!");
                    }
                    if request.contains(&"exact".to_string()) {
                        println!(
                            "Time between wallpapers in nanoseconds: {}ns",
                            response.duration.as_nanos()
                        );
                        println!(
                            "Time till next wallaper in nanoseconds: {}ns",
                            response.time_left.as_nanos()
                        );
                    } else {
                        println!("Time between wallpapers: {}s", response.duration.as_secs());
                        println!(
                            "Time till next wallpaper: {}s",
                            response.time_left.as_secs()
                        );
                    }
                    println!("Playing: {}", if response.playing { "yes" } else { "no" })
                }
            } else {
                if request.contains(&"wallpaper".to_string())
                    || request.contains(&"duration".to_string())
                    || request.contains(&"time-left".to_string())
                {
                    if request.contains(&"wallpaper".to_string()) {
                        if let Some(wallpaper) = response.current_wallpaper {
                            println!("{}", wallpaper.to_str().unwrap());
                        } else {
                            println!("none");
                        }
                    }
                    if request.contains(&"duration".to_string()) {
                        if request.contains(&"exact".to_string()) {
                            println!("{}", response.duration.as_nanos());
                        } else {
                            println!("{}", response.duration.as_secs());
                        }
                    }
                    if request.contains(&"time-left".to_string()) {
                        if request.contains(&"exact".to_string()) {
                            println!("{}", response.time_left.as_nanos());
                        } else {
                            println!("{}", response.time_left.as_secs());
                        }
                    }
                    if request.contains(&"state".to_string()) {
                        println!("{}", response.playing)
                    }
                } else {
                    if let Some(wallpaper) = response.current_wallpaper {
                        println!("{}", wallpaper.to_str().unwrap());
                    } else {
                        println!("none");
                    }
                    if request.contains(&"exact".to_string()) {
                        println!("{}", response.duration.as_nanos());
                        println!("{}", response.time_left.as_nanos());
                    } else {
                        println!("{}", response.duration.as_secs());
                        println!("{}", response.time_left.as_secs());
                    }
                    println!("{}", response.playing)
                }
            }
        }
    } else {
        eprintln!("Wrong response body: {:?}", response);
    }
    Ok(())
}

fn parser(arg: String, possible_arguments: &[(&str, char)]) -> Result<Vec<String>, Error> {
    let mut parsed = Vec::new();
    if arg.starts_with('-') {
        let argument = arg.strip_prefix('-').unwrap();
        Ok(vec![
            possible_arguments
                .iter()
                .find(|possible| possible.0 == argument)
                .ok_or(io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Invalid argument: {}", argument),
                ))?
                .0
                .to_owned()
                .to_string(),
        ])
    } else {
        for each in arg.chars() {
            parsed.push(
                possible_arguments
                    .iter()
                    .find(|possible| possible.1 == each)
                    .ok_or(io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Invalid argument: {}", each),
                    ))?
                    .0
                    .to_owned()
                    .to_string(),
            );
        }
        Ok(parsed)
    }
}

fn convert_str_msg(request: &str) -> Result<Message, Error> {
    match request {
        "kill" => Ok(Message::Kill),
        "next" => Ok(Message::Next),
        "reload" => Ok(Message::Reload),
        "pause" => Ok(Message::SetControlFlow(false)),
        "resume" => Ok(Message::SetControlFlow(true)),
        "toggle" => Ok(Message::ToggleControlFlow),
        _ => Err(wallie_lib::Error::Io(io::Error::new(
            ErrorKind::InvalidInput,
            "Invalid command",
        ))),
    }
}

fn generate_config() {
    //if !PathBuf::from_str("~/.config/wallie/test").unwrap().exists() {
    //    if !PathBuf::from_str("~/.config/wallie").unwrap().exists() {
    let data = Database {
        tags: vec![
            Tag {
                name: "dark".to_string(),
            },
            Tag {
                name: "anime".to_string(),
            },
            Tag {
                name: "light".to_string(),
            },
        ],
        wallpapers: vec![
            Wallpaper::new(
                PathBuf::from_str("/home/argentum/Pictures/Wallpapers/112.jpg").unwrap(),
                vec![Tag {
                    name: "dark".to_string(),
                }],
            ),
            Wallpaper::new(
                PathBuf::from_str("/home/argentum/Pictures/Wallpapers/sve.png").unwrap(),
                vec![Tag {
                    name: "light".to_string(),
                }],
            ),
            Wallpaper::new(
                PathBuf::from_str("/home/argentum/Pictures/Wallpapers/100.png").unwrap(),
                vec![Tag {
                    name: "light".to_string(),
                }],
            ),
            Wallpaper::new(
                PathBuf::from_str("/home/argentum/Pictures/Wallpapers/Anime/8808094.png").unwrap(),
                vec![Tag {
                    name: "anime".to_string(),
                }],
            ),
        ],
        rules: vec![
            Rule::new(
                "day".to_string(),
                1,
                1,
                false,
                vec![Tag {
                    name: "light".to_string(),
                }],
                Condition::Time(Time::Daily {
                    timerange: TimeRange::new((20, 0, 0), (8, 0, 0)),
                }),
            ),
            Rule::new(
                "night".to_string(),
                1,
                1,
                false,
                vec![Tag {
                    name: "dark".to_string(),
                }],
                Condition::Time(Time::Daily {
                    timerange: TimeRange::new((8, 0, 0), (20, 0, 0)),
                }),
            ),
            Rule::new(
                "Anime".to_string(),
                1,
                1,
                false,
                vec![Tag {
                    name: "anime".to_string(),
                }],
                Condition::Always,
            ),
        ],
    };
    println!(
        "{}",
        ron::ser::to_string_pretty(&data, PrettyConfig::new()).unwrap()
    );
}
