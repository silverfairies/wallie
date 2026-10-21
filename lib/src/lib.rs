/*!
A library to build gui or other interfaces to wallie-daemon.
*/
#![warn(missing_docs)]
use std::io;

use thiserror::Error;

pub mod communicate;
pub mod dataset;
pub mod rules;

#[doc(hidden)]
#[cfg(debug_assertions)]
pub fn dprint(print: String) {
    eprintln!("{}", print);
}

#[doc(hidden)]
#[cfg(not(debug_assertions))]
pub fn dprint(_print: String) {
    ()
}

/**Error enum for all posible errors from the library*/
#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Ron(#[from] ron::Error),
    #[error(transparent)]
    RonSpanned(#[from] ron::error::SpannedError),
    #[error("Action not allowed. {0}")]
    NotAllowed(String),
    #[error("Not yet implemented!")]
    NotImplemented,
    #[error("Unknown error encountered!")]
    Unknown,
    #[error("{0}")]
    Other(String),
}
