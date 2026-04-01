use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::debug;

use super::feature_extractor::FeatureSnapshot;

const GEMINI_API_BASE: &str = "https://generativelanguage.googleapis.com/v1beta/models";
const ANALYSIS_TIMEOUT_SECS: u64 = 30;
const TRANSLATION_TIMEOUT_SECS: u64 = 15;

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

/// Translation response for incident/maintenance text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationResult {
    pub name: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub updates: Vec<TranslatedUpdate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslatedUpdate {
    pub update_id: String,
    pub translated_body: String,
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
    #[serde(default)]
    pub confidence: f64,
    #[serde(default)]
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
        let features_json = serde_json::to_string(features)
            .map_err(|e| InsightError::ParseFailed(e.to_string()))?;

        let request_body = build_analysis_request_body(&features_json);

        debug!(model = %self.model, "Sending analysis request to Gemini API");

        self.call_gemini(&request_body, ANALYSIS_TIMEOUT_SECS).await
    }

    pub async fn translate_insight(
        &self,
        english_insight: &InsightResponse,
        target_locale: &str,
    ) -> Result<InsightResponse, InsightError> {
        let request_body = build_translation_request_body(english_insight, target_locale)?;

        debug!(model = %self.model, locale = target_locale, "Sending translation request to Gemini API");

        let mut translated = self
            .call_gemini(&request_body, TRANSLATION_TIMEOUT_SECS)
            .await?;

        // Copy non-translatable fields from English source
        translated.affected_surfaces = english_insight.affected_surfaces.clone();
        translated.confidence = english_insight.confidence;
        translated.severity = english_insight.severity.clone();

        Ok(translated)
    }

    /// Translate incident/maintenance content to the target locale.
    pub async fn translate_content(
        &self,
        name: &str,
        body: &str,
        updates: &[(String, String)], // (update_id, body)
        target_locale: &str,
    ) -> Result<TranslationResult, InsightError> {
        let request_body = build_content_translation_body(name, body, updates, target_locale);

        debug!(model = %self.model, locale = target_locale, "Sending content translation request to Gemini API");

        let raw = self
            .call_gemini_raw(&request_body, TRANSLATION_TIMEOUT_SECS)
            .await?;

        serde_json::from_str(&raw)
            .map_err(|e| InsightError::ParseFailed(format!("JSON parse error: {e}, text: {raw}")))
    }

    /// Low-level Gemini call that returns the raw text from the response.
    async fn call_gemini_raw(
        &self,
        request_body: &serde_json::Value,
        timeout_secs: u64,
    ) -> Result<String, InsightError> {
        let url = format!(
            "{}/{}:generateContent?key={}",
            GEMINI_API_BASE, self.model, self.api_key
        );

        let response = self
            .client
            .post(&url)
            .json(request_body)
            .timeout(std::time::Duration::from_secs(timeout_secs))
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
        gemini_response["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| {
                InsightError::ParseFailed(format!(
                    "Missing text in Gemini response: {}",
                    serde_json::to_string_pretty(&gemini_response).unwrap_or_default()
                ))
            })
    }

    async fn call_gemini(
        &self,
        request_body: &serde_json::Value,
        timeout_secs: u64,
    ) -> Result<InsightResponse, InsightError> {
        let url = format!(
            "{}/{}:generateContent?key={}",
            GEMINI_API_BASE, self.model, self.api_key
        );

        let response = self
            .client
            .post(&url)
            .json(request_body)
            .timeout(std::time::Duration::from_secs(timeout_secs))
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

fn build_analysis_request_body(features_json: &str) -> serde_json::Value {
    serde_json::json!({
        "contents": [{
            "parts": [{
                "text": format!(
                    "You are a VRChat server status analyst. Analyze the following server metrics snapshot and provide a status insight in English.\n\n{features_json}"
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
                        "description": "One-line summary"
                    },
                    "bullets": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "3-4 analysis points"
                    },
                    "affected_surfaces": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Affected services (steam_auth, api, oculus_auth, etc.)"
                    },
                    "reasoning_basis": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Reasoning basis for the analysis"
                    },
                    "confidence": {
                        "type": "number",
                        "description": "Analysis confidence 0.0-1.0"
                    },
                    "severity": {
                        "type": "string",
                        "enum": ["stable", "warning", "critical"],
                        "description": "Severity level"
                    }
                },
                "required": ["headline", "bullets", "confidence", "severity"]
            }
        }
    })
}

fn build_translation_request_body(
    english_insight: &InsightResponse,
    target_locale: &str,
) -> Result<serde_json::Value, InsightError> {
    let (locale_name, desc_headline, desc_bullets, desc_reasoning) = match target_locale {
        "ko" => (
            "Korean",
            "한줄 요약 (한국어)",
            "분석 포인트 3-4개 (한국어)",
            "분석 근거 (한국어)",
        ),
        "jp" => (
            "Japanese",
            "一行要約（日本語）",
            "分析ポイント3-4個（日本語）",
            "分析根拠（日本語）",
        ),
        _ => (
            target_locale,
            "One-line summary (translated)",
            "3-4 analysis points (translated)",
            "Reasoning basis (translated)",
        ),
    };

    let source_json = serde_json::json!({
        "headline": english_insight.headline,
        "bullets": english_insight.bullets,
        "reasoning_basis": english_insight.reasoning_basis,
    });
    let source_text = serde_json::to_string(&source_json)
        .map_err(|e| InsightError::ParseFailed(e.to_string()))?;

    Ok(serde_json::json!({
        "contents": [{
            "parts": [{
                "text": format!(
                    "Translate the following VRChat server status analysis to {locale_name}. \
                     Maintain technical terms (e.g. Steam, API, Oculus). \
                     Return only the translated fields in the same JSON structure.\n\n{source_text}"
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
                        "description": desc_headline
                    },
                    "bullets": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": desc_bullets
                    },
                    "reasoning_basis": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": desc_reasoning
                    }
                },
                "required": ["headline", "bullets"]
            }
        }
    }))
}

fn build_content_translation_body(
    name: &str,
    body: &str,
    updates: &[(String, String)],
    target_locale: &str,
) -> serde_json::Value {
    let locale_name = match target_locale {
        "ko" => "Korean",
        "ja" | "jp" => "Japanese",
        _ => target_locale,
    };

    let updates_json: Vec<serde_json::Value> = updates
        .iter()
        .map(|(id, text)| {
            serde_json::json!({
                "update_id": id,
                "body": text
            })
        })
        .collect();

    let source = serde_json::json!({
        "name": name,
        "body": body,
        "updates": updates_json
    });

    serde_json::json!({
        "contents": [{
            "parts": [{
                "text": format!(
                    "Translate the following VRChat server status text to {locale_name}. \
                     Maintain technical terms (e.g. Steam, API, Oculus). \
                     Use a neutral, informative tone suitable for a status monitoring dashboard. \
                     Return the translated fields in the specified JSON structure.\n\n{}",
                    serde_json::to_string(&source).unwrap_or_default()
                )
            }]
        }],
        "generationConfig": {
            "responseMimeType": "application/json",
            "responseSchema": {
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Translated title"
                    },
                    "body": {
                        "type": "string",
                        "description": "Translated body/description"
                    },
                    "updates": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "update_id": { "type": "string" },
                                "translated_body": { "type": "string" }
                            },
                            "required": ["update_id", "translated_body"]
                        },
                        "description": "Translated status updates"
                    }
                },
                "required": ["name"]
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
    fn test_build_analysis_request_body_has_response_schema() {
        let body = build_analysis_request_body("{}");
        assert_eq!(
            body["generationConfig"]["responseMimeType"],
            "application/json"
        );
        assert!(body["generationConfig"]["responseSchema"]["properties"]["headline"].is_object());
        assert!(
            body["generationConfig"]["responseSchema"]["properties"]["severity"]["enum"].is_array()
        );
        // Verify English prompt
        let text = body["contents"][0]["parts"][0]["text"].as_str().unwrap();
        assert!(text.contains("in English"));
    }

    #[test]
    fn test_build_translation_request_body_korean() {
        let insight = InsightResponse {
            headline: "VRChat servers stable".to_string(),
            bullets: vec!["All systems normal".to_string()],
            affected_surfaces: vec!["api".to_string()],
            reasoning_basis: vec!["Low error rate".to_string()],
            confidence: 0.9,
            severity: "stable".to_string(),
        };
        let body = build_translation_request_body(&insight, "ko").unwrap();
        assert_eq!(
            body["generationConfig"]["responseMimeType"],
            "application/json"
        );
        let text = body["contents"][0]["parts"][0]["text"].as_str().unwrap();
        assert!(text.contains("Korean"));
        assert!(text.contains("VRChat servers stable"));
        // Translation schema should not include severity/confidence
        assert!(body["generationConfig"]["responseSchema"]["properties"]["severity"].is_null());
    }

    #[test]
    fn test_build_translation_request_body_japanese() {
        let insight = InsightResponse {
            headline: "VRChat servers stable".to_string(),
            bullets: vec!["All systems normal".to_string()],
            affected_surfaces: vec!["api".to_string()],
            reasoning_basis: vec!["Low error rate".to_string()],
            confidence: 0.9,
            severity: "stable".to_string(),
        };
        let body = build_translation_request_body(&insight, "jp").unwrap();
        let text = body["contents"][0]["parts"][0]["text"].as_str().unwrap();
        assert!(text.contains("Japanese"));
        let desc =
            body["generationConfig"]["responseSchema"]["properties"]["headline"]["description"]
                .as_str()
                .unwrap();
        assert!(desc.contains("日本語"));
    }

    #[test]
    fn test_parse_gemini_response_valid() {
        let response = serde_json::json!({
            "candidates": [{
                "content": {
                    "parts": [{
                        "text": r#"{"headline":"VRChat servers stable","bullets":["All systems normal"],"confidence":0.9,"severity":"stable"}"#
                    }]
                }
            }]
        });
        let result = parse_gemini_response(&response).unwrap();
        assert_eq!(result.headline, "VRChat servers stable");
        assert_eq!(result.severity, "stable");
        assert!((result.confidence - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_gemini_response_missing_text() {
        let response = serde_json::json!({ "candidates": [] });
        assert!(parse_gemini_response(&response).is_err());
    }

    #[test]
    fn test_parse_translation_response_without_confidence_severity() {
        // Translation schema only requires headline + bullets.
        // InsightResponse must parse with serde(default) for missing fields.
        let response = serde_json::json!({
            "candidates": [{
                "content": {
                    "parts": [{
                        "text": r#"{"headline":"VRChat 서버 안정","bullets":["정상 운영 중"],"reasoning_basis":["낮은 오류율"]}"#
                    }]
                }
            }]
        });
        let result = parse_gemini_response(&response).unwrap();
        assert_eq!(result.headline, "VRChat 서버 안정");
        assert_eq!(result.bullets, vec!["정상 운영 중"]);
        // These should default to 0.0 and "" since translation doesn't return them
        assert!((result.confidence - 0.0).abs() < f64::EPSILON);
        assert_eq!(result.severity, "");
    }

    #[test]
    fn test_translation_body_only_includes_translatable_fields() {
        let insight = InsightResponse {
            headline: "VRChat servers stable".to_string(),
            bullets: vec!["All systems normal".to_string()],
            affected_surfaces: vec!["api".to_string(), "steam_auth".to_string()],
            reasoning_basis: vec!["Low error rate".to_string()],
            confidence: 0.95,
            severity: "warning".to_string(),
        };
        let body = build_translation_request_body(&insight, "ko").unwrap();
        let text = body["contents"][0]["parts"][0]["text"].as_str().unwrap();
        // Source text should contain translatable fields
        assert!(text.contains("VRChat servers stable"));
        assert!(text.contains("All systems normal"));
        assert!(text.contains("Low error rate"));
        // Source text should NOT contain non-translatable fields
        assert!(!text.contains("0.95"));
        assert!(!text.contains("\"warning\""));
        assert!(!text.contains("steam_auth"));
    }
}
