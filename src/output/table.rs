use tabled::{settings::Style, Table};

/// Render data as a human-readable ASCII table.
pub fn render(data: &serde_json::Value) -> String {
    match data {
        serde_json::Value::Array(items) if !items.is_empty() => render_array(items),
        serde_json::Value::Object(map) => render_object(map),
        other => serde_json::to_string_pretty(other).unwrap_or_default(),
    }
}

fn render_array(items: &[serde_json::Value]) -> String {
    // Collect all keys from the first object to use as columns
    let headers: Vec<String> = match items.first() {
        Some(serde_json::Value::Object(map)) => map.keys().cloned().collect(),
        _ => {
            return items
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        }
    };

    let mut rows: Vec<Vec<String>> = Vec::with_capacity(items.len() + 1);
    rows.push(headers.clone());

    for item in items {
        let row: Vec<String> = headers
            .iter()
            .map(|key| match item.get(key) {
                Some(serde_json::Value::String(s)) => s.clone(),
                Some(serde_json::Value::Null) => String::new(),
                Some(v) => v.to_string(),
                None => String::new(),
            })
            .collect();
        rows.push(row);
    }

    let table = Table::from_iter(rows).with(Style::rounded()).to_string();
    table
}

fn render_object(map: &serde_json::Map<String, serde_json::Value>) -> String {
    let rows: Vec<Vec<String>> = map
        .iter()
        .map(|(k, v)| {
            let val = match v {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Null => String::new(),
                other => other.to_string(),
            };
            vec![k.clone(), val]
        })
        .collect();

    let mut all_rows = vec![vec!["Key".to_string(), "Value".to_string()]];
    all_rows.extend(rows);

    Table::from_iter(all_rows)
        .with(Style::rounded())
        .to_string()
}
