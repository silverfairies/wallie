use std::{
    io::{self, BufRead, BufReader, Error},
    thread::sleep,
    time::Duration,
};

use interprocess::local_socket::{
    GenericNamespaced, ListenerNonblockingMode, ListenerOptions, ToNsName, traits::Listener,
};

mod manager;
use manager::Manager;

fn main() -> Result<(), Error> {
    println!("Hello, world!");

    let mut manager = Manager::new_from_args();
    manager.init_pictures()?;

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

    loop {
        let socket = listener.accept();

        if let Ok(request) = socket {
            let mut request_text = String::new();
            let mut read_request = BufReader::new(request);
            read_request.read_line(&mut request_text)?;
            match request_text.trim() {
                "kill" => break,
                "next" => manager.next_picture(true)?,
                "reload" => manager.init_pictures()?,
                _ => (),
            }
        } else {
            manager.next_picture(false)?;
        }
        sleep(Duration::from_millis(10));
    }
    Ok(())
}
