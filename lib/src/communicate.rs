use std::{
    io::{BufReader, prelude::*},
    path::PathBuf,
    time::Duration,
};

use interprocess::local_socket::{GenericFilePath, GenericNamespaced, Stream, prelude::*};
use serde::{Deserialize, Serialize};

use crate::Error;

///Communication protocol for all the different processes.
#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Kill,
    Next,
    Reload,
    InfoRequest,
    InfoAnswer(Info),
    SetControlFlow(bool),
    ToggleControlFlow,
    Confirmation(Box<Message>),
}

///Information for the curent state of wallie-daemon
#[derive(Debug, Serialize, Deserialize)] //TODO: impl Display
pub struct Info {
    pub current_wallpaper: Option<PathBuf>,
    pub duration: Duration,
    pub time_left: Duration,
    pub playing: bool,
}

#[derive(Serialize, Deserialize)]
struct MessageWrapper {
    msg: Message,
    awaits_confirmation: bool,
}

#[allow(unused)]
fn await_message() -> Result<(Message, bool), Error> {
    Err(Error::NotImplemented)
}

/**Send a message to the server.

As this does not await a response and returns immedeately upon sending the message, you can not
use `Message::InfoRequest` here. If you do, the function will throw an `Error::NotAllowed` error.
*/
pub fn send_message(msg: Message) -> Result<(), Error> {
    if let Message::InfoRequest = msg {
        Err(Error::NotAllowed(
            "Unproper use of send_message(). InfoRequest expects a response, but this function does not await a responce. Use send_message_await() instead.".to_string(),
        ))
    } else {
        call_server(
            ron::to_string(&MessageWrapper {
                msg,
                awaits_confirmation: false,
            })?
            .as_str(),
        )
    }
}

/**Send message to the server, expecting a response.

For most messages it should return `Message::Confirmation`. `Message::InfoRequest` should return `Message::InfoAnswer`.
*/
pub fn send_message_await(msg: Message) -> Result<Message, Error> {
    call_server(
        ron::to_string(&MessageWrapper {
            msg,
            awaits_confirmation: true,
        })?
        .as_str(),
    )?;
    Ok(ron::from_str::<MessageWrapper>(listen_server()?.as_str())?.msg)
}

/**A function for the server to read the socket directly and deserealize without depending on serde.

Use `send_message_await()` with a request instead of reading the socket directly, as this may intercept the server-client communication from other clients.
If there will be a need, in the future there will be created a special socket or file to advertise the state of wallie-daemon.
*/
pub fn deserealise(request: &str) -> Result<(Message, bool), Error> {
    let data: MessageWrapper = ron::from_str(request)?;
    Ok((data.msg, data.awaits_confirmation))
}

fn call_server(request: &str) -> Result<(), Error> {
    let name = if GenericNamespaced::is_supported() {
        "wallie.sock".to_ns_name::<GenericNamespaced>()?
    } else {
        "/tmp/wallie.sock".to_fs_name::<GenericFilePath>()?
    };

    // Will fail immediately if the server hasn't started yet.
    let mut conn = BufReader::new(Stream::connect(name)?);

    // BufReader doesn't pass Write through, so we use get_mut.
    conn.get_mut().write_all(request.as_bytes())?;

    // Avoid holding up resources.
    drop(conn);

    Ok(())
}

fn listen_server() -> Result<String, Error> {
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
