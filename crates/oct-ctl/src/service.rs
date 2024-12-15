use std::collections::HashMap;
use reqwest::{self, Response};

pub async fn run_container(container_name: String, port: String) -> Result<Response, reqwest::Error> {
    let client = reqwest::Client::new();

    let mut map = HashMap::new();
    map.insert("image_uri", container_name.as_str());
    map.insert("port", port.as_str());

    let response = client
        .post("http://localhost:31888/run-container")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .body(serde_json::to_string(&map).unwrap())
        .send()
        .await;

    return response
}
