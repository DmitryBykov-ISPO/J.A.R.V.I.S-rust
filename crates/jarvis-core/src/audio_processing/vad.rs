mod none;
mod energy;

#[cfg(feature = "nnnoiseless")]
mod nnnoiseless;

use once_cell::sync::OnceCell;
use std::sync::Mutex;

use crate::config::structs::VadBackend;

static BACKEND: OnceCell<VadBackend> = OnceCell::new();

#[cfg(feature = "nnnoiseless")]
static NNNOISELESS_STATE: OnceCell<Mutex<nnnoiseless::NnnoiselessVAD>> = OnceCell::new();

pub fn init(backend: VadBackend) {
    if BACKEND.get().is_some() {
        return;
    }

    BACKEND.set(backend).ok();

    match backend {
        VadBackend::None => {
            info!("VAD: disabled");
        }
        VadBackend::Energy => {
            info!("VAD: Energy-based");
        }
        #[cfg(feature = "nnnoiseless")]
        VadBackend::Nnnoiseless => {
            NNNOISELESS_STATE.set(Mutex::new(nnnoiseless::NnnoiselessVAD::new())).ok();
            info!("VAD: Nnnoiseless");
        }
        #[cfg(not(feature = "nnnoiseless"))]
        VadBackend::Nnnoiseless => {
            warn!("Nnnoiseless not compiled in, falling back to Energy");
            BACKEND.set(VadBackend::Energy).ok();
        }
    }
}

// Returns (is_voice, confidence)
pub fn detect(input: &[i16]) -> (bool, f32) {
    match BACKEND.get() {
        Some(VadBackend::None) | None => none::detect(input),
        Some(VadBackend::Energy) => energy::detect(input),
        #[cfg(feature = "nnnoiseless")]
        Some(VadBackend::Nnnoiseless) => {
            if let Some(state) = NNNOISELESS_STATE.get() {
                state.lock().unwrap().detect(input)
            } else {
                energy::detect(input)
            }
        }
        #[cfg(not(feature = "nnnoiseless"))]
        Some(VadBackend::Nnnoiseless) => energy::detect(input),
    }
}

pub fn reset() {
    match BACKEND.get() {
        #[cfg(feature = "nnnoiseless")]
        Some(VadBackend::Nnnoiseless) => {
            if let Some(state) = NNNOISELESS_STATE.get() {
                state.lock().unwrap().reset();
            }
        }
        _ => {}
    }
}