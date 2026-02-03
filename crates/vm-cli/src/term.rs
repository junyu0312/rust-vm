use std::io;
use std::os::fd::AsRawFd;

use termios::*;

pub struct TermBackup {
    old_termios: Termios,
}

impl Drop for TermBackup {
    fn drop(&mut self) {
        let stdin_fd = io::stdin().as_raw_fd();

        if let Err(err) = tcsetattr(stdin_fd, TCSANOW, &self.old_termios) {
            eprintln!("Failed to restore terminal: {err}");
        }
    }
}

pub fn term_init() -> anyhow::Result<TermBackup> {
    let stdin_fd = io::stdin().as_raw_fd();

    let old_termios = Termios::from_fd(stdin_fd)?;

    let mut termios = old_termios;

    termios.c_lflag &= !(ICANON | ECHO | ISIG);
    termios.c_iflag &= !(ICRNL);

    tcsetattr(stdin_fd, TCSANOW, &termios)?;

    Ok(TermBackup { old_termios })
}
