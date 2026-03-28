use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::debug;

use super::feature_extractor::FeatureSnapshot;

const GEMINI_API_BASE: &str = "https://generativelanguage.googleapis.com/v1beta/models";
const REQUEST_TIMEOUT_SECS: u64 = 30;

#[derive(Debug, Error)]
pub enum InsightError {
    #[error("Gemini API request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Invalid API key (401/403)")]
    InvalidKey,

    #[error("Rate limited (429)")]
    RateLimited,

    #[error("Failed to parse Gemini response: {0}")]
    ParseFailed(String),

    #[error("Gemini API returned error: {status} {body}")]
    ApiError { status: u16, body: String },
}

/// Structured response from Gemini, matching the responseSchema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightResponse {
    pub headline: String,
    pub bullets: Vec<String>,
    #[serde(default)]
    pub affected_surfaces: Vec<String>,
    #[serde(default)]
    pub reasoning_basis: Vec<String>,
    pub confidence: f64,
    pub severity: String,
}

pub struct GeminiClient {
    client: Client,
    api_key: String,
    model: String,
}

impl GeminiClient {
    pub fn new(client: Client, api_key: String, model: &str) -> Self {
        Self {
            client,
            api_key,
            model: model.to_string(),
        }
    }

    pub async fn generate_insight(
        &self,
        features: &FeatureSnapshot,
    ) -> Result<InsightResponse, InsightError> {
        let url = format!(
            "{}/{}:generateContent?key={}",
            GEMINI_API_BASE, self.model, self.api_key
        );

        let features_json = serde_json::to_string(features)
            .map_err(|e| InsightError::ParseFailed(e.to_string()))?;

        let request_body = build_request_body(&features_json);

        debug!(model = %self.model, "Sending request to Gemini API");

        let response = self
            .client
            .post(&url)
            .json(&request_body)
            .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .send()
            .await?;

        let status = response.status().as_u16();
        match status {
            200 => {}
            401 | 403 => return Err(InsightError::InvalidKey),
            429 => return Err(InsightError::RateLimited),
            _ => {
                let body = response.text().await.unwrap_or_default();
                return Err(InsightError::ApiError { status, body });
            }
        }

        let gemini_response: serde_json::Value = response.json().await?;
        parse_gemini_response(&gemini_response)
    }
}

fn build_request_body(features_json: &str) -> serde_json::Value {
    serde_json::json!({
        "contents": [{
            "parts": [{
                "text": format!(
                    "You are a VRChat server status analyst. Analyze the following server metrics snapshot and provide a status insight in Korean.\n\n{features_json}"
                )
            }]
        }],
        "generationConfig": {
            "responseMimeType": "application/json",
            "responseSchema": {
                "type": "object",
                "properties": {
                    "headline": {
                        "type": "string",
                        "description": "한줄 요약 (한국어)"
                    },
                    "bullets": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "분석 포인트 3-4개 (한국어)"
                    },
                    "affected_surfaces": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "영향받는 서비스 (steam_auth, api, oculus_auth 등)"
                    },
                    "reasoning_basis": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "분석 근거"
                    },
                    "confidence": {
                        "type": "number",
                        "description": "분석 신뢰도 0.0-1.0"
                    },
                    "severity": {
                        "type": "string",
                        "enum": ["stable", "warning", "critical"],
                        "description": "심각도"
                    }
                },
                "required": ["headline", "bullets", "confidence", "severity"]
            }
        }
    })
}

fn parse_gemini_response(response: &serde_json::Value) -> Result<InsightResponse, InsightError> {
    // Gemini returns: { candidates: [{ content: { parts: [{ text: "..." }] } }] }
    let text = response["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .ok_or_else(|| {
            InsightError::ParseFailed(format!(
                "Missing text in Gemini response: {}",
                serde_json::to_string_pretty(response).unwrap_or_default()
            ))
        })?;

    serde_json::from_str(text)
        .map_err(|e| InsightError::ParseFailed(format!("JSON parse error: {e}, text: {text}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_request_body_has_response_schema() {
        let body = build_request_body("{}");
        assert_eq!(
            body["generationConfig"]["responseMimeType"],
            "application/json"
        );
        assert!(body["generationConfig"]["responseSchema"]["properties"]["headline"].is_object());
        assert!(
            body["generationConfig"]["responseSchema"]["properties"]["severity"]["enum"].is_array()
        );
    }

    #[test]
    fn test_parse_gemini_response_valid() {
        let response = serde_json::json!({
            "candidates": [{
                "content": {
                    "parts": [{
                        "text": r#"{"headline":"VRChat 서버 안정","bullets":["정상 운영 중"],"confidence":0.9,"severity":"stable"}"#
                    }]
                }
            }]
        });
        let result = parse_gemini_response(&response).unwrap();
        assert_eq!(result.headline, "VRChat 서버 안정");
        assert_eq!(result.severity, "stable");
        assert!((result.confidence - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_gemini_response_missing_text() {
        let response = serde_json::json!({ "candidates": [] });
        assert!(parse_gemini_response(&response).is_err());
    }
}
