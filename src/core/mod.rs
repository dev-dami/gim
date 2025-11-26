use std::collections::HashMap;

pub trait MetricCollector {
    fn collect(&self) -> Result<MetricData, Box<dyn std::error::Error>>;
    fn name(&self) -> &'static str;
}

#[derive(Debug, Clone)]
pub struct MetricData {
    pub timestamp: std::time::SystemTime,
    pub metrics: HashMap<String, MetricValue>,
}

#[derive(Debug, Clone)]
pub enum MetricValue {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    List(Vec<MetricValue>),
}

impl From<i64> for MetricValue {
    fn from(value: i64) -> Self {
        MetricValue::Integer(value)
    }
}

impl From<f64> for MetricValue {
    fn from(value: f64) -> Self {
        MetricValue::Float(value)
    }
}

impl From<String> for MetricValue {
    fn from(value: String) -> Self {
        MetricValue::String(value)
    }
}

impl From<&str> for MetricValue {
    fn from(value: &str) -> Self {
        MetricValue::String(value.to_string())
    }
}

impl From<bool> for MetricValue {
    fn from(value: bool) -> Self {
        MetricValue::Boolean(value)
    }
}
