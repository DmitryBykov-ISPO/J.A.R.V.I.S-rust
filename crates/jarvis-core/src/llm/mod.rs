mod client;
mod history;

pub use client::{
    ChatMessage, ConfigError, LlmClient, LlmError,
    DEFAULT_BASE_URL, DEFAULT_MODEL, ENV_BASE_URL, ENV_MODEL, ENV_TOKEN,
};
pub use history::ConversationHistory;
