# gim - GENERIC INFRASTRUCTURE MONITOR

gmi-cli is a fast, high performance, modular system metrics and diagnostics CLI tool written in Rust.

# why - use cases
- Monitoring system resources such as CPU, memory, disk usage, network traffic, and more.
- Diagnosing issues with system performance and stability.
- Gathering metrics for performance analysis and optimization.
- Providing insights into system health and performance trends.
- Automating routine tasks and alerts for system monitoring.

# planned features
- CPU, memory, disk, and network metric collectors  
- Unified MetricCollector trait for easy module additions  
- JSON, table, and raw output modes  
- Optional ratatui-based live dashboard  
- JSON config support

# Build & Run

```bash
# Build the project
cargo build

# Run with default settings
cargo run

# Run a specific module
cargo run -- --module cpu
cargo run -- --module memory

# Use different output formats
cargo run -- --output json
cargo run -- --output raw
cargo run -- --module cpu --output json
```

# Current Features
-  CPU metric collection (usage, core count)
-  Memory metric collection (usage, available/free/used, swap)
-  Unified MetricCollector trait for modules
-  Multiple output formats (JSON, table, raw)
-  CLI argument parsing with clap
-  Basic architecture with core, modules, and output components

# Next Steps
- Disk and network metric collectors
- TUI dashboard implementation
- Configuration file support
- Live monitoring capabilities
