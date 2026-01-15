use std::time::Duration;

use reqwest::StatusCode;
use serde_json::Value;

use crate::config::{openai_api_key_env, OpenAiMode};
use crate::error::{CoreError, CoreResult};
use crate::providers::{openai_mode_for, Provider, ProviderRequest};
use crate::retry::sleep_with_jitter;

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
        api_key: Option<String>,
    ) -> CoreResult<Self> {
        let api_key = api_key.or_else(openai_api_key_env).ok_or_else(|| {
            CoreError::Provider(
                "OpenAI API key is missing (run setup or set OPENAI_API_KEY)".to_string(),
            )
        })?;
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .connect_timeout(Duration::from_secs(timeout_secs))
            .build()?;

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

    async fn send_with_retries(&self, request: reqwest::RequestBuilder) -> CoreResult<Value> {
        let mut attempt = 0usize;
        let max_attempts = 3usize;
        let mut last_error = None;

        while attempt < max_attempts {
            let response = request
                .try_clone()
                .ok_or_else(|| CoreError::Provider("failed to clone request".to_string()))?
                .send()
                .await;

            match response {
                Ok(resp) => {
                    if resp.status().is_success() {
                        return resp.json::<Value>().await.map_err(CoreError::from);
                    }

                    let status = resp.status();
                    let body = resp.text().await.unwrap_or_default();
                    let err = CoreError::Provider(format!("openai error {status}: {body}"));
                    if should_retry(status) {
                        last_error = Some(err);
                        sleep_with_jitter(attempt, 200, 2000).await;
                        attempt += 1;
                        continue;
                    }

                    return Err(err);
                }
                Err(err) => {
                    last_error = Some(CoreError::Provider(format!("openai request failed: {err}")));
                    sleep_with_jitter(attempt, 200, 2000).await;
                    attempt += 1;
                }
            }
        }

        Err(last_error.unwrap_or_else(|| CoreError::Provider("openai request failed".to_string())))
    }

    async fn complete_responses(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        request: ProviderRequest,
    ) -> CoreResult<String> {
        let base = self.responses_base_payload(system_prompt, user_prompt, request.temperature);

        match self
            .complete_responses_with_param(&base, "max_output_tokens", request.max_output_tokens)
            .await
        {
            Ok(message) => Ok(message),
            Err(err) => {
                if is_unsupported_param(&err, "max_output_tokens") {
                    return self
                        .complete_responses_with_param(
                            &base,
                            "max_completion_tokens",
                            request.max_output_tokens,
                        )
                        .await;
                }
                Err(err)
            }
        }
    }

    async fn complete_chat(
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

    fn responses_base_payload(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        temperature: f32,
    ) -> Value {
        serde_json::json!({
            "model": self.model,
            "input": [
                {
                    "role": "system",
                    "content": [{ "type": "input_text", "text": system_prompt }]
                },
                {
                    "role": "user",
                    "content": [{ "type": "input_text", "text": user_prompt }]
                }
            ],
            "temperature": temperature
        })
    }

    async fn complete_responses_with_param(
        &self,
        base: &Value,
        param: &str,
        max_tokens: u32,
    ) -> CoreResult<String> {
        let mut body = base.clone();
        if let Some(obj) = body.as_object_mut() {
            obj.insert(param.to_string(), serde_json::json!(max_tokens));
        }

        let request = self
            .client
            .post(self.responses_url())
            .bearer_auth(&self.api_key)
            .json(&body);

        let json = self.send_with_retries(request).await?;
        parse_responses_output(&json)
    }
}

#[async_trait::async_trait]
impl Provider for OpenAiProvider {
    async fn complete(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        request: ProviderRequest,
    ) -> CoreResult<String> {
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

fn parse_responses_output(json: &Value) -> CoreResult<String> {
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

    Err(CoreError::Provider(
        "openai response missing output text".to_string(),
    ))
}

fn parse_chat_output(json: &Value) -> CoreResult<String> {
    json.get("choices")
        .and_then(|choices| choices.get(0))
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(|content| content.as_str())
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
        .ok_or_else(|| CoreError::Provider("openai response missing content".to_string()))
}

fn should_retry(status: StatusCode) -> bool {
    matches!(status, StatusCode::TOO_MANY_REQUESTS)
        || status.is_server_error()
        || status == StatusCode::REQUEST_TIMEOUT
}

fn is_unsupported_param(err: &CoreError, param: &str) -> bool {
    let message = err.to_string();
    message.contains("unsupported_parameter") && message.contains(param)
}
