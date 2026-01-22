use super::types::{OpenAiMode, ProviderKind};
use super::values::Config;

#[test]
fn merge_overrides_defaults() {
    let base = Config::defaults();
    let override_config = Config {
        push: Some(false),
        ..Config::default()
    };

    let merged = base.merge(override_config).resolve().expect("resolve");
    assert!(!merged.push);
}

#[test]
fn resolve_forces_responses_for_gpt5_openai() {
    let config = Config {
        provider: Some(ProviderKind::OpenAi),
        model: Some("gpt-5-nano-2025-08-07".to_string()),
        openai_mode: Some(OpenAiMode::Chat),
        ..Config::default()
    };

    let resolved = config.resolve().expect("resolve");
    assert_eq!(resolved.openai_mode, OpenAiMode::Responses);
}
