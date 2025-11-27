# Contributing to gim

Thank you for your interest in contributing to gim! This document outlines the process and guidelines for contributing to the project.

## Project Overview

gim is a fast, high-performance, modular system metrics and diagnostics CLI tool written in Rust. It collects and displays system metrics like CPU, memory, disk, and network usage.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/your-username/gim.git`
3. Create a new branch: `git checkout -b feature/your-feature-name`
4. Make your changes
5. Test thoroughly
6. Submit a pull request

## Development Setup

```bash
# Install Rust if you don't have it
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build the project
cargo build

# Run the project
cargo run

# Run tests
cargo test

# Format code
cargo fmt

# Check for linting issues
cargo clippy
```

## Project Structure

```
src/
├── lib.rs          # Main application logic
├── main.rs         # Entry point
├── cli/            # Command line interface parsing
├── core/           # Core data structures and traits
├── modules/        # Metric collection modules
├── output/         # Output formatting logic
└── tui/            # Terminal UI (planned future feature)
```

## Adding New Modules

To add a new metric collection module:

1. Create a new file in `src/modules/` (e.g., `disk.rs`)
2. Implement the `MetricCollector` trait
3. Add the module to `src/modules/mod.rs`
4. Register the module in the main function in `src/lib.rs`
5. Update the CLI to accept your module as an argument (if needed)

Example module implementation:
```rust
use crate::core::{MetricCollector, MetricData, MetricValue};
use std::collections::HashMap;

pub struct NewCollector {}

impl NewCollector {
    pub fn new() -> Self {
        NewCollector {}
    }
}

impl MetricCollector for NewCollector {
    fn collect(&self) -> Result<MetricData, Box<dyn std::error::Error>> {
        let mut metrics = HashMap::new();
        
        // Collect your metrics here
        metrics.insert("metric_name".to_string(), MetricValue::from(42));
        
        Ok(MetricData {
            timestamp: std::time::SystemTime::now(),
            metrics,
        })
    }

    fn name(&self) -> &'static str {
        "module_name"
    }
}
```

## Code Style

- Follow Rust idioms and best practices
- Use `cargo fmt` to format code
- Add documentation comments for public APIs
- Write tests for new functionality
- Keep functions focused and well-named

## Pull Request Guidelines

1. Describe your changes clearly in the PR description
2. Include tests if adding new functionality
3. Ensure all tests pass before submitting
4. Link any relevant issues
5. Keep PRs focused on a single feature or fix

## Questions?

If you have questions, feel free to open an issue or reach out to the maintainers.