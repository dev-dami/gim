use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "gim",
    author = "Dev-Dami",
    version = "0.0.1",
    about = "System metrics CLI"
)]
pub struct Cli {
    #[arg(short, long)]
    pub module: Option<String>,

    #[arg(short, long)]
    pub output: Option<String>,
}

pub fn parse_args() -> Cli {
    Cli::parse()
}
