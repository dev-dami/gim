# gim - GENERIC INFRASTRUCTURE MONITOR

gim is a fast, high-performance, modular system metrics and diagnostics CLI tool written in Rust.

## Features

- *Modular Design*: Easy to extend with new metric collectors
- *Multiple Output Formats*: JSON, table, and raw output modes
- *Real-time Metrics*: CPU and memory usage statistics
- *Clean Architecture*: Well-structured codebase for easy maintenance

## Use Cases

- Monitor system resources (CPU, memory, disk, network)
- Diagnose performance issues
- Gather metrics for analysis
- Track system health trends

## Installation & Usage

### Build from Source

```bash
# Clone the repository
git clone [https://github.com/your-repo/gim.git](https://github.com/your-repo/gim.git)

# Build the project
cargo build --release

# Run with default settings (now includes CPU, Memory, and Disk)
cargo run

# Run a specific module
cargo run -- --module cpu
cargo run -- --module memory
cargo run -- --module disk # <--- Disk Module 

# Use different output formats
cargo run -- --output json
cargo run -- --output raw
cargo run -- --module cpu --output json