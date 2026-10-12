//! EGRESSO: il gioco e il suo engine.
//!
//! Questo file è la "radice" della libreria. Ogni `pub mod` qui sotto
//! rende visibile una cartella (o un file) di `src/` al resto del progetto,
//! compresi i programmi in `src/main.rs` e `src/bin/`.

pub mod engine;
pub mod game;

// Nel browser: esporta le funzioni che la pagina web (web/index.html) chiama.
#[cfg(target_arch = "wasm32")]
esporta_web!(game::Egresso, game::Egresso::new());
