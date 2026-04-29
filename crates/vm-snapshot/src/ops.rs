use std::io::Read;
use std::io::Write;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub trait Pausable {
    fn pause(&mut self) -> Result<(), Error>;

    fn resume(&mut self) -> Result<(), Error>;
}

pub trait Snapshotable {
    fn save(&self, writer: &mut dyn Write) -> Result<Vec<u8>, Error>;

    fn restore(&mut self, reader: &mut dyn Read) -> Result<(), Error>;
}
