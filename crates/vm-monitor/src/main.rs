use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use tokio::io::AsyncReadExt;
use tokio::net::UnixStream;

const PATH: &'static str = "/tmp/vm.sock";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut rl = DefaultEditor::new()?;

    let mut stream = UnixStream::connect(PATH).await?;

    loop {
        let readline = rl.readline("vmm>> ");
        match readline {
            Ok(line) => {
                let cmd = line.trim();

                if cmd.is_empty() {
                    continue;
                }

                let _ = rl.add_history_entry(line.as_str());

                stream.writable().await?;
                stream.try_write(line.as_bytes())?;

                stream.readable().await?;
                let mut buf = vec![0u8; 1024];
                let n = stream.read_buf(&mut buf).await?;
                println!("{}", str::from_utf8(&buf[..n])?);
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(err) => {
                eprintln!("{err}");
                break;
            }
        }
    }

    Ok(())
}
