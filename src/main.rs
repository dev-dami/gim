use clap::Parser;

fn main() {
    let args = gim::cli::Cli::parse();

    if let Err(e) = gim::run(args) {
        eprintln!("error: {e}");
        std::process::exit(e.exit_code());
    }
}
