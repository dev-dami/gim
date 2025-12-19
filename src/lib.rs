pub mod cli;
pub mod core;
pub mod modules;
pub mod output;
pub mod tui;

use crate::core::MetricCollector;
use crate::modules::disk::DiskCollector;
use crate::modules::{cpu::CpuCollector, memory::MemoryCollector};
use crate::output::{OutputFormat, format_output};

pub fn run() {
    let args = cli::parse_args();

    // get list of collectors based on args
   let collectors: Vec<Box<dyn MetricCollector>> = match args.module.as_deref() {
    Some("cpu") => vec![Box::new(CpuCollector::new())],
    Some("memory") => vec![Box::new(MemoryCollector::new())],
    Some("disk") => vec![Box::new(DiskCollector::new())],
    Some(_) => {
        eprintln!("Unknown module: {}", args.module.unwrap());
        std::process::exit(1);
    }
    None => {
        vec![
            Box::new(CpuCollector::new()),
            Box::new(MemoryCollector::new()),
            Box::new(DiskCollector::new()),    // default me disk bhi add
        ]
    }
};


    // determine output format
    let format = match args.output.as_deref() {
        Some("json") => OutputFormat::Json,
        Some("table") => OutputFormat::Table,
        Some("raw") => OutputFormat::Raw,
        Some(_) => {
            eprintln!("Unknown output format: {}", args.output.unwrap());
            std::process::exit(1);
        }
        None => OutputFormat::Table,
    };

    // collect and display metrics
    for collector in collectors {
        match collector.collect() {
            Ok(data) => {
                println!("=== {} Metrics ===", collector.name());
                println!("{}", format_output(&data, format.clone()));
                println!();
            }
            Err(e) => eprintln!("Error collecting {} metrics: {}", collector.name(), e),
        }
    }
}