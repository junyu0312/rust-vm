use clap::Parser;

#[derive(Parser, Debug)]
pub struct Command {
    #[arg(short, long)]
    pub memory: usize,
}
