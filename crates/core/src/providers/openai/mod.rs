use std::time::Duration;

use serde_json::Value;
use tracing::instrument;

use crate::config::{openai_api_key_env, OpenAiMode};
use crate::error::{CoreError, CoreResult};
use crate::providers::{openai_mode_for, Provider, ProviderRequest};
use crate::retry::sleep_with_jitter;

mod parse;
mod payloads;
mod retry;

#[cfg(test)]
mod tests;

pub struct OpenAiProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    model: String,
    mode: OpenAiMode,
}

impl OpenAiProvider {
    /// Create a new `OpenAI` provider client.
    ///
    /// # Errors
    /// Returns an error if the API key is missing or the HTTP client fails to build.
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
                    if retry::should_retry(status) {
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
        let temperature = if self.is_gpt5() {
            None
        } else {
            Some(request.temperature)
        };
        let base = payloads::responses_base_payload(
            &self.model,
            system_prompt,
            user_prompt,
            temperature,
            self.is_gpt5(),
        );

        match self
            .complete_responses_with_fallbacks(&base, request.max_output_tokens)
            .await
        {
            Ok(message) => Ok(message),
            Err(err) => {
                if retry::is_unsupported_param(&err, "temperature") {
                    let base = payloads::responses_base_payload(
                        &self.model,
                        system_prompt,
                        user_prompt,
                        None,
                        self.is_gpt5(),
                    );
                    return self
                        .complete_responses_with_fallbacks(&base, request.max_output_tokens)
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
        let temperature = if self.is_gpt5() {
            None
        } else {
            Some(request.temperature)
        };
        let body = payloads::chat_payload(
            &self.model,
            system_prompt,
            user_prompt,
            request.max_output_tokens,
            temperature,
        );

        let http_request = self
            .client
            .post(self.chat_url())
            .bearer_auth(&self.api_key)
            .json(&body);

        let json = match self.send_with_retries(http_request).await {
            Ok(json) => json,
            Err(err) => {
                if retry::is_unsupported_param(&err, "temperature") {
                    let body = payloads::chat_payload(
                        &self.model,
                        system_prompt,
                        user_prompt,
                        request.max_output_tokens,
                        None,
                    );
                    let http_request = self
                        .client
                        .post(self.chat_url())
                        .bearer_auth(&self.api_key)
                        .json(&body);
                    let json = self.send_with_retries(http_request).await?;
                    return parse::parse_chat_output(&json);
                }
                return Err(err);
            }
        };

        parse::parse_chat_output(&json)
    }

    async fn complete_responses_with_fallbacks(
        &self,
        base: &Value,
        max_tokens: u32,
    ) -> CoreResult<String> {
        match self
            .complete_responses_with_param(base, "max_output_tokens", max_tokens)
            .await
        {
            Ok(message) => Ok(message),
            Err(err) => {
                if retry::is_unsupported_param(&err, "max_output_tokens") {
                    return self
                        .complete_responses_with_param(base, "max_completion_tokens", max_tokens)
                        .await;
                }
                Err(err)
            }
        }
    }

    pub(super) fn is_gpt5(&self) -> bool {
        payloads::is_gpt5_model(&self.model)
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
        parse::parse_responses_output(&json)
    }
}

#[async_trait::async_trait]
impl Provider for OpenAiProvider {
    #[instrument(level = "debug", skip(self, system_prompt, user_prompt))]
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
