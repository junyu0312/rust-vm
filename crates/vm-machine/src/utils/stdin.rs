use std::io::Read;
use std::io::{self};
use std::os::fd::AsRawFd;
use std::sync::mpsc::Sender;
use std::thread;

use ::termios::Termios;
use ::termios::cfmakeraw;
use ::termios::tcsetattr;
use nix::libc::STDIN_FILENO;
use termios::TCSANOW;

fn disable_stdin_echo() -> anyhow::Result<()> {
    let fd = std::io::stdin().as_raw_fd();
    let mut termios = Termios::from_fd(fd)?;
    cfmakeraw(&mut termios);
    tcsetattr(STDIN_FILENO, TCSANOW, &termios)?;

    Ok(())
}

#[allow(dead_code)]
pub fn init_stdin(tx: Sender<u8>) -> anyhow::Result<()> {
    disable_stdin_echo()?;

    thread::spawn(move || {
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        let mut buffer = [0u8; 1]; // 按字节读取

        while let Ok(n) = handle.read(&mut buffer) {
            if n == 0 {
                break;
            }
            tx.send(buffer[0]).unwrap();
        }
    });

    Ok(())
}
