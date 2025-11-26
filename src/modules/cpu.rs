use crate::core::{MetricCollector, MetricData, MetricValue};
use std::collections::HashMap;
use sysinfo::{CpuExt, CpuRefreshKind, System, SystemExt};

pub struct CpuCollector {}

impl CpuCollector {
    pub fn new() -> Self {
        CpuCollector {}
    }
}

impl MetricCollector for CpuCollector {
    fn collect(&self) -> Result<MetricData, Box<dyn std::error::Error>> {
        let mut sys = System::new();
        // call - Refresh CPU usage information
        sys.refresh_cpu_specifics(CpuRefreshKind::new().with_cpu_usage());

        let mut metrics = HashMap::new();

        // call - Total CPU usage
        let cpus = sys.cpus();
        let avg_cpu_usage: f64 =
            cpus.iter().map(|cpu| cpu.cpu_usage() as f64).sum::<f64>() / cpus.len() as f64;

        metrics.insert(
            "cpu_usage_percent".to_string(),
            MetricValue::from(avg_cpu_usage),
        );
        metrics.insert(
            "cpu_count".to_string(),
            MetricValue::from(cpus.len() as i64),
        );

        // call - Memory information
        metrics.insert(
            "total_memory_bytes".to_string(),
            MetricValue::from(sys.total_memory() as i64),
        );
        metrics.insert(
            "used_memory_bytes".to_string(),
            MetricValue::from(sys.used_memory() as i64),
        );
        metrics.insert(
            "free_memory_bytes".to_string(),
            MetricValue::from(sys.free_memory() as i64),
        );

        metrics.insert(
            "total_swap_bytes".to_string(),
            MetricValue::from(sys.total_swap() as i64),
        );
        metrics.insert(
            "used_swap_bytes".to_string(),
            MetricValue::from(sys.used_swap() as i64),
        );
        metrics.insert(
            "free_swap_bytes".to_string(),
            MetricValue::from(sys.free_swap() as i64),
        );

        Ok(MetricData {
            timestamp: std::time::SystemTime::now(),
            metrics,
        })
    }

    fn name(&self) -> &'static str {
        "cpu"
    }
}
