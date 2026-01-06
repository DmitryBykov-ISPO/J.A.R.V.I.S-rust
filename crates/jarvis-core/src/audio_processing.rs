pub mod noise_suppression;
pub mod vad;
pub mod gain_normalizer;

use once_cell::sync::OnceCell;
use std::sync::Mutex;

use crate::config::structs::{NoiseSuppressionBackend, VadBackend};
use crate::DB;

static PROCESSOR: OnceCell<Mutex<AudioProcessor>> = OnceCell::new();

#[derive(Debug, Clone)]
pub struct ProcessedAudio {
    pub samples: Vec<i16>,
    pub is_voice: bool,
    pub vad_confidence: f32,
}

struct AudioProcessor {
    ns_backend: NoiseSuppressionBackend,
    vad_backend: VadBackend,
    gain_enabled: bool,
}

impl AudioProcessor {
    fn new(ns: NoiseSuppressionBackend, vad: VadBackend, gain: bool) -> Self {
        // init backends
        noise_suppression::init(ns);
        vad::init(vad);
        if gain {
            gain_normalizer::init();
        }

        Self {
            ns_backend: ns,
            vad_backend: vad,
            gain_enabled: gain,
        }
    }

    fn process(&mut self, input: &[i16]) -> ProcessedAudio {
        let mut samples = input.to_vec();

        // step 1: gain normalization (before other processing)
        if self.gain_enabled {
            samples = gain_normalizer::normalize(&samples);
        }

        // step 2: noise suppression
        samples = noise_suppression::process(&samples);

        // step 3: VAD
        let (is_voice, confidence) = vad::detect(&samples);

        ProcessedAudio {
            samples,
            is_voice,
            vad_confidence: confidence,
        }
    }

    fn reset(&mut self) {
        noise_suppression::reset();
        vad::reset();
        gain_normalizer::reset();
    }
}



pub fn init() -> Result<(), String> {
    if PROCESSOR.get().is_some() {
        return Ok(());
    }

    let (ns, vad, gain) = get_settings();
    info!("Initializing audio processing: NS={:?}, VAD={:?}, Gain={}", ns, vad, gain);

    let processor = AudioProcessor::new(ns, vad, gain);
    PROCESSOR
        .set(Mutex::new(processor))
        .map_err(|_| "Audio processor already initialized")?;

    info!("Audio processing initialized.");
    Ok(())
}

pub fn process(input: &[i16]) -> ProcessedAudio {
    match PROCESSOR.get() {
        Some(p) => p.lock().unwrap().process(input),
        None => ProcessedAudio {
            samples: input.to_vec(),
            is_voice: true,
            vad_confidence: 1.0,
        },
    }
}

pub fn reset() {
    if let Some(p) = PROCESSOR.get() {
        p.lock().unwrap().reset();
    }
}

fn get_settings() -> (NoiseSuppressionBackend, VadBackend, bool) {
    match DB.get() {
        Some(db) => {
            let settings = db.read();
            (settings.noise_suppression, settings.vad, settings.gain_normalizer)
        }
        None => (
            crate::config::DEFAULT_NOISE_SUPPRESSION,
            crate::config::DEFAULT_VAD,
            crate::config::DEFAULT_GAIN_NORMALIZER,
        ),
    }
}