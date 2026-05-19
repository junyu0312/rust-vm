use std::io::Read;
use std::io::Write;

pub trait Pausable {
    type Error;

    fn pause(&mut self) -> Result<(), Self::Error>;

    fn resume(&mut self) -> Result<(), Self::Error>;
}

pub trait Snapshotable {
    type Error;

    fn save(&self, writer: &mut dyn Write) -> Result<(), Self::Error>;

    fn restore(&self, reader: &mut dyn Read) -> Result<(), Self::Error>;
}
