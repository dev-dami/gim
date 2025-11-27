use clap::Parser;

#[derive(Parser)]
#[command(
    name = "gim",
    author = "Dev-Dami",
    version = "0.0.1",
    about = "System metrics CLI"
)]
pub struct Cli {
    /// module to run (cpu, memory, etc.)
    #[arg(short, long)]
    pub module: Option<String>,

    /// output format (json, table, raw)
    #[arg(short, long)]
    pub output: Option<String>,
}

pub fn parse_args() -> Cli {
    Cli::parse()
}
