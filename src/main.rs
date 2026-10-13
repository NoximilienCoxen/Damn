//! EGRESSO.
//!
//! `cargo run --release`                          gioca
//! `cargo run --release -- --headless [opzioni]`  gira senza finestra e salva PNG
//!
//! Opzioni headless:
//!   --frames N         quanti passi simulare (default 600)
//!   --script TESTO     input registrato, es. "10:right:+ 80:right:- 90:e:!"
//!   --script-file F    come sopra, letto da file
//!   --shots 10,200     frame da salvare
//!   --every N          salva un frame ogni N
//!   --out CARTELLA     dove salvare (default "shots")
//!   --trace            stampa lo stato a ogni frame
//!   --seme N           la notte da generare (default 1)

use egresso::engine::app::{parse_script, run_headless};
use egresso::engine::window::run_window;
use egresso::game::Egresso;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if !args.iter().any(|a| a == "--headless") {
        // Ogni partita una notte diversa.
        let seme = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(1, |d| d.subsec_nanos() | 1);
        run_window(Egresso::con_seme(seme), "EGRESSO");
        return;
    }

    let valore = |nome: &str| args.iter().position(|a| a == nome).and_then(|i| args.get(i + 1)).cloned();
    let frames = valore("--frames").and_then(|v| v.parse().ok()).unwrap_or(600);
    let ogni = valore("--every").and_then(|v| v.parse().ok()).unwrap_or(0);
    let cartella = valore("--out").unwrap_or_else(|| "shots".into());
    let scatti: Vec<u32> = valore("--shots").map(|v| v.split(',').filter_map(|s| s.trim().parse().ok()).collect()).unwrap_or_default();
    let testo = match (valore("--script"), valore("--script-file")) {
        (Some(s), _) => s,
        (None, Some(f)) => std::fs::read_to_string(&f).unwrap_or_else(|e| panic!("non riesco a leggere {f}: {e}")),
        _ => String::new(),
    };
    let script = parse_script(&testo).unwrap_or_else(|e| panic!("script non valido: {e}"));

    let mut gioco = Egresso::new();
    if let Some(n) = valore("--seme").and_then(|v| v.parse().ok()) {
        gioco = Egresso::con_seme(n);
    }
    let traccia = args.iter().any(|a| a == "--trace");
    run_headless(&mut gioco, frames, &script, &scatti, ogni, &cartella, traccia).expect("errore salvando gli screenshot");
    println!("ok: {frames} frame simulati, screenshot in {cartella}/");
}
