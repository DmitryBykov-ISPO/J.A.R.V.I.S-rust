use crate::config;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowDecision {
    KeepListening,
    Close,
    HardCap,
}

#[derive(Debug, Clone)]
pub struct ListenWindow {
    frame_ms: u32,
    elapsed_ms: u32,
    speech_ms: u32,
    silence_ms: u32,
    end_silence_ms: u32,
    min_speech_ms: u32,
    min_listen_ms: u32,
    max_listen_ms: u32,
}

impl ListenWindow {
    pub fn new(frame_ms: u32) -> Self {
        Self::with_params(
            frame_ms,
            config::VAD_COMMAND_END_SILENCE_MS,
            config::VAD_COMMAND_MIN_SPEECH_MS,
            config::VAD_COMMAND_MIN_LISTEN_MS,
            config::VAD_COMMAND_MAX_LISTEN_MS,
        )
    }

    pub fn with_params(
        frame_ms: u32,
        end_silence_ms: u32,
        min_speech_ms: u32,
        min_listen_ms: u32,
        max_listen_ms: u32,
    ) -> Self {
        Self {
            frame_ms,
            elapsed_ms: 0,
            speech_ms: 0,
            silence_ms: 0,
            end_silence_ms,
            min_speech_ms,
            min_listen_ms,
            max_listen_ms,
        }
    }

    pub fn push(&mut self, is_speech: bool) -> WindowDecision {
        self.elapsed_ms = self.elapsed_ms.saturating_add(self.frame_ms);

        if is_speech {
            self.speech_ms = self.speech_ms.saturating_add(self.frame_ms);
            self.silence_ms = 0;
        } else {
            self.silence_ms = self.silence_ms.saturating_add(self.frame_ms);
        }

        if self.elapsed_ms >= self.max_listen_ms {
            return WindowDecision::HardCap;
        }
        if self.elapsed_ms < self.min_listen_ms {
            return WindowDecision::KeepListening;
        }
        if self.speech_ms >= self.min_speech_ms && self.silence_ms >= self.end_silence_ms {
            return WindowDecision::Close;
        }
        WindowDecision::KeepListening
    }

    pub fn elapsed_ms(&self) -> u32 { self.elapsed_ms }
    pub fn speech_ms(&self) -> u32 { self.speech_ms }
    pub fn silence_ms(&self) -> u32 { self.silence_ms }

    pub fn had_speech(&self) -> bool { self.speech_ms > 0 }
}
