use once_cell::sync::OnceCell;
use parking_lot::Mutex;

use jarvis_core::config;
use jarvis_core::i18n;
use jarvis_core::ipc::{self, IpcEvent};
use jarvis_core::llm::{ChatMessage, ConversationHistory, LlmClient};
use jarvis_core::voices;

struct State {
    client: LlmClient,
    history: Mutex<ConversationHistory>,
    max_tokens: u32,
}

static STATE: OnceCell<Option<State>> = OnceCell::new();

pub fn init() {
    let _ = STATE.set(build_state());
}

fn build_state() -> Option<State> {
    if !config::LLM_DEFAULT_ENABLED {
        info!("LLM fallback disabled by config.");
        return None;
    }

    let client = match LlmClient::from_env() {
        Ok(c) => c,
        Err(e) => {
            warn!("LLM fallback disabled: {}. Set GROQ_TOKEN to enable.", e);
            return None;
        }
    };

    let lang = i18n::get_language();
    let prompt = config::get_llm_system_prompt(&lang);
    let history = Mutex::new(ConversationHistory::new(prompt, config::LLM_DEFAULT_MAX_HISTORY));

    info!("LLM fallback enabled (model: {}).", client.model());
    Some(State {
        client,
        history,
        max_tokens: config::LLM_DEFAULT_MAX_TOKENS,
    })
}

pub fn is_enabled() -> bool {
    STATE.get().and_then(|s| s.as_ref()).is_some()
}

pub fn extract_prompt(text: &str) -> Option<String> {
    let lang = i18n::get_language();
    let triggers = config::get_llm_trigger_phrases(&lang);
    let lowered = text.to_lowercase();

    for trig in triggers {
        if let Some(rest) = strip_trigger(&lowered, trig) {
            let trimmed = rest.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

fn strip_trigger(text: &str, trigger: &str) -> Option<String> {
    let t = text.trim_start();
    if let Some(after) = t.strip_prefix(trigger) {
        let next = after.chars().next();
        if next.map_or(true, |c| !c.is_alphanumeric()) {
            return Some(after.to_string());
        }
    }
    None
}

pub fn handle(prompt: &str) {
    let state = match STATE.get().and_then(|s| s.as_ref()) {
        Some(s) => s,
        None => {
            warn!("LLM fallback called while disabled — ignoring.");
            return;
        }
    };

    info!("LLM prompt: {}", prompt);

    let snapshot: Vec<ChatMessage> = {
        let mut h = state.history.lock();
        h.push_user(prompt);
        h.snapshot()
    };

    match state.client.complete(&snapshot, state.max_tokens) {
        Ok(reply) => {
            let reply = reply.trim().to_string();
            info!("LLM reply: {}", reply);
            state.history.lock().push_assistant(reply.clone());
            ipc::send(IpcEvent::LlmReply { text: reply });
            voices::play_ok();
        }
        Err(e) => {
            error!("LLM request failed: {}", e);
            state.history.lock().pop_last_user();
            ipc::send(IpcEvent::LlmReply {
                text: config::LLM_FALLBACK_ERROR_RU.to_string(),
            });
            voices::play_error();
        }
    }
}
