use std::io;
use std::os::fd::AsRawFd;

use termios::*;

pub fn term_init() -> anyhow::Result<()> {
    let stdin_fd = io::stdin().as_raw_fd();

    let orig_termios = Termios::from_fd(stdin_fd)?;

    let mut termios = orig_termios;

    termios.c_lflag &= !(ICANON | ECHO | ISIG);
    termios.c_iflag &= !(ICRNL);

    tcsetattr(stdin_fd, TCSANOW, &termios)?;

    Ok(())
}
