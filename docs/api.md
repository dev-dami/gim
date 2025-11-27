# gim API Documentation

## Overview

gim is a modular system metrics collection library with a CLI interface. The API is designed to be extensible, allowing for new metric collectors to be easily added while maintaining a consistent interface.

## Core Components

### MetricCollector Trait

The `MetricCollector` trait is the core abstraction that all metric collection modules implement:

```rust
pub trait MetricCollector {
    fn collect(&self) -> Result<MetricData, Box<dyn std::error::Error>>;
    fn name(&self) -> &'static str;
}
```

- `collect()`: Gathers metrics and returns them as `MetricData`
- `name()`: Returns the identifier for the collector (e.g., "cpu", "memory")

### MetricData Structure

Represents the collected metrics with timestamp:

```rust
pub struct MetricData {
    pub timestamp: std::time::SystemTime,
    pub metrics: HashMap<String, MetricValue>,
}
```

### MetricValue Enum

A flexible type to represent different metric value types:

```rust
pub enum MetricValue {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    List(Vec<MetricValue>),
}
```

## Available Modules

### CPU Collector

Collects CPU-related metrics:

- `cpu_usage_percent`: Average CPU usage percentage
- `cpu_count`: Number of CPU cores
- Memory-related metrics are also available (total, used, free, swap)

### Memory Collector

Collects memory-related metrics:

- `total_memory_bytes`: Total system memory in bytes
- `used_memory_bytes`: Currently used memory in bytes
- `free_memory_bytes`: Currently free memory in bytes
- `available_memory_bytes`: Available memory for applications
- `total_swap_bytes`, `used_swap_bytes`, `free_swap_bytes`: Swap memory metrics
- `memory_usage_percent`: Memory usage percentage

## Output Formats

The system supports multiple output formats through the `OutputFormat` enum:

- `Table`: Formatted table with headers
- `Json`: JSON output with timestamp and metrics
- `Raw`: Raw key=value format

## CLI Interface

Command line arguments are handled by the `Cli` struct:

```rust
pub struct Cli {
    pub module: Option<String>,  // Module to run (cpu, memory, etc.)
    pub output: Option<String>,  // Output format (json, table, raw)
}
```

### Usage Examples

```bash
# Run with default modules and table output
cargo run

# Run specific module
cargo run -- --module cpu

# Use specific output format
cargo run -- --output json

# Combine module and output format
cargo run -- --module memory --output raw
```

## Adding New Modules

To add a new metric collection module:

1. Create a struct that implements `MetricCollector`
2. Implement the required methods (`collect` and `name`)
3. Register the module in the main application logic
4. Optionally add CLI support for the new module

## Library Functions

### `run()`

Main entry point that handles command parsing, module selection, and output formatting.

### `format_output(data: &MetricData, format: OutputFormat)`

Formats metric data according to the specified output format.