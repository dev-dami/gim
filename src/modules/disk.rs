use sysinfo::Disks;
use std::collections::HashMap;
use crate::core::{MetricCollector, MetricData, MetricValue};

pub struct DiskCollector;

impl DiskCollector {
    pub fn new() -> Self {
        DiskCollector
    }
}

impl MetricCollector for DiskCollector {
    fn collect(&self) -> Result<MetricData, Box<dyn std::error::Error>> {
        let disks = Disks::new_with_refreshed_list();

        let mut total: u64 = 0;
        let mut free: u64 = 0;

        for disk in disks.list() {
            total += disk.total_space();
            free += disk.available_space();
        }

        let used = total.saturating_sub(free);
        let usage_percent = if total > 0 {
            (used as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        let mut metrics = HashMap::new();
        metrics.insert("total_bytes".to_string(), MetricValue::Integer(total as i64));
        metrics.insert("used_bytes".to_string(), MetricValue::Integer(used as i64));
        metrics.insert("free_bytes".to_string(), MetricValue::Integer(free as i64));
        metrics.insert("usage_percent".to_string(), MetricValue::Float(usage_percent));
        metrics.insert("disk_count".to_string(), MetricValue::Integer(disks.list().len() as i64));

        Ok(MetricData {
            timestamp: std::time::SystemTime::now(),
            metrics,
        })
    }

    fn name(&self) -> &'static str {
        "disk"
    }
}
