mod none;

#[cfg(feature = "nnnoiseless")]
mod nnnoiseless;

use once_cell::sync::OnceCell;
use std::sync::Mutex;

use crate::config::structs::NoiseSuppressionBackend;

static BACKEND: OnceCell<NoiseSuppressionBackend> = OnceCell::new();

#[cfg(feature = "nnnoiseless")]
static NNNOISELESS_STATE: OnceCell<Mutex<nnnoiseless::NnnoiselessNS>> = OnceCell::new();

pub fn init(backend: NoiseSuppressionBackend) {
    if BACKEND.get().is_some() {
        return;
    }

    BACKEND.set(backend).ok();

    match backend {
        NoiseSuppressionBackend::None => {
            info!("Noise suppression: disabled");
        }
        #[cfg(feature = "nnnoiseless")]
        NoiseSuppressionBackend::Nnnoiseless => {
            NNNOISELESS_STATE.set(Mutex::new(nnnoiseless::NnnoiselessNS::new())).ok();
            info!("Noise suppression: Nnnoiseless");
        }
        #[cfg(not(feature = "nnnoiseless"))]
        NoiseSuppressionBackend::Nnnoiseless => {
            warn!("Nnnoiseless not compiled in, falling back to None");
            BACKEND.set(NoiseSuppressionBackend::None).ok();
        }
    }
}

pub fn process(input: &[i16]) -> Vec<i16> {
    match BACKEND.get() {
        Some(NoiseSuppressionBackend::None) | None => none::process(input),
        #[cfg(feature = "nnnoiseless")]
        Some(NoiseSuppressionBackend::Nnnoiseless) => {
            if let Some(state) = NNNOISELESS_STATE.get() {
                state.lock().unwrap().process(input)
            } else {
                none::process(input)
            }
        }
        #[cfg(not(feature = "nnnoiseless"))]
        Some(NoiseSuppressionBackend::Nnnoiseless) => none::process(input),
    }
}

pub fn reset() {
    match BACKEND.get() {
        #[cfg(feature = "nnnoiseless")]
        Some(NoiseSuppressionBackend::Nnnoiseless) => {
            if let Some(state) = NNNOISELESS_STATE.get() {
                state.lock().unwrap().reset();
            }
        }
        _ => {}
    }
}