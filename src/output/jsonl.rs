/// Render data as newline-delimited JSON (one JSON object per line).
pub fn render(data: &serde_json::Value) -> String {
    match data {
        serde_json::Value::Array(items) => items
            .iter()
            .map(|v| serde_json::to_string(v).unwrap_or_default())
            .collect::<Vec<_>>()
            .join("\n"),
        other => serde_json::to_string(other).unwrap_or_default(),
    }
}
