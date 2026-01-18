use serde::Deserialize;

#[derive(Deserialize)]
struct RequestArgs {
    method: String,
    path: String,
    content_type: Option<String>,
    body: serde_json::Value,
}
