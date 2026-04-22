use serde::{Deserialize, Serialize};
use std::env;
use thiserror::Error;

pub const DEFAULT_BASE_URL: &str = "https://api.groq.com/openai/v1";
pub const DEFAULT_MODEL: &str = "llama-3.3-70b-versatile";
pub const ENV_TOKEN: &str = "GROQ_TOKEN";
pub const ENV_BASE_URL: &str = "GROQ_BASE_URL";
pub const ENV_MODEL: &str = "GROQ_MODEL";

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("missing environment variable: {0}")]
    MissingEnv(&'static str),
}

#[derive(Debug, Error)]
pub enum LlmError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("failed to parse response JSON: {0}")]
    Deserialize(String),
    #[error("API returned error (status {status}): {body}")]
    Api { status: u16, body: String },
    #[error("response contained no choices")]
    EmptyResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self { role: "user".to_string(), content: content.into() }
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self { role: "system".to_string(), content: content.into() }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self { role: "assistant".to_string(), content: content.into() }
    }
}

#[derive(Debug, Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: &'a [ChatMessage],
    max_tokens: u32,
    temperature: f32,
    top_p: f32,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Debug, Deserialize)]
struct ResponseMessage {
    content: String,
}

pub struct LlmClient {
    base_url: String,
    api_key: String,
    model: String,
    http: reqwest::blocking::Client,
}

impl LlmClient {
    pub fn new(
        base_url: impl Into<String>,
        api_key: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        Self {
            base_url: base_url.into(),
            api_key: api_key.into(),
            model: model.into(),
            http: reqwest::blocking::Client::new(),
        }
    }

    pub fn from_env() -> Result<Self, ConfigError> {
        let api_key = env::var(ENV_TOKEN).map_err(|_| ConfigError::MissingEnv(ENV_TOKEN))?;
        let base_url = env::var(ENV_BASE_URL).unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());
        let model = env::var(ENV_MODEL).unwrap_or_else(|_| DEFAULT_MODEL.to_string());
        Ok(Self::new(base_url, api_key, model))
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub fn complete(&self, messages: &[ChatMessage], max_tokens: u32) -> Result<String, LlmError> {
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let body = ChatRequest {
            model: &self.model,
            messages,
            max_tokens,
            temperature: 0.7,
            top_p: 1.0,
        };

        let resp = self
            .http
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()?;

        let status = resp.status();
        let text = resp.text()?;

        if !status.is_success() {
            return Err(LlmError::Api { status: status.as_u16(), body: text });
        }

        let parsed: ChatResponse =
            serde_json::from_str(&text).map_err(|e| LlmError::Deserialize(e.to_string()))?;
        parsed
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or(LlmError::EmptyResponse)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_chat_response() {
        let raw = r#"{
            "id": "chatcmpl-xyz",
            "object": "chat.completion",
            "model": "llama-3.3-70b-versatile",
            "choices": [
                {
                    "index": 0,
                    "message": { "role": "assistant", "content": "Hello there!" },
                    "finish_reason": "stop"
                }
            ],
            "usage": { "prompt_tokens": 5, "completion_tokens": 3, "total_tokens": 8 }
        }"#;

        let parsed: ChatResponse = serde_json::from_str(raw).expect("valid JSON should parse");
        assert_eq!(parsed.choices.len(), 1);
        assert_eq!(parsed.choices[0].message.content, "Hello there!");
    }

    #[test]
    fn from_env_fails_when_token_missing() {
        let prev = env::var(ENV_TOKEN).ok();
        unsafe { env::remove_var(ENV_TOKEN); }
        let result = LlmClient::from_env();
        let err = match result {
            Ok(_) => panic!("expected ConfigError when {ENV_TOKEN} is unset"),
            Err(e) => e,
        };
        match err {
            ConfigError::MissingEnv(name) => assert_eq!(name, ENV_TOKEN),
        }
        if let Some(v) = prev {
            unsafe { env::set_var(ENV_TOKEN, v); }
        }
    }

    #[test]
    fn chat_message_helpers_set_role() {
        assert_eq!(ChatMessage::user("hi").role, "user");
        assert_eq!(ChatMessage::system("ctx").role, "system");
        assert_eq!(ChatMessage::assistant("ok").role, "assistant");
    }
}
