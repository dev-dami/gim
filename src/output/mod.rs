use crate::core::{MetricData, MetricValue};
use serde_json;

#[derive(Clone)]
pub enum OutputFormat {
    Table,
    Json,
    Raw,
}

pub fn format_output(data: &MetricData, format: OutputFormat) -> String {
    match format {
        OutputFormat::Json => format_json(data),
        OutputFormat::Table => format_table(data),
        OutputFormat::Raw => format_raw(data),
    }
}

fn format_json(data: &MetricData) -> String {
    // convert metricdata to a serializable format
    use std::collections::HashMap;

    let mut output = std::collections::HashMap::new();
    output.insert("timestamp".to_string(), serde_json::Value::String(format!("{:?}", data.timestamp)));

    let mut metrics = std::collections::HashMap::new();
    for (key, value) in &data.metrics {
        match value {
            MetricValue::Integer(i) => {
                metrics.insert(key.clone(), serde_json::Value::Number((*i).into()));
            }
            MetricValue::Float(f) => {
                if let Some(num) = serde_json::Number::from_f64(*f) {
                    metrics.insert(key.clone(), serde_json::Value::Number(num));
                } else {
                    metrics.insert(key.clone(), serde_json::Value::String(f.to_string()));
                }
            }
            MetricValue::String(s) => {
                metrics.insert(key.clone(), serde_json::Value::String(s.clone()));
            }
            MetricValue::Boolean(b) => {
                metrics.insert(key.clone(), serde_json::Value::Bool(*b));
            }
            MetricValue::List(_) => {
                metrics.insert(
                    key.clone(),
                    serde_json::Value::String("List not supported in JSON output".to_string()),
                );
            }
        }
    }
    output.insert(
        "metrics".to_string(),
        serde_json::Value::Object(metrics.into_iter().collect()),
    );

    serde_json::to_string_pretty(&output)
        .unwrap_or_else(|_| "Error serializing to JSON".to_string())
}

fn format_table(data: &MetricData) -> String {
    let mut output = String::new();
    output.push_str(&format!("Timestamp: {:?}\n", data.timestamp));
    output.push_str("Metrics:\n");
    output.push_str("----------\n");

    for (key, value) in &data.metrics {
        output.push_str(&format!(
            "{:<25} | {}\n",
            key,
            metric_value_to_string(value)
        ));
    }

    output
}

fn format_raw(data: &MetricData) -> String {
    let mut output = String::new();

    for (key, value) in &data.metrics {
        output.push_str(&format!("{}={}\n", key, metric_value_to_string(value)));
    }

    output
}

fn metric_value_to_string(value: &MetricValue) -> String {
    match value {
        MetricValue::Integer(i) => i.to_string(),
        MetricValue::Float(f) => format!("{:.2}", f),
        MetricValue::String(s) => s.clone(),
        MetricValue::Boolean(b) => b.to_string(),
        MetricValue::List(l) => format!("{:?}", l), // simple representation for lists
    }
}
