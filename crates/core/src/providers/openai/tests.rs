use super::payloads;
use super::retry::is_unsupported_param;
use super::OpenAiProvider;
use crate::config::OpenAiMode;
use crate::error::CoreError;

#[test]
fn responses_payload_uses_input_text_parts() {
    let payload = payloads::responses_base_payload(
        "gpt-5-nano-2025-08-07",
        "system",
        "user",
        Some(0.2),
        true,
    );
    let input = payload
        .get("input")
        .and_then(|value| value.as_array())
        .expect("input array");

    let system = input[0]
        .get("content")
        .and_then(|value| value.as_array())
        .expect("system content");
    assert_eq!(
        system[0].get("type").and_then(|value| value.as_str()),
        Some("input_text")
    );
    assert_eq!(
        system[0].get("text").and_then(|value| value.as_str()),
        Some("system")
    );

    let user = input[1]
        .get("content")
        .and_then(|value| value.as_array())
        .expect("user content");
    assert_eq!(
        user[0].get("type").and_then(|value| value.as_str()),
        Some("input_text")
    );
    assert_eq!(
        user[0].get("text").and_then(|value| value.as_str()),
        Some("user")
    );
}

#[test]
fn responses_payload_omits_temperature_when_none() {
    let payload =
        payloads::responses_base_payload("gpt-5-nano-2025-08-07", "system", "user", None, true);
    assert!(payload.get("temperature").is_none());
}

#[test]
fn responses_payload_sets_reasoning_minimal_for_gpt5() {
    let payload =
        payloads::responses_base_payload("gpt-5-nano-2025-08-07", "system", "user", None, true);
    let effort = payload
        .get("reasoning")
        .and_then(|value| value.get("effort"))
        .and_then(|value| value.as_str());
    assert_eq!(effort, Some("minimal"));
    assert!(payload.get("text").is_some());
}

#[test]
fn responses_payload_skips_reasoning_for_non_gpt5() {
    let payload = payloads::responses_base_payload("gpt-4o-mini", "system", "user", None, false);
    assert!(payload.get("reasoning").is_none());
}

#[test]
fn chat_payload_omits_temperature_when_none() {
    let payload = payloads::chat_payload("gpt-5-nano-2025-08-07", "system", "user", 100, None);
    assert!(payload.get("temperature").is_none());
}

#[test]
fn unsupported_param_matches_openai_message() {
    let err = CoreError::Provider(
        "openai error 400 Bad Request: {\"error\": {\"message\": \"Unsupported parameter: 'temperature' is not supported with this model.\", \"type\": \"invalid_request_error\", \"param\": \"temperature\", \"code\": null}}"
            .to_string(),
    );

    assert!(is_unsupported_param(&err, "temperature"));
}

#[test]
fn provider_is_gpt5_detection() {
    let provider = OpenAiProvider::new(
        "gpt-5-nano-2025-08-07".to_string(),
        "https://api.openai.com/v1".to_string(),
        OpenAiMode::Responses,
        5,
        Some("test-key".to_string()),
    )
    .expect("provider");

    assert!(provider.is_gpt5());
}
