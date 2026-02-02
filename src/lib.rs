pub mod cli;
pub mod config;
pub mod core;
pub mod engine;
pub mod error;
pub mod modules;
pub mod output;
pub mod tui;

use crate::cli::{Cli, Command};
use crate::config::load_config;
use crate::engine::Engine;
use crate::error::Result;
use crate::output::{format_snapshot, OutputFormat};

pub fn run(args: Cli) -> Result<()> {
    let config = if args.no_config {
        config::Config::default()
    } else {
        load_config(args.config.as_deref())?
    };

    match args.command {
        Some(Command::Tui { module }) => {
            let modules = resolve_modules(module, &config);
            let engine = Engine::new(&modules)?;
            tui::run_tui(engine, config)
        }
        Some(Command::Print {
            module,
            output,
            watch,
        }) => {
            let modules = resolve_modules(module, &config);
            let engine = Engine::new(&modules)?;
            let format = match output {
                Some(fmt) => OutputFormat::from(fmt),
                None => OutputFormat::from_str_lossy(&config.print.output),
            };

            if watch || config.print.watch {
                run_watch(engine, format, config.general.refresh_ms)
            } else {
                run_print_once(engine, format)
            }
        }
        None => {
            let modules = resolve_modules(None, &config);
            let engine = Engine::new(&modules)?;
            let format = OutputFormat::from_str_lossy(&config.print.output);
            run_print_once(engine, format)
        }
    }
}

fn resolve_modules(cli_modules: Option<Vec<String>>, config: &config::Config) -> Vec<String> {
    cli_modules.unwrap_or_else(|| config.general.default_modules.clone())
}

fn run_print_once(engine: Engine, format: OutputFormat) -> Result<()> {
    let snapshot = engine.collect_once();
    print!("{}", format_snapshot(&snapshot, &format));
    Ok(())
}

fn run_watch(engine: Engine, format: OutputFormat, refresh_ms: u64) -> Result<()> {
    let duration = std::time::Duration::from_millis(refresh_ms);

    crossterm::terminal::enable_raw_mode()?;

    let result = (|| -> Result<()> {
        loop {
            print!("\x1B[2J\x1B[1;1H");
            let snapshot = engine.collect_once();
            print!("{}", format_snapshot(&snapshot, &format));

            if crossterm::event::poll(duration)? {
                if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                    if key.code == crossterm::event::KeyCode::Char('q')
                        || key.code == crossterm::event::KeyCode::Esc
                    {
                        break;
                    }
                }
            }
        }
        Ok(())
    })();

    crossterm::terminal::disable_raw_mode()?;
    result
}
