use std::{
    io::{self, BufReader, Error, Read, Write},
    thread::sleep,
    time::Duration,
};

use constcat::concat;
use interprocess::local_socket::{
    GenericNamespaced, ListenerNonblockingMode, ListenerOptions, ToNsName, traits::Listener,
};

mod manager;
use manager::Manager;

mod dataset;

mod rules;

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
                Backend for displaying wallpapers. One of awww, swaybg, auto (default: auto, if can't find anything, then default awww)

");

fn main() -> Result<(), Error> {
    if let Ok(mut manager) = Manager::new_from_args() {
        manager.init_pictures()?;
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
            let socket = listener.accept();

            if let Ok(request) = socket {
                let mut request_text = String::new();
                let mut read_request = BufReader::new(request);
                read_request.read_to_string(&mut request_text)?;
                match request_text.as_str() {
                    "kill" => living = false,
                    "next" => manager.next_picture(true)?,
                    "reload" => manager.init_pictures()?,
                    "info" => listener
                        .accept()?
                        .write_all(ron::to_string(&manager.get_info()).unwrap().as_bytes())?,
                    _ => manager.next_picture(false)?,
                }
            } else {
                manager.next_picture(false)?;
            }
            sleep(Duration::from_millis(100));
        }
    } else {
        println!("{}", HELP);
    }
    Ok(())
}
