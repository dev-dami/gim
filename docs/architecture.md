# Architecture

## Overview

gim follows a modular architecture with clear separation of concerns between metric collection, output formatting, and command-line interface handling.

## Component Diagram

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   CLI       │───▶│   Core      │───▶│  Output     │
│  Parsing    │    │  Metrics    │    │ Formatting  │
└─────────────┘    └─────────────┘    └─────────────┘
                        │
                        ▼
                ┌─────────────┐
                │  Modules    │
                │(CPU,Memory) │
                └─────────────┘
```

## Core Components

### 1. CLI Module
- Handles command-line argument parsing using `clap`
- Defines the `Cli` struct with `module` and `output` options
- Provides `parse_args()` function to parse command line

### 2. Core Module
- Defines the `MetricCollector` trait that all collectors implement
- Provides `MetricData` struct to encapsulate collected metrics
- Defines `MetricValue` enum for flexible metric types

### 3. Modules
- Individual metric collection implementations (CPU, Memory, etc.)
- Each implements the `MetricCollector` trait
- Responsible for gathering specific system metrics
- Return structured data via `MetricData`

### 4. Output Module
- Handles formatting of collected metrics
- Supports multiple output formats: JSON, Table, Raw
- Contains formatter functions for each format type

## Data Flow

1. **Initialization**: CLI arguments are parsed to determine which modules to run and output format
2. **Collection**: Each selected module's `collect()` method is called to gather metrics
3. **Formatting**: Collected `MetricData` is formatted according to the specified output format
4. **Display**: Formatted output is printed to stdout

## Extensibility

The architecture is designed for easy extension:
- New modules can be added by implementing the `MetricCollector` trait
- New output formats can be added to the `OutputFormat` enum and formatter functions
- The core logic remains unchanged when adding new functionality

## Key Data Structures

### MetricData
```rust
struct MetricData {
    timestamp: SystemTime,
    metrics: HashMap<String, MetricValue>
}
```

### MetricValue
```rust
enum MetricValue {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    List(Vec<MetricValue>)
}
```