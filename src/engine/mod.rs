use crate::core::{MetricCollector, MetricData};
use crate::error::{GimError, Result};
use crate::modules::cpu::CpuCollector;
use crate::modules::disk::DiskCollector;
use crate::modules::memory::MemoryCollector;
use crate::modules::network::NetworkCollector;
use crate::modules::process::ProcessCollector;
use crate::modules::system::SystemCollector;

pub struct MetricsSnapshot {
    pub modules: Vec<(String, MetricData)>,
}

pub struct Engine {
    collectors: Vec<Box<dyn MetricCollector>>,
}

impl Engine {
    pub fn new(module_names: &[String]) -> Result<Self> {
        let mut collectors: Vec<Box<dyn MetricCollector>> = Vec::new();

        for name in module_names {
            match name.as_str() {
                "cpu" => collectors.push(Box::new(CpuCollector::new())),
                "memory" => collectors.push(Box::new(MemoryCollector::new())),
                "disk" => collectors.push(Box::new(DiskCollector::new())),
                "network" => collectors.push(Box::new(NetworkCollector::new())),
                "process" => collectors.push(Box::new(ProcessCollector::new())),
                "system" => collectors.push(Box::new(SystemCollector::new())),
                other => return Err(GimError::UnknownModule(other.to_string())),
            }
        }

        Ok(Self { collectors })
    }

    pub fn collect_once(&self) -> MetricsSnapshot {
        let mut modules = Vec::new();

        for collector in &self.collectors {
            match collector.collect() {
                Ok(data) => {
                    modules.push((collector.name().to_string(), data));
                }
                Err(e) => {
                    eprintln!("warning: {} collection failed: {}", collector.name(), e);
                }
            }
        }

        MetricsSnapshot { modules }
    }

    pub fn module_names(&self) -> Vec<&str> {
        self.collectors.iter().map(|c| c.name()).collect()
    }
}

pub const AVAILABLE_MODULES: &[&str] = &["cpu", "memory", "disk", "network", "process", "system"];
