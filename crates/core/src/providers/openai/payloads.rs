use serde_json::Value;

pub(super) fn responses_base_payload(
    model: &str,
    system_prompt: &str,
    user_prompt: &str,
    temperature: Option<f32>,
    gpt5: bool,
) -> Value {
    let mut payload = serde_json::json!({
        "model": model,
        "input": [
            {
                "role": "system",
                "content": [{ "type": "input_text", "text": system_prompt }]
            },
            {
                "role": "user",
                "content": [{ "type": "input_text", "text": user_prompt }]
            }
        ]
    });

    if let Some(obj) = payload.as_object_mut() {
        if gpt5 {
            obj.insert(
                "reasoning".to_string(),
                serde_json::json!({ "effort": "minimal" }),
            );
            obj.insert(
                "text".to_string(),
                serde_json::json!({ "format": { "type": "text" } }),
            );
        }
        if let Some(value) = temperature {
            obj.insert("temperature".to_string(), serde_json::json!(value));
        }
    }

    payload
}

pub(super) fn chat_payload(
    model: &str,
    system_prompt: &str,
    user_prompt: &str,
    max_tokens: u32,
    temperature: Option<f32>,
) -> Value {
    let mut payload = serde_json::json!({
        "model": model,
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": user_prompt }
        ],
        "max_tokens": max_tokens
    });

    if let Some(obj) = payload.as_object_mut() {
        if let Some(value) = temperature {
            obj.insert("temperature".to_string(), serde_json::json!(value));
        }
    }

    payload
}

pub(super) fn is_gpt5_model(model: &str) -> bool {
    model.trim().to_lowercase().starts_with("gpt-5")
}
