use std::io::{Read, Write};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub trait Pausable {
    fn is_running(&self) -> bool;

    fn pause(&mut self) -> Result<(), Error>;

    fn resume(&mut self) -> Result<(), Error>;
}

pub trait Snapshotable {
    fn save<W>(&self, writer: &mut W) -> Result<Vec<u8>, Error>
    where
        W: Write;

    fn restore<R>(&mut self, reader: &mut R) -> Result<(), Error>
    where
        R: Read;
}
