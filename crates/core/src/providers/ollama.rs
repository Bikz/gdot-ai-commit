use std::time::Duration;

use serde_json::Value;

use crate::error::{CoreError, CoreResult};
use crate::providers::{Provider, ProviderRequest};
use crate::retry::sleep_with_jitter;

pub struct OllamaProvider {
    client: reqwest::Client,
    endpoint: String,
    model: String,
}

impl OllamaProvider {
    pub fn new(model: String, endpoint: String, timeout_secs: u64) -> CoreResult<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .connect_timeout(Duration::from_secs(timeout_secs))
            .build()?;

        Ok(Self {
            client,
            endpoint,
            model,
        })
    }

    async fn send_with_retries(&self, body: Value) -> CoreResult<Value> {
        let mut attempt = 0usize;
        let max_attempts = 3usize;
        let mut last_error = None;

        while attempt < max_attempts {
            let response = self.client.post(&self.endpoint).json(&body).send().await;

            match response {
                Ok(resp) => {
                    let status = resp.status();
                    let json: Value = resp.json().await?;
                    if status.is_success() {
                        return Ok(json);
                    }

                    let err = if let Some(error) = json.get("error").and_then(|v| v.as_str()) {
                        CoreError::Provider(format!("ollama error: {error}"))
                    } else {
                        CoreError::Provider(format!("ollama error: {status}"))
                    };

                    if status.is_server_error() || status == reqwest::StatusCode::REQUEST_TIMEOUT {
                        last_error = Some(err);
                        sleep_with_jitter(attempt, 200, 2000).await;
                        attempt += 1;
                        continue;
                    }

                    return Err(err);
                }
                Err(err) => {
                    last_error = Some(CoreError::Provider(format!("ollama request failed: {err}")));
                    sleep_with_jitter(attempt, 200, 2000).await;
                    attempt += 1;
                }
            }
        }

        Err(last_error.unwrap_or_else(|| CoreError::Provider("ollama request failed".to_string())))
    }
}

#[async_trait::async_trait]
impl Provider for OllamaProvider {
    async fn complete(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        request: ProviderRequest,
    ) -> CoreResult<String> {
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

        let json = self.send_with_retries(body).await?;
        json.get("message")
            .and_then(|msg| msg.get("content"))
            .and_then(|content| content.as_str())
            .map(|text| text.trim().to_string())
            .filter(|text| !text.is_empty())
            .ok_or_else(|| CoreError::Provider("ollama response missing content".to_string()))
    }
}
