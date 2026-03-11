use posthog::output;
use posthog::output::OutputFormat;
use serde_json::json;

#[test]
fn test_json_render_envelope() {
    let data = json!({"key": "test-flag", "active": true});
    let meta = json!({"cached": false, "duration_ms": 42});

    let result = output::render(OutputFormat::Json, &data, &meta);
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

    assert_eq!(parsed["ok"], true);
    assert_eq!(parsed["data"]["key"], "test-flag");
    assert_eq!(parsed["meta"]["cached"], false);
    assert_eq!(parsed["meta"]["duration_ms"], 42);
}

#[test]
fn test_json_render_array() {
    let data = json!([
        {"id": 1, "name": "Alpha"},
        {"id": 2, "name": "Beta"},
    ]);
    let meta = json!({"cached": true});

    let result = output::render(OutputFormat::Json, &data, &meta);
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

    assert_eq!(parsed["ok"], true);
    assert!(parsed["data"].is_array());
    assert_eq!(parsed["data"].as_array().unwrap().len(), 2);
}

#[test]
fn test_csv_render_array() {
    let data = json!([
        {"name": "Alice", "age": 30},
        {"name": "Bob", "age": 25},
    ]);
    let meta = json!({});

    let result = output::render(OutputFormat::Csv, &data, &meta);
    // serde_json iterates keys alphabetically, so header is "age,name"
    assert!(result.contains("age"));
    assert!(result.contains("name"));
    assert!(result.contains("Alice"));
    assert!(result.contains("Bob"));
    assert!(result.contains("30"));
    assert!(result.contains("25"));
}

#[test]
fn test_csv_render_with_commas() {
    let data = json!([
        {"name": "Smith, John", "city": "New York"},
    ]);
    let meta = json!({});

    let result = output::render(OutputFormat::Csv, &data, &meta);
    // Name with comma should be quoted
    assert!(result.contains("\"Smith, John\""));
}

#[test]
fn test_jsonl_render_array() {
    let data = json!([
        {"id": 1},
        {"id": 2},
        {"id": 3},
    ]);
    let meta = json!({});

    let result = output::render(OutputFormat::Jsonl, &data, &meta);
    let lines: Vec<&str> = result.lines().collect();
    assert_eq!(lines.len(), 3);

    // Each line should be valid JSON
    for line in &lines {
        let _: serde_json::Value = serde_json::from_str(line).unwrap();
    }
}

#[test]
fn test_jsonl_render_single_object() {
    let data = json!({"status": "ok"});
    let meta = json!({});

    let result = output::render(OutputFormat::Jsonl, &data, &meta);
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(parsed["status"], "ok");
}

#[test]
fn test_table_render_array() {
    let data = json!([
        {"id": 1, "name": "Alpha"},
        {"id": 2, "name": "Beta"},
    ]);
    let meta = json!({});

    let result = output::render(OutputFormat::Table, &data, &meta);
    assert!(result.contains("id"));
    assert!(result.contains("name"));
    assert!(result.contains("Alpha"));
    assert!(result.contains("Beta"));
}

#[test]
fn test_table_render_object() {
    let data = json!({"key": "value", "count": 42});
    let meta = json!({});

    let result = output::render(OutputFormat::Table, &data, &meta);
    assert!(result.contains("Key"));
    assert!(result.contains("Value"));
    assert!(result.contains("key"));
    assert!(result.contains("value"));
    assert!(result.contains("count"));
    assert!(result.contains("42"));
}

#[test]
fn test_render_value_json() {
    let value = json!({"name": "test", "active": true});
    let result = output::render_value(OutputFormat::Json, &value);
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(parsed["name"], "test");
}
