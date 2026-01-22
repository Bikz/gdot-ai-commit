use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProviderKind {
    OpenAi,
    Ollama,
}

impl ProviderKind {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            ProviderKind::OpenAi => "openai",
            ProviderKind::Ollama => "ollama",
        }
    }
}

impl std::str::FromStr for ProviderKind {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_lowercase().as_str() {
            "openai" => Ok(ProviderKind::OpenAi),
            "ollama" => Ok(ProviderKind::Ollama),
            other => Err(format!("unknown provider: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OpenAiMode {
    Auto,
    Responses,
    Chat,
}

impl std::str::FromStr for OpenAiMode {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_lowercase().as_str() {
            "auto" => Ok(OpenAiMode::Auto),
            "responses" => Ok(OpenAiMode::Responses),
            "chat" => Ok(OpenAiMode::Chat),
            other => Err(format!("unknown openai mode: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StageMode {
    Auto,
    All,
    None,
    Interactive,
}

impl std::str::FromStr for StageMode {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_lowercase().as_str() {
            "auto" => Ok(StageMode::Auto),
            "all" => Ok(StageMode::All),
            "none" => Ok(StageMode::None),
            "interactive" => Ok(StageMode::Interactive),
            other => Err(format!("unknown stage mode: {other}")),
        }
    }
}
