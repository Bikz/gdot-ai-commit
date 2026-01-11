use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use serde_json::Value;

use crate::providers::{Provider, ProviderRequest};

pub struct OllamaProvider {
    client: reqwest::Client,
    endpoint: String,
    model: String,
}

impl OllamaProvider {
    pub fn new(model: String, endpoint: String, timeout_secs: u64) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .context("failed to build http client")?;

        Ok(Self {
            client,
            endpoint,
            model,
        })
    }
}

#[async_trait::async_trait]
impl Provider for OllamaProvider {
    async fn complete(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        request: ProviderRequest,
    ) -> Result<String> {
        let body = serde_json::json!({
            "model": self.model,
            "messages": [
                { "role": "system", "content": system_prompt },
                { "role": "user", "content": user_prompt }
            ],
            "stream": false,
            "options": {
                "temperature": request.temperature,
                "num_predict": request.max_output_tokens
            }
        });

        let response = self
            .client
            .post(&self.endpoint)
            .json(&body)
            .send()
            .await
            .context("failed to reach ollama")?;

        let status = response.status();
        let json: Value = response.json().await.context("invalid ollama response")?;
        if !status.is_success() {
            if let Some(error) = json.get("error").and_then(|v| v.as_str()) {
                return Err(anyhow!("ollama error: {error}"));
            }
            return Err(anyhow!("ollama error: {status}"));
        }

        json.get("message")
            .and_then(|msg| msg.get("content"))
            .and_then(|content| content.as_str())
            .map(|text| text.trim().to_string())
            .filter(|text| !text.is_empty())
            .ok_or_else(|| anyhow!("ollama response missing content"))
    }
}
