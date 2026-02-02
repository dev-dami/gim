use crate::cli::OutputFormatArg;
use crate::core::{MetricData, MetricValue};
use crate::engine::MetricsSnapshot;

#[derive(Clone)]
pub enum OutputFormat {
    Table,
    Json,
    Raw,
}

impl From<OutputFormatArg> for OutputFormat {
    fn from(arg: OutputFormatArg) -> Self {
        match arg {
            OutputFormatArg::Json => OutputFormat::Json,
            OutputFormatArg::Table => OutputFormat::Table,
            OutputFormatArg::Raw => OutputFormat::Raw,
        }
    }
}

impl OutputFormat {
    pub fn from_str_lossy(s: &str) -> Self {
        match s {
            "json" => OutputFormat::Json,
            "raw" => OutputFormat::Raw,
            _ => OutputFormat::Table,
        }
    }
}

pub fn format_snapshot(snapshot: &MetricsSnapshot, format: &OutputFormat) -> String {
    let mut output = String::new();
    for (name, data) in &snapshot.modules {
        output.push_str(&format!("=== {} ===\n", name.to_uppercase()));
        output.push_str(&format_output(data, format));
        output.push('\n');
    }
    output
}

pub fn format_output(data: &MetricData, format: &OutputFormat) -> String {
    match format {
        OutputFormat::Json => format_json(data),
        OutputFormat::Table => format_table(data),
        OutputFormat::Raw => format_raw(data),
    }
}

fn format_json(data: &MetricData) -> String {
    let mut output = std::collections::HashMap::new();
    output.insert(
        "timestamp".to_string(),
        serde_json::Value::String(format!("{:?}", data.timestamp)),
    );

    let mut metrics = serde_json::Map::new();
    for (key, value) in &data.metrics {
        metrics.insert(key.clone(), metric_value_to_json(value));
    }
    output.insert("metrics".to_string(), serde_json::Value::Object(metrics));

    serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_string())
}

fn metric_value_to_json(value: &MetricValue) -> serde_json::Value {
    match value {
        MetricValue::Integer(i) => serde_json::Value::Number((*i).into()),
        MetricValue::Float(f) => serde_json::Number::from_f64(*f)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::String(f.to_string())),
        MetricValue::String(s) => serde_json::Value::String(s.clone()),
        MetricValue::Boolean(b) => serde_json::Value::Bool(*b),
        MetricValue::List(items) => {
            serde_json::Value::Array(items.iter().map(metric_value_to_json).collect())
        }
    }
}

fn format_table(data: &MetricData) -> String {
    let mut output = String::new();

    let mut entries: Vec<_> = data.metrics.iter().collect();
    entries.sort_by_key(|(k, _)| k.clone());

    let max_key_len = entries.iter().map(|(k, _)| k.len()).max().unwrap_or(20);
    let width = max_key_len.max(20);

    output.push_str(&format!("{:─<w$}┬{:─<40}\n", "", "", w = width + 2));
    for (key, value) in &entries {
        output.push_str(&format!(
            " {:<w$} │ {}\n",
            key,
            metric_value_to_display(value),
            w = width,
        ));
    }
    output.push_str(&format!("{:─<w$}┴{:─<40}\n", "", "", w = width + 2));

    output
}

fn format_raw(data: &MetricData) -> String {
    let mut entries: Vec<_> = data.metrics.iter().collect();
    entries.sort_by_key(|(k, _)| k.clone());

    entries
        .iter()
        .map(|(key, value)| format!("{}={}", key, metric_value_to_display(value)))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

fn metric_value_to_display(value: &MetricValue) -> String {
    match value {
        MetricValue::Integer(i) => format_bytes_or_int(*i),
        MetricValue::Float(f) => format!("{:.2}", f),
        MetricValue::String(s) => s.clone(),
        MetricValue::Boolean(b) => b.to_string(),
        MetricValue::List(items) => items
            .iter()
            .map(metric_value_to_display)
            .collect::<Vec<_>>()
            .join(", "),
    }
}

fn format_bytes_or_int(value: i64) -> String {
    let abs = value.unsigned_abs();
    if abs >= 1_073_741_824 {
        format!("{:.2} GB", abs as f64 / 1_073_741_824.0)
    } else if abs >= 1_048_576 {
        format!("{:.2} MB", abs as f64 / 1_048_576.0)
    } else if abs >= 1024 {
        format!("{:.2} KB", abs as f64 / 1024.0)
    } else {
        value.to_string()
    }
}
