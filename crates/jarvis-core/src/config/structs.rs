use std::fmt;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq)]
pub enum WakeWordEngine {
    Rustpotter,
    Vosk,
    Porcupine,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq)]
pub enum IntentRecognitionEngine {
    IntentClassifier,
    EmbeddingClassifier,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq)]
pub enum NoiseSuppressionBackend {
    None,
    Nnnoiseless,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq)]
pub enum VadBackend {
    None,
    Energy,
    Nnnoiseless,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SpeechToTextEngine {
    Vosk,
}

#[derive(PartialEq, Debug)]
pub enum RecorderType {
    Cpal,
    PvRecorder,
    PortAudio,
}

#[derive(PartialEq, Debug)]
pub enum AudioType {
    Rodio,
    Kira,
}



impl fmt::Display for WakeWordEngine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for SpeechToTextEngine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for IntentRecognitionEngine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for NoiseSuppressionBackend {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for VadBackend {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// pub enum TextToSpeechEngine {}

// pub enum IntentRecognitionEngine {}
