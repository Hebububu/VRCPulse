use reqwest::Client;
use serde::de::DeserializeOwned;
use thiserror::Error;

/// VRChat Status API base URL
pub const VRCHAT_STATUS_API_BASE: &str = "https://status.vrchat.com/api/v2";

/// CloudFront Metrics API base URL
pub const CLOUDFRONT_METRICS_BASE: &str = "https://d31qqo63tn8lj0.cloudfront.net";

#[derive(Debug, Error)]
pub enum CollectorError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
}

pub type Result<T> = std::result::Result<T, CollectorError>;

/// Fetch JSON from a URL and deserialize to type T
pub async fn fetch_json<T: DeserializeOwned>(client: &Client, url: &str) -> Result<T> {
    let response = client.get(url).send().await?.error_for_status()?;
    let data = response.json::<T>().await?;
    Ok(data)
}

/// Build full URL for VRChat Status API endpoint
pub fn status_api_url(endpoint: &str) -> String {
    format!("{}{}", VRCHAT_STATUS_API_BASE, endpoint)
}

/// Build full URL for CloudFront Metrics API endpoint
pub fn metrics_api_url(endpoint: &str) -> String {
    format!("{}{}", CLOUDFRONT_METRICS_BASE, endpoint)
}
