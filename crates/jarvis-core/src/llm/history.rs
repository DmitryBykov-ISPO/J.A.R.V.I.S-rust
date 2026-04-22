use super::client::ChatMessage;

pub struct ConversationHistory {
    system: Option<ChatMessage>,
    turns: Vec<ChatMessage>,
    max_turns: usize,
}

impl ConversationHistory {
    pub fn new(system_prompt: impl Into<String>, max_turns: usize) -> Self {
        Self {
            system: Some(ChatMessage::system(system_prompt)),
            turns: Vec::new(),
            max_turns: max_turns.max(1),
        }
    }

    pub fn without_system(max_turns: usize) -> Self {
        Self {
            system: None,
            turns: Vec::new(),
            max_turns: max_turns.max(1),
        }
    }

    pub fn push_user(&mut self, content: impl Into<String>) {
        self.turns.push(ChatMessage::user(content));
        self.truncate();
    }

    pub fn push_assistant(&mut self, content: impl Into<String>) {
        self.turns.push(ChatMessage::assistant(content));
        self.truncate();
    }

    pub fn snapshot(&self) -> Vec<ChatMessage> {
        let mut out = Vec::with_capacity(self.turns.len() + 1);
        if let Some(s) = &self.system {
            out.push(s.clone());
        }
        out.extend(self.turns.iter().cloned());
        out
    }

    pub fn turns(&self) -> &[ChatMessage] {
        &self.turns
    }

    pub fn clear(&mut self) {
        self.turns.clear();
    }

    fn truncate(&mut self) {
        if self.turns.len() > self.max_turns {
            let drop = self.turns.len() - self.max_turns;
            self.turns.drain(0..drop);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evicts_oldest_turns_but_keeps_system_prompt() {
        let mut h = ConversationHistory::new("you are jarvis", 4);
        h.push_user("u1");
        h.push_assistant("a1");
        h.push_user("u2");
        h.push_assistant("a2");
        h.push_user("u3");

        let snap = h.snapshot();
        assert_eq!(snap[0].role, "system");
        assert_eq!(snap[0].content, "you are jarvis");
        assert_eq!(snap.len(), 1 + 4);

        let contents: Vec<&str> = snap.iter().skip(1).map(|m| m.content.as_str()).collect();
        assert_eq!(contents, vec!["a1", "u2", "a2", "u3"]);
    }

    #[test]
    fn snapshot_with_no_system_returns_only_turns() {
        let mut h = ConversationHistory::without_system(3);
        h.push_user("hi");
        h.push_assistant("hello");
        let snap = h.snapshot();
        assert_eq!(snap.len(), 2);
        assert_eq!(snap[0].role, "user");
        assert_eq!(snap[1].role, "assistant");
    }

    #[test]
    fn cap_of_zero_is_clamped_to_one() {
        let mut h = ConversationHistory::new("sys", 0);
        h.push_user("a");
        h.push_user("b");
        let snap = h.snapshot();
        assert_eq!(snap.len(), 2);
        assert_eq!(snap[0].role, "system");
        assert_eq!(snap[1].content, "b");
    }

    #[test]
    fn clear_removes_turns_but_not_system() {
        let mut h = ConversationHistory::new("sys", 4);
        h.push_user("u");
        h.push_assistant("a");
        h.clear();
        let snap = h.snapshot();
        assert_eq!(snap.len(), 1);
        assert_eq!(snap[0].role, "system");
    }
}
