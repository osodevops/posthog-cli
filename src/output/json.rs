/// Render data inside the standard PostHog CLI envelope.
pub fn render(data: &serde_json::Value, meta: &serde_json::Value) -> String {
    let envelope = serde_json::json!({
        "ok": true,
        "data": data,
        "meta": meta,
    });
    serde_json::to_string_pretty(&envelope).unwrap_or_default()
}
