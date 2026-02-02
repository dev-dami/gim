use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "gim",
    author = "Dev-Dami",
    version,
    about = "Generic Infrastructure Monitor — fast, modular system metrics",
    long_about = "gim is a fast, modular system metrics CLI and TUI tool.\n\n\
        Use `gim print` for one-shot CLI output or `gim tui` for an interactive dashboard."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    #[arg(short, long, global = true, help = "Path to config file (YAML)")]
    pub config: Option<PathBuf>,

    #[arg(long, global = true, help = "Ignore config file, use defaults only")]
    pub no_config: bool,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(about = "Print metrics to stdout (default if no subcommand given)")]
    Print {
        #[arg(short, long, value_delimiter = ',', help = "Modules to collect (cpu,memory,disk)")]
        module: Option<Vec<String>>,

        #[arg(short, long, value_enum, help = "Output format")]
        output: Option<OutputFormatArg>,

        #[arg(short, long, help = "Watch mode: refresh periodically")]
        watch: bool,
    },

    #[command(about = "Launch interactive TUI dashboard")]
    Tui {
        #[arg(short, long, value_delimiter = ',', help = "Modules to display (cpu,memory,disk)")]
        module: Option<Vec<String>>,
    },
}

#[derive(Clone, ValueEnum)]
pub enum OutputFormatArg {
    Json,
    Table,
    Raw,
}

pub fn parse_args() -> Cli {
    Cli::parse()
}
