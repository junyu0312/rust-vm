use std::io::Read;
use std::io::{self};
use std::sync::mpsc::Sender;
use std::thread;

pub fn init_stdin(tx: Sender<u8>) {
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
}
