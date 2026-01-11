use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use rand::{thread_rng, Rng};
use reqwest::StatusCode;
use serde_json::Value;

use crate::config::{openai_api_key, OpenAiMode};
use crate::providers::{openai_mode_for, Provider, ProviderRequest};

pub struct OpenAiProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    model: String,
    mode: OpenAiMode,
}

impl OpenAiProvider {
    pub fn new(
        model: String,
        base_url: String,
        mode: OpenAiMode,
        timeout_secs: u64,
    ) -> Result<Self> {
        let api_key = openai_api_key().ok_or_else(|| anyhow!("OPENAI_API_KEY is not set"))?;
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .context("failed to build http client")?;

        Ok(Self {
            client,
            api_key,
            base_url,
            model,
            mode,
        })
    }

    fn responses_url(&self) -> String {
        format!("{}/responses", self.base_url.trim_end_matches('/'))
    }

    fn chat_url(&self) -> String {
        format!("{}/chat/completions", self.base_url.trim_end_matches('/'))
    }

    async fn send_with_retries(&self, request: reqwest::RequestBuilder) -> Result<Value> {
        let mut attempt = 0;
        let max_attempts = 3;
        let mut last_error = None;

        while attempt < max_attempts {
            let response = request
                .try_clone()
                .ok_or_else(|| anyhow!("failed to clone request"))?
                .send()
                .await;

            match response {
                Ok(resp) => {
                    if resp.status().is_success() {
                        return resp.json::<Value>().await.context("invalid json response");
                    }

                    let status = resp.status();
                    let body = resp.text().await.unwrap_or_default();
                    if should_retry(status) {
                        last_error = Some(anyhow!("openai error {status}: {body}"));
                        sleep_jitter(attempt).await;
                        attempt += 1;
                        continue;
                    }

                    return Err(anyhow!("openai error {status}: {body}"));
                }
                Err(err) => {
                    last_error = Some(anyhow!("openai request failed: {err}"));
                    sleep_jitter(attempt).await;
                    attempt += 1;
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow!("openai request failed")))
    }

    async fn complete_responses(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        request: ProviderRequest,
    ) -> Result<String> {
        let body = serde_json::json!({
            "model": self.model,
            "input": [
                {
                    "role": "system",
                    "content": [{ "type": "text", "text": system_prompt }]
                },
                {
                    "role": "user",
                    "content": [{ "type": "text", "text": user_prompt }]
                }
            ],
            "max_completion_tokens": request.max_output_tokens,
            "temperature": request.temperature
        });

        let request = self
            .client
            .post(self.responses_url())
            .bearer_auth(&self.api_key)
            .json(&body);

        let json = self.send_with_retries(request).await?;
        parse_responses_output(&json)
    }

    async fn complete_chat(
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
            "max_tokens": request.max_output_tokens,
            "temperature": request.temperature
        });

        let request = self
            .client
            .post(self.chat_url())
            .bearer_auth(&self.api_key)
            .json(&body);

        let json = self.send_with_retries(request).await?;
        parse_chat_output(&json)
    }
}

#[async_trait::async_trait]
impl Provider for OpenAiProvider {
    async fn complete(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        request: ProviderRequest,
    ) -> Result<String> {
        let mode = openai_mode_for(&self.model, self.mode);
        let request = ProviderRequest {
            max_output_tokens: request.max_output_tokens,
            temperature: request.temperature,
        };

        match mode {
            OpenAiMode::Responses => {
                self.complete_responses(system_prompt, user_prompt, request)
                    .await
            }
            OpenAiMode::Chat => {
                self.complete_chat(system_prompt, user_prompt, request)
                    .await
            }
            OpenAiMode::Auto => unreachable!(),
        }
    }
}

fn parse_responses_output(json: &Value) -> Result<String> {
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

    Err(anyhow!("openai response missing output text"))
}

fn parse_chat_output(json: &Value) -> Result<String> {
    json.get("choices")
        .and_then(|choices| choices.get(0))
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(|content| content.as_str())
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
        .ok_or_else(|| anyhow!("openai response missing content"))
}

fn should_retry(status: StatusCode) -> bool {
    matches!(status, StatusCode::TOO_MANY_REQUESTS)
        || status.is_server_error()
        || status == StatusCode::REQUEST_TIMEOUT
}

async fn sleep_jitter(attempt: usize) {
    let jitter: u64 = thread_rng().gen_range(100..400);
    let backoff = 200_u64.saturating_mul(2_u64.pow(attempt as u32));
    let delay = backoff + jitter;
    tokio::time::sleep(Duration::from_millis(delay)).await;
}
