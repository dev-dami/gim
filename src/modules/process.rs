use crate::core::{MetricCollector, MetricData, MetricValue};
use sysinfo::{System, RefreshKind, ProcessRefreshKind};
use std::collections::HashMap;

pub struct ProcessCollector;

impl ProcessCollector {
    pub fn new() -> Self {
        ProcessCollector
    }
}

impl MetricCollector for ProcessCollector {
    fn collect(&self) -> Result<MetricData, Box<dyn std::error::Error>> {
        let sys = System::new_with_specifics(
            RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
        );

        let processes = sys.processes();
        let total = processes.len() as i64;

        let mut by_memory: Vec<_> = processes.values().collect();
        by_memory.sort_by(|a, b| b.memory().cmp(&a.memory()));

        let top_mem: Vec<MetricValue> = by_memory
            .iter()
            .take(5)
            .map(|p| {
                MetricValue::String(format!(
                    "{} (pid {}) — {:.1}MB",
                    p.name().to_string_lossy(),
                    p.pid(),
                    p.memory() as f64 / 1_048_576.0,
                ))
            })
            .collect();

        let mut by_cpu: Vec<_> = processes.values().collect();
        by_cpu.sort_by(|a, b| b.cpu_usage().partial_cmp(&a.cpu_usage()).unwrap_or(std::cmp::Ordering::Equal));

        let top_cpu: Vec<MetricValue> = by_cpu
            .iter()
            .take(5)
            .map(|p| {
                MetricValue::String(format!(
                    "{} (pid {}) — {:.1}%",
                    p.name().to_string_lossy(),
                    p.pid(),
                    p.cpu_usage(),
                ))
            })
            .collect();

        let mut metrics = HashMap::new();
        metrics.insert("total_processes".to_string(), MetricValue::Integer(total));
        metrics.insert("top_by_memory".to_string(), MetricValue::List(top_mem));
        metrics.insert("top_by_cpu".to_string(), MetricValue::List(top_cpu));

        Ok(MetricData {
            timestamp: std::time::SystemTime::now(),
            metrics,
        })
    }

    fn name(&self) -> &'static str {
        "process"
    }
}
