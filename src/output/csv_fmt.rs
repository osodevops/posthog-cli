/// Render data as CSV.
pub fn render(data: &serde_json::Value) -> String {
    match data {
        serde_json::Value::Array(items) if !items.is_empty() => render_array(items),
        serde_json::Value::Object(map) => {
            // Single object: key,value rows
            let mut out = String::from("key,value\n");
            for (k, v) in map {
                let val = match v {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Null => String::new(),
                    other => other.to_string(),
                };
                out.push_str(&format!("{},{}\n", escape_csv(k), escape_csv(&val)));
            }
            out
        }
        other => other.to_string(),
    }
}

fn render_array(items: &[serde_json::Value]) -> String {
    let headers: Vec<String> = match items.first() {
        Some(serde_json::Value::Object(map)) => map.keys().cloned().collect(),
        _ => {
            return items
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join("\n");
        }
    };

    let mut out = headers
        .iter()
        .map(|h| escape_csv(h))
        .collect::<Vec<_>>()
        .join(",");
    out.push('\n');

    for item in items {
        let row: Vec<String> = headers
            .iter()
            .map(|key| match item.get(key) {
                Some(serde_json::Value::String(s)) => escape_csv(s),
                Some(serde_json::Value::Null) => String::new(),
                Some(v) => escape_csv(&v.to_string()),
                None => String::new(),
            })
            .collect();
        out.push_str(&row.join(","));
        out.push('\n');
    }

    out
}

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
