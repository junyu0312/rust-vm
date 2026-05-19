use std::io::Read;
use std::io::Write;

pub trait Pausable {
    type Error;

    fn pause(&mut self) -> Result<(), Self::Error>;

    fn resume(&mut self) -> Result<(), Self::Error>;
}

pub trait SaveSnapshot {
    type Error;

    fn save(&self, writer: &mut dyn Write) -> Result<(), Self::Error>;
}

pub trait LoadSnapshot {
    type Error;

    fn load(&mut self, reader: &mut dyn Read) -> Result<(), Self::Error>;
}
