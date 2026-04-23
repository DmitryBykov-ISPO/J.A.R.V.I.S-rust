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

#[cfg(test)]
mod tests {
    use super::*;

    fn run(verdicts: &[bool]) -> (WindowDecision, usize) {
        let mut w = ListenWindow::with_params(30, 1200, 500, 1000, 15000);
        for (i, &v) in verdicts.iter().enumerate() {
            let d = w.push(v);
            if d != WindowDecision::KeepListening {
                return (d, i + 1);
            }
        }
        (WindowDecision::KeepListening, verdicts.len())
    }

    #[test]
    fn closes_after_speech_then_silence() {
        let speech: Vec<bool> = std::iter::repeat(true).take(40).collect();
        let silence: Vec<bool> = std::iter::repeat(false).take(60).collect();
        let mut v = speech;
        v.extend(silence);

        let (decision, frame_idx) = run(&v);
        assert_eq!(decision, WindowDecision::Close);
        let elapsed_ms = (frame_idx as u32) * 30;
        let speech_ms = 40 * 30;
        assert!(elapsed_ms >= speech_ms + 1200, "closed at {} ms, expected >= {}", elapsed_ms, speech_ms + 1200);
        assert!(elapsed_ms <= speech_ms + 1200 + 30, "closed too late at {} ms", elapsed_ms);
    }

    #[test]
    fn does_not_close_before_min_listen_even_with_long_silence() {
        let v: Vec<bool> = std::iter::repeat(false).take(100).collect();
        let mut w = ListenWindow::with_params(30, 1200, 500, 1000, 15000);
        for &x in v.iter().take(33) {
            assert_eq!(w.push(x), WindowDecision::KeepListening);
        }
        assert!(w.elapsed_ms() <= 1000);
    }

    #[test]
    fn hard_cap_fires_at_max_listen() {
        let v: Vec<bool> = std::iter::repeat(true).take(1000).collect();
        let (decision, frame_idx) = run(&v);
        assert_eq!(decision, WindowDecision::HardCap);
        assert_eq!((frame_idx as u32) * 30, 15000);
    }

    #[test]
    fn silence_only_does_not_close_keeps_waiting_until_hard_cap() {
        let v: Vec<bool> = std::iter::repeat(false).take(600).collect();
        let (decision, _) = run(&v);
        assert_eq!(decision, WindowDecision::HardCap);
    }
}
