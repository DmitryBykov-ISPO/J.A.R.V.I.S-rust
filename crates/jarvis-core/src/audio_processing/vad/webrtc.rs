use webrtc_vad::{SampleRate, Vad, VadMode};

use crate::config;

pub const FRAME_SAMPLES: usize = 480;
pub const FRAME_MS: u32 = 30;

pub struct WebRtcVad {
    inner: Vad,
    buf: Vec<i16>,
}

impl WebRtcVad {
    pub fn new() -> Self {
        Self::with_aggressiveness(config::VAD_AGGRESSIVENESS)
    }

    pub fn with_aggressiveness(level: u8) -> Self {
        let mode = match level {
            0 => VadMode::Quality,
            1 => VadMode::LowBitrate,
            2 => VadMode::Aggressive,
            _ => VadMode::VeryAggressive,
        };
        Self {
            inner: Vad::new_with_rate_and_mode(SampleRate::Rate16kHz, mode),
            buf: Vec::with_capacity(FRAME_SAMPLES * 2),
        }
    }

    pub fn push_samples(&mut self, samples: &[i16]) -> Vec<bool> {
        self.buf.extend_from_slice(samples);
        let mut out = Vec::new();
        while self.buf.len() >= FRAME_SAMPLES {
            let frame: Vec<i16> = self.buf.drain(..FRAME_SAMPLES).collect();
            let is_speech = self.inner.is_voice_segment(&frame).unwrap_or(false);
            out.push(is_speech);
        }
        out
    }

    pub fn reset(&mut self) {
        self.buf.clear();
    }
}

impl Default for WebRtcVad {
    fn default() -> Self { Self::new() }
}
