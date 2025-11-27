# Modules

## Overview

Modules in gim are responsible for collecting specific types of system metrics. Each module implements the `MetricCollector` trait and provides a focused set of related metrics.

## Available Modules

### CPU Module

The CPU module collects processor-related metrics.

**Name**: `"cpu"`

**Collected Metrics**:
- `cpu_usage_percent`: Average CPU usage percentage across all cores
- `cpu_count`: Number of CPU cores available on the system

**Implementation**: `CpuCollector` struct in `src/modules/cpu.rs`

### Memory Module

The Memory module collects system memory usage metrics.

**Name**: `"memory"`

**Collected Metrics**:
- `total_memory_bytes`: Total physical memory installed (in bytes)
- `used_memory_bytes`: Currently used physical memory (in bytes)
- `free_memory_bytes`: Currently free physical memory (in bytes)
- `available_memory_bytes`: Available memory for applications (in bytes)
- `total_swap_bytes`: Total swap space configured (in bytes)
- `used_swap_bytes`: Currently used swap space (in bytes)
- `free_swap_bytes`: Currently free swap space (in bytes)
- `memory_usage_percent`: Percentage of memory currently in use

**Implementation**: `MemoryCollector` struct in `src/modules/memory.rs`

## Creating New Modules

To create a new metric collection module:

### 1. Define Your Collector

Create a new struct in `src/modules/your_module.rs`:

```rust
use crate::core::{MetricCollector, MetricData, MetricValue};
use std::collections::HashMap;

pub struct YourCollector {}

impl YourCollector {
    pub fn new() -> Self {
        YourCollector {}
    }
}
```

### 2. Implement MetricCollector Trait

```rust
impl MetricCollector for YourCollector {
    fn collect(&self) -> Result<MetricData, Box<dyn std::error::Error>> {
        let mut metrics = HashMap::new();
        
        // Collect your metrics here
        metrics.insert("metric_name".to_string(), MetricValue::from(your_value));
        
        Ok(MetricData {
            timestamp: std::time::SystemTime::now(),
            metrics,
        })
    }

    fn name(&self) -> &'static str {
        "your_module_name"
    }
}
```

### 3. Register the Module

Add your module to `src/modules/mod.rs`:

```rust
pub mod cpu;
pub mod memory;
pub mod your_module;  // Add this line
```

### 4. Update Main Application Logic

In `src/lib.rs`, update the module selection logic:

```rust
match args.module.as_deref() {
    Some("cpu") => vec![Box::new(CpuCollector::new())],
    Some("memory") => vec![Box::new(MemoryCollector::new())],
    Some("your_module") => vec![Box::new(YourCollector::new())],  // Add this line
    // ... other cases
}
```

## Best Practices

- Keep modules focused on a single domain of metrics
- Use descriptive names for metrics
- Follow consistent naming conventions (snake_case)
- Include units in metric names when appropriate (e.g., `_bytes`, `_percent`)
- Handle errors gracefully and return appropriate error types
- Document your metrics clearly