use serde_json::Value;

use crate::error::{CoreError, CoreResult};

pub(super) fn parse_responses_output(json: &Value) -> CoreResult<String> {
    if let Some(text) = json.get("output_text").and_then(|v| v.as_str()) {
        if !text.trim().is_empty() {
            return Ok(text.trim().to_string());
        }
    }

    if let Some(output) = json.get("output").and_then(|v| v.as_array()) {
        let mut collected = String::new();
        for item in output {
            if let Some(content) = item.get("content").and_then(|v| v.as_array()) {
                for part in content {
                    if let Some(text) = part.get("text").and_then(|v| v.as_str()) {
                        collected.push_str(text);
                    }
                }
            }
        }

        let trimmed = collected.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
    }

    tracing::debug!(response = ?json, "openai response missing output text");
    Err(CoreError::Provider(
        "openai response missing output text".to_string(),
    ))
}

pub(super) fn parse_chat_output(json: &Value) -> CoreResult<String> {
    let content = json
        .get("choices")
        .and_then(|choices| choices.get(0))
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(|content| content.as_str())
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty());

    if content.is_none() {
        tracing::debug!(response = ?json, "openai response missing content");
    }

    content.ok_or_else(|| CoreError::Provider("openai response missing content".to_string()))
}
