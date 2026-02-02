use crate::core::{MetricCollector, MetricData, MetricValue};
use sysinfo::Networks;
use std::collections::HashMap;

pub struct NetworkCollector;

impl NetworkCollector {
    pub fn new() -> Self {
        NetworkCollector
    }
}

impl MetricCollector for NetworkCollector {
    fn collect(&self) -> Result<MetricData, Box<dyn std::error::Error>> {
        let networks = Networks::new_with_refreshed_list();

        let mut total_rx: u64 = 0;
        let mut total_tx: u64 = 0;
        let mut iface_count: i64 = 0;
        let mut iface_details: Vec<MetricValue> = Vec::new();

        for (name, data) in networks.iter() {
            let rx = data.total_received();
            let tx = data.total_transmitted();
            total_rx += rx;
            total_tx += tx;
            iface_count += 1;
            iface_details.push(MetricValue::String(
                format!("{}: rx={} tx={}", name, format_bytes(rx), format_bytes(tx)),
            ));
        }

        let mut metrics = HashMap::new();
        metrics.insert("total_received_bytes".to_string(), MetricValue::Integer(total_rx as i64));
        metrics.insert("total_transmitted_bytes".to_string(), MetricValue::Integer(total_tx as i64));
        metrics.insert("interface_count".to_string(), MetricValue::Integer(iface_count));
        metrics.insert("interfaces".to_string(), MetricValue::List(iface_details));

        Ok(MetricData {
            timestamp: std::time::SystemTime::now(),
            metrics,
        })
    }

    fn name(&self) -> &'static str {
        "network"
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.2}GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.2}MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.2}KB", bytes as f64 / 1024.0)
    } else {
        format!("{}B", bytes)
    }
}
