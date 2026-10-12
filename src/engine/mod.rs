//! L'engine: i pezzi riutilizzabili che non sanno niente di EGRESSO.
//!
//! Regola d'oro: niente in `engine/` deve parlare di occhi, parole o
//! paperelle. Quella è roba di `game/`.

pub mod app;
pub mod font;
pub mod framebuffer;
pub mod glow;
pub mod input;
pub mod math;
pub mod png;
pub mod time;
pub mod web;
#[cfg(not(target_arch = "wasm32"))]
pub mod window;
