use std::{env::args, io::Error};
use {
    interprocess::local_socket::{GenericFilePath, GenericNamespaced, Stream, prelude::*},
    std::io::{BufReader, prelude::*},
};

const COMMANDS: [&str; 6] = ["next", "kill", "reload", "info", "img", "help"];

fn main() {
    let mut unproper = true;
    for entry in args() {
        if COMMANDS.contains(&entry.trim()) {
            call_server(entry.trim()).unwrap();
            unproper = false;
            break;
        }
    }
    if unproper {
        eprintln!("Unproper arguments!");
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
        .write_all(format!("{request}\n").as_bytes())?;

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
