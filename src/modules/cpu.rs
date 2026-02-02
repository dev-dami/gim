use crate::core::{MetricCollector, MetricData, MetricValue};
use std::collections::HashMap;
use sysinfo::{CpuRefreshKind, System, RefreshKind};

pub struct CpuCollector;

impl CpuCollector {
    pub fn new() -> Self {
        CpuCollector
    }
}

impl MetricCollector for CpuCollector {
    fn collect(&self) -> Result<MetricData, Box<dyn std::error::Error>> {
        let mut sys = System::new_with_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::everything()),
        );

        // Sleep briefly to allow CPU usage to be calculated accurately
        std::thread::sleep(std::time::Duration::from_millis(200));
        sys.refresh_cpu_all();

        let mut metrics = HashMap::new();

        let cpus = sys.cpus();
        let avg_usage: f64 = if cpus.is_empty() {
            0.0
        } else {
            cpus.iter().map(|cpu| cpu.cpu_usage() as f64).sum::<f64>() / cpus.len() as f64
        };

        metrics.insert("cpu_usage_percent".to_string(), MetricValue::Float(avg_usage));
        metrics.insert("cpu_count".to_string(), MetricValue::Integer(cpus.len() as i64));

        let per_core: Vec<MetricValue> = cpus
            .iter()
            .enumerate()
            .map(|(i, cpu)| {
                MetricValue::String(format!("core_{}: {:.1}%", i, cpu.cpu_usage()))
            })
            .collect();
        metrics.insert("per_core_usage".to_string(), MetricValue::List(per_core));

        if let Some(first_cpu) = cpus.first() {
            metrics.insert("cpu_brand".to_string(), MetricValue::String(first_cpu.brand().to_string()));
        }

        Ok(MetricData {
            timestamp: std::time::SystemTime::now(),
            metrics,
        })
    }

    fn name(&self) -> &'static str {
        "cpu"
    }
}
