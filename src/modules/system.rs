use crate::core::{MetricCollector, MetricData, MetricValue};
use sysinfo::System;
use std::collections::HashMap;

pub struct SystemCollector;

impl SystemCollector {
    pub fn new() -> Self {
        SystemCollector
    }
}

impl MetricCollector for SystemCollector {
    fn collect(&self) -> Result<MetricData, Box<dyn std::error::Error>> {
        let mut metrics = HashMap::new();

        if let Some(name) = System::name() {
            metrics.insert("os_name".to_string(), MetricValue::String(name));
        }
        if let Some(version) = System::os_version() {
            metrics.insert("os_version".to_string(), MetricValue::String(version));
        }
        if let Some(kernel) = System::kernel_version() {
            metrics.insert("kernel_version".to_string(), MetricValue::String(kernel));
        }
        if let Some(host) = System::host_name() {
            metrics.insert("hostname".to_string(), MetricValue::String(host));
        }
        if let Some(arch) = System::cpu_arch() {
            metrics.insert("arch".to_string(), MetricValue::String(arch));
        }

        let uptime_secs = System::uptime();
        metrics.insert("uptime_seconds".to_string(), MetricValue::Integer(uptime_secs as i64));
        metrics.insert("uptime_human".to_string(), MetricValue::String(format_uptime(uptime_secs)));

        let load = System::load_average();
        metrics.insert("load_1m".to_string(), MetricValue::Float(load.one));
        metrics.insert("load_5m".to_string(), MetricValue::Float(load.five));
        metrics.insert("load_15m".to_string(), MetricValue::Float(load.fifteen));

        Ok(MetricData {
            timestamp: std::time::SystemTime::now(),
            metrics,
        })
    }

    fn name(&self) -> &'static str {
        "system"
    }
}

fn format_uptime(secs: u64) -> String {
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;

    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}
