use crate::core::{MetricCollector, MetricData, MetricValue};
use std::collections::HashMap;
use sysinfo::{System, RefreshKind};

pub struct MemoryCollector {}

impl MemoryCollector {
    pub fn new() -> Self {
        MemoryCollector {}
    }
}

impl MetricCollector for MemoryCollector {
    fn collect(&self) -> Result<MetricData, Box<dyn std::error::Error>> {
        let mut sys = System::new_with_specifics(RefreshKind::new().with_memory());

        let mut metrics = HashMap::new();

        metrics.insert("total_memory_bytes".to_string(), MetricValue::from(sys.total_memory() as i64));
        metrics.insert("used_memory_bytes".to_string(), MetricValue::from(sys.used_memory() as i64));
        metrics.insert("free_memory_bytes".to_string(), MetricValue::from(sys.free_memory() as i64));
        metrics.insert("available_memory_bytes".to_string(), MetricValue::from(sys.available_memory() as i64));

        metrics.insert("total_swap_bytes".to_string(), MetricValue::from(sys.total_swap() as i64));
        metrics.insert("used_swap_bytes".to_string(), MetricValue::from(sys.used_swap() as i64));
        metrics.insert("free_swap_bytes".to_string(), MetricValue::from(sys.free_swap() as i64));

        // Calculate memory usage percentage
        let memory_percent = if sys.total_memory() > 0 {
            (sys.used_memory() as f64 / sys.total_memory() as f64) * 100.0
        } else {
            0.0
        };
        metrics.insert("memory_usage_percent".to_string(), MetricValue::from(memory_percent));

        Ok(MetricData {
            timestamp: std::time::SystemTime::now(),
            metrics,
        })
    }

    fn name(&self) -> &'static str {
        "memory"
    }
}
