/// Output formatting - Interface Segregation Principle
/// Dedicated abstraction for output formatting, independent from command logic
use serde_json::Value;

pub trait OutputFormatter {
    fn format(&self, data: Value) -> String;
}

pub struct JsonFormatter;
pub struct TextFormatter;
pub struct YamlFormatter;

impl OutputFormatter for JsonFormatter {
    fn format(&self, data: Value) -> String {
        serde_json::to_string_pretty(&data).unwrap_or_else(|_| "{}".to_string())
    }
}

impl OutputFormatter for TextFormatter {
    fn format(&self, data: Value) -> String {
        match data {
            Value::Object(obj) => {
                let mut output = String::new();
                
                // Handle status and count
                if let Some(status) = obj.get("status") {
                    output.push_str(&format!("Status: {}\n", status));
                }
                if let Some(count) = obj.get("count") {
                    output.push_str(&format!("Count: {}\n", count));
                }
                
                // Handle data array
                if let Some(Value::Array(items)) = obj.get("data") {
                    for item in items {
                        if let Value::Object(item_obj) = item {
                            for (key, value) in item_obj.iter() {
                                output.push_str(&format!("  {}: {}\n", key, value));
                            }
                            output.push('\n');
                        }
                    }
                }
                // Handle data object
                else if let Some(data_obj) = obj.get("data") {
                    output.push_str(&format!("{}\n", serde_json::to_string_pretty(data_obj).unwrap_or_default()));
                }
                
                output
            }
            _ => data.to_string(),
        }
    }
}

impl OutputFormatter for YamlFormatter {
    fn format(&self, data: Value) -> String {
        // Convert JSON to YAML string representation
        match serde_yaml::to_string(&data) {
            Ok(yaml) => yaml,
            Err(_) => serde_json::to_string_pretty(&data).unwrap_or_else(|_| "{}".to_string()),
        }
    }
}

pub fn get_formatter(format: &str) -> Box<dyn OutputFormatter> {
    match format.to_lowercase().as_str() {
        "json" => Box::new(JsonFormatter),
        "text" | "txt" => Box::new(TextFormatter),
        "yaml" | "yml" => Box::new(YamlFormatter),
        _ => Box::new(JsonFormatter), // Default to JSON
    }
}
