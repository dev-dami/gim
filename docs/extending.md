# Extending gim

## Overview

gim is designed to be easily extensible. You can add new metric collection modules, output formats, or enhance existing functionality. This guide covers how to extend the system.

## Adding New Metric Collection Modules

### 1. Create the Module File

Create a new file in `src/modules/` (e.g., `disk.rs`):

```rust
use crate::core::{MetricCollector, MetricData, MetricValue};
use std::collections::HashMap;

pub struct DiskCollector {}

impl DiskCollector {
    pub fn new() -> Self {
        DiskCollector {}
    }
}

impl MetricCollector for DiskCollector {
    fn collect(&self) -> Result<MetricData, Box<dyn std::error::Error>> {
        let mut metrics = HashMap::new();
        
        // Collect your metrics
        metrics.insert("disk_usage_percent".to_string(), MetricValue::from(42.5));
        metrics.insert("total_disk_space_bytes".to_string(), MetricValue::from(1000000000i64));
        
        Ok(MetricData {
            timestamp: std::time::SystemTime::now(),
            metrics,
        })
    }

    fn name(&self) -> &'static str {
        "disk"
    }
}
```

### 2. Register the Module

In `src/modules/mod.rs`, add:

```rust
pub mod cpu;
pub mod memory;
pub mod disk;  // Add your module here
```

### 3. Update the Main Application

In `src/lib.rs`, update the module selection logic:

```rust
match args.module.as_deref() {
    Some("cpu") => vec![Box::new(CpuCollector::new())],
    Some("memory") => vec![Box::new(MemoryCollector::new())],
    Some("disk") => vec![Box::new(DiskCollector::new())],  // Add your module
    // ... other cases
    None => {
        // Default to all collectors including your new one
        vec![
            Box::new(CpuCollector::new()),
            Box::new(MemoryCollector::new()),
            Box::new(DiskCollector::new()),  // Add your module to default list
        ]
    }
}
```

## Adding New Output Formats

### 1. Extend the OutputFormat Enum

In `src/output/mod.rs`, add your new format:

```rust
pub enum OutputFormat {
    Table,
    Json,
    Raw,
    YourFormat,  // Add your new format
}
```

### 2. Implement the Formatter

Add a new formatter function and update `format_output`:

```rust
pub fn format_output(data: &MetricData, format: OutputFormat) -> String {
    match format {
        OutputFormat::Json => format_json(data),
        OutputFormat::Table => format_table(data),
        OutputFormat::Raw => format_raw(data),
        OutputFormat::YourFormat => format_your_format(data),  // Add your format
    }
}

fn format_your_format(data: &MetricData) -> String {
    // Implement your formatting logic here
    // Return a formatted string
}
```

### 3. Update CLI Support

In `src/cli/mod.rs`, if you want command-line support for your format:

```rust
// Update the output format validation in src/lib.rs
match args.output.as_deref() {
    Some("json") => OutputFormat::Json,
    Some("table") => OutputFormat::Table,
    Some("raw") => OutputFormat::Raw,
    Some("your_format") => OutputFormat::YourFormat,  // Add your format
    // ... other cases
}
```

## Adding Configuration Support

For adding configuration file support (planned feature), consider:

1. Create a configuration structure in a new module
2. Use `serde` for serialization/deserialization
3. Load configuration at startup
4. Pass configuration to collectors and formatters

## Testing Your Extensions

### Unit Tests

Add unit tests for your new modules:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_your_collector() {
        let collector = YourCollector::new();
        let result = collector.collect();
        assert!(result.is_ok());
        // Add more specific assertions
    }
}
```

### Integration Tests

Consider adding integration tests in the `tests/` directory to test your module end-to-end.

## Dependencies

If your module requires new dependencies, add them to `Cargo.toml`:

```toml
[dependencies]
# existing dependencies...
your_new_dependency = "version"
```

Make sure to follow Rust's dependency management best practices and consider the impact on the final binary size.

## Performance Considerations

- Keep collection operations efficient
- Avoid unnecessary system calls
- Cache values when appropriate
- Consider the impact of your module on overall performance
- Profile your module to identify bottlenecks

## Best Practices

1. Follow Rust idioms and naming conventions
2. Write clear, descriptive metric names
3. Include units in metric names where appropriate
4. Handle errors gracefully
5. Write comprehensive documentation
6. Add unit tests for your module
7. Consider edge cases and error conditions
8. Keep modules focused on a single domain