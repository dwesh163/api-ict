use reqwest;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
}

pub async fn get_token() -> Result<String, Box<dyn std::error::Error>> {
    let body = reqwest::get("https://www.modulbaukasten.ch/assets/auth.php")
        .await?
        .text()
        .await?;

    if let Some(inner_string) = serde_json::from_str::<Value>(&body)?.as_str() {
        return Ok(serde_json::from_str::<TokenResponse>(inner_string)?.access_token);
    }

    Err("Failed to parse token response".into())
}
