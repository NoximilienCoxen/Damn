//! `cargo run --bin dump`: disegna una scena di prova e la salva in
//! `egresso.ppm`, senza bisogno di una finestra.
//!
//! Il formato PPM è il più semplice che esista: un'intestazione di testo
//! e poi i byte R, G, B di ogni pixel. Si apre con GIMP, IrfanView,
//! l'anteprima di macOS o un qualunque visualizzatore online.
//!
//! Funziona solo dopo aver risolto gli esercizi della lezione 01:
//! prima, i `todo!()` lo fanno fermare con un panic.

use std::fs::File;
use std::io::{BufWriter, Write};

use egresso::engine::framebuffer::{rgb, Framebuffer};

const LARGHEZZA: usize = 320;
const ALTEZZA: usize = 180;

fn main() -> std::io::Result<()> {
    let mut fb = Framebuffer::new(LARGHEZZA, ALTEZZA);

    // Una stanza della Hall: buio, pavimento, due muri e una porta.
    fb.clear(rgb(10, 8, 20));
    fb.fill_rect(0, 150, 320, 30, rgb(40, 34, 60)); // pavimento
    fb.fill_rect(0, 0, 16, 150, rgb(28, 24, 44)); // muro sinistro
    fb.fill_rect(304, 0, 16, 150, rgb(28, 24, 44)); // muro destro
    fb.fill_rect(148, 102, 24, 48, rgb(90, 70, 40)); // la porta
    fb.fill_rect(166, 124, 2, 2, rgb(230, 200, 90)); // la maniglia
    fb.fill_rect(60, 136, 8, 14, rgb(220, 220, 230)); // il protagonista
    fb.fill_rect(-10, 60, 30, 4, rgb(200, 60, 80)); // esce dallo schermo: viene tagliato

    // Qualche "stella" pixel per pixel, per provare set_pixel.
    for i in 0..40 {
        let x = (i * 73 % 300) + 10;
        let y = i * 37 % 90;
        fb.set_pixel(x, y, rgb(120, 110, 160));
    }

    salva_ppm(&fb, "egresso.ppm")?;
    println!("Scritto egresso.ppm ({LARGHEZZA}x{ALTEZZA})");
    Ok(())
}

/// Scrive il framebuffer in formato PPM binario (P6).
fn salva_ppm(fb: &Framebuffer, percorso: &str) -> std::io::Result<()> {
    let mut out = BufWriter::new(File::create(percorso)?);
    write!(out, "P6\n{} {}\n255\n", fb.larghezza(), fb.altezza())?;
    for &p in fb.pixel() {
        // Estraiamo i tre byte dal colore 0x00RRGGBB: l'operazione inversa di `rgb`.
        let r = (p >> 16) as u8;
        let g = (p >> 8) as u8;
        let b = p as u8;
        out.write_all(&[r, g, b])?;
    }
    Ok(())
}
