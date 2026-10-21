use std::{
    io::{self, BufReader, Error, Read, Write},
    thread::sleep,
    time::{Duration, Instant},
};

use constcat::concat;
use interprocess::local_socket::{
    GenericNamespaced, ListenerNonblockingMode, ListenerOptions, ToNsName, traits::Listener,
};

mod manager;
use manager::Manager;
use ron::error::SpannedResult;
use wallie_lib::{communicate::Message, dprint};

mod dataset;

mod renderers;

const HELP: &str = concat!(env!("CARGO_PKG_NAME"), " v", env!("CARGO_PKG_VERSION"), "

wallie-daemon [options]

Options:
    -h, --help
        Show this help message
    --simple [options] /path/to/wallpaper/directory
        Iterate through random images in a directory and its subdirectories without applying any rules
        Takes in options:
            -d integer
                The duration between wallpaper changes in seconds (default: 300 seconds)

            -r renderer
                Backend for displaying wallpapers. One of awww, swaybg, plasma, auto (default: auto, if can't find anything, then default swaybg)

");

fn main() -> Result<(), Error> {
    /*eprintln!(
        "{:#?}",
        ron::de::from_str::<Database>(
            read_to_string(home_dir().unwrap().join(".config/wallie/database.ron"))
                .unwrap()
                .as_str()
        )
    );*/
    let get_manager = Manager::new_from_args();
    if let Ok(mut manager) = get_manager {
        manager.run()?;

        let printname = "wallie.sock";
        let name = printname.to_ns_name::<GenericNamespaced>()?;

        let listener = match ListenerOptions::new().name(name).create_sync() {
            Err(e) if e.kind() == io::ErrorKind::AddrInUse => {
                eprintln!(
                    "Error: could not start server because the socket file is \
               occupied. Please check if {printname} is in use by another \
               process and try again."
                );
                return Err(e);
            }
            x => x?,
        };

        eprintln!("Server running at {printname}");
        listener.set_nonblocking(ListenerNonblockingMode::Accept)?;

        let mut living = true;

        while living {
            let frame_start = Instant::now();
            let socket = listener.accept();

            if let Ok(request) = socket {
                let mut buffer = String::new();
                BufReader::new(request).read_to_string(&mut buffer)?;
                let request_deser = wallie_lib::communicate::deserealise(&buffer);
                if let Ok(request_text) = request_deser {
                    match request_text.0 {
                        Message::Kill => living = false,
                        Message::Next => manager.next_picture(true)?,
                        Message::Reload => manager.init_pictures()?,
                        Message::InfoRequest => listener.accept()?.write_all(
                            ron::to_string(&Message::InfoAnswer(manager.get_info()))
                                .unwrap()
                                .as_bytes(),
                        )?,
                        Message::SetControlFlow(set) => {
                            if set {
                                manager.resume()
                            } else {
                                manager.pause()
                            }
                        }
                        Message::ToggleControlFlow => manager.toggle(),
                        _ => manager.next_picture(false)?,
                    }
                } else {
                    eprintln!("Garbled request: {:?}", request_deser);
                }
            } else {
                manager.next_picture(false)?;
            }
            /*dprint(format!(
                "Cycle length: ~{}ms",
                Instant::now()
                    .saturating_duration_since(frame_start)
                    .as_millis()
            ));*/
            sleep(
                Duration::from_millis(50)
                    .saturating_sub(Instant::now().saturating_duration_since(frame_start)),
            );
        }
    } else if let Err(err) = get_manager {
        println!("{}\n{}", err, HELP);
    }
    Ok(())
}
