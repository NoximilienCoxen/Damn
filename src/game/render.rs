//! Come si disegna EGRESSO.
//!
//! Ordine di un frame:
//! 1. buio e polvere;
//! 2. la realtà, solo dentro le rivelazioni (texture procedurale pre-calcolata);
//! 3. le lettere nominate, le proposte delle voci, gli attributi dei sensi;
//! 4. particelle e protagonista;
//! 5. glow: tutto ciò che emette luce è stato disegnato anche nel buffer
//!    `luce`, che viene sfocato e sommato all'immagine;
//! 6. interfaccia (biglietti, messaggi, suggerimenti), nitida sopra il glow.
//!
//! Prima del glow c'è anche il "vuoto visibile": pulviscolo che cade nelle
//! colonne dove sotto i piedi non c'è niente di nominato.

use super::lessico::{Natura, Voce};
use super::livello::{COLONNE, RIGHE, TILE};
use super::mondo::{Fase, Mondo, Reale};
use super::{Egresso, Scena};
use crate::engine::app::{Stats, ALTEZZA, LARGHEZZA};
use crate::engine::font::{misura, Font, AVANZAMENTO};
use crate::engine::framebuffer::{mix, rgb, scala, Framebuffer};
use crate::engine::math::{hash01, Vec2};

const SFONDO: u32 = rgb(6, 5, 12);
const ATTRIBUTO: u32 = rgb(150, 140, 200);
const CARTA: u32 = rgb(200, 190, 150);

const BAYER: [[f32; 4]; 4] = [
    [0.0 / 16.0, 8.0 / 16.0, 2.0 / 16.0, 10.0 / 16.0],
    [12.0 / 16.0, 4.0 / 16.0, 14.0 / 16.0, 6.0 / 16.0],
    [3.0 / 16.0, 11.0 / 16.0, 1.0 / 16.0, 9.0 / 16.0],
    [15.0 / 16.0, 7.0 / 16.0, 13.0 / 16.0, 5.0 / 16.0],
];

impl Egresso {
    pub(super) fn disegna(&mut self, fb: &mut Framebuffer, stats: &Stats) {
        fb.clear(SFONDO);
        self.luce.clear(0);

        // Polvere che luccica piano.
        for (i, &(x, y, b)) in self.polvere.iter().enumerate() {
            let k = 0.6 + 0.4 * (self.t * 0.7 + i as f32 * 1.3).sin();
            let b = (b as f32 * k) as u8;
            fb.set_pixel(x, y, rgb(b, b, b.saturating_add(8)));
        }

        self.disegna_realta(fb);
        self.disegna_vuoto(fb);
        self.disegna_nomi(fb);
        self.disegna_proposte(fb);
        self.disegna_biglietti(fb);
        self.disegna_particelle(fb);
        self.disegna_giocatore(fb);

        self.glow.apply(&self.luce, fb, 2, 1.2);
        self.glow.apply(&self.luce, fb, 9, 1.6);

        self.disegna_interfaccia(fb, stats);
    }

    fn disegna_realta(&mut self, fb: &mut Framebuffer) {
        let (w, h) = (LARGHEZZA as i32, ALTEZZA as i32);
        for r in &self.mondo.rivelazioni {
            let raggio = r.raggio();
            if raggio <= 0.5 {
                continue;
            }
            let x0 = ((r.centro.x - raggio) as i32).max(0);
            let x1 = ((r.centro.x + raggio) as i32 + 1).min(w);
            let y0 = ((r.centro.y - raggio) as i32).max(0);
            let y1 = ((r.centro.y + raggio) as i32 + 1).min(h);
            for y in y0..y1 {
                for x in x0..x1 {
                    let d = (Vec2::new(x as f32, y as f32) - r.centro).len();
                    // Bordo "a dithering": la realtà affiora a puntini.
                    let v = ((raggio - d) / 12.0).clamp(0.0, 1.0);
                    if v <= BAYER[(y & 3) as usize][(x & 3) as usize] {
                        continue;
                    }
                    let i = (y * w + x) as usize;
                    let k = 0.85 + 0.35 * (1.0 - d / raggio).max(0.0);
                    fb.pixel_mut()[i] = scala(self.realta[i], k);
                    let l = self.realta_luce[i];
                    if l != 0 {
                        self.luce.pixel_mut()[i] = l;
                    }
                }
            }
        }
    }

    /// Dove sotto i piedi non c'è niente che regga, il pulviscolo cade:
    /// si vede dove si precipita.
    fn disegna_vuoto(&mut self, fb: &mut Framebuffer) {
        let g = self.giocatore.rect();
        let piedi = ((g.bottom() + 1.0) / TILE as f32).floor() as i32;
        let y0 = piedi * TILE;
        let altezza = (ALTEZZA as i32 - y0).max(1);
        let col = rgb(150, 140, 200);
        for tx in 0..COLONNE {
            if !self.mondo.colonna_vuota(tx, piedi) {
                continue;
            }
            for k in 0..6 {
                let h = hash01(tx, k, 17);
                let vel = 30.0 + 40.0 * hash01(tx, k, 18);
                let dy = ((self.t * vel + h * altezza as f32) % altezza as f32) as i32;
                let x = tx * TILE + (hash01(tx, k, 19) * 7.0) as i32;
                let y = y0 + dy;
                // Più giù, più si spegne: il vuoto inghiotte.
                let a = (1.0 - dy as f32 / altezza as f32) * 210.0;
                fb.blend_pixel(x, y, col, a as u8);
                fb.blend_pixel(x, y - 1, col, (a * 0.5) as u8);
                fb.blend_pixel(x, y - 2, col, (a * 0.25) as u8);
            }
        }
    }

    fn disegna_nomi(&mut self, fb: &mut Framebuffer) {
        for ty in 0..RIGHE {
            for tx in 0..COLONNE {
                let Some(c) = self.mondo.cella(tx, ty) else { continue };
                let p = &self.mondo.percetti[c.percetto as usize];
                // Appena affermata, la parola si accende di colpo.
                let lampo = match self.lampo {
                    Some((i, t)) if i == c.percetto as usize => t / 0.35,
                    _ => 0.0,
                };
                // Il bordo superiore di ciò che regge: qui si può stare.
                if c.natura.regge() && self.mondo.solido(tx, ty - 1).is_none() {
                    let bordo = scala(c.voce.colore(), 0.5 + lampo);
                    for dx in 0..TILE {
                        fb.blend_pixel(tx * TILE + dx, ty * TILE, bordo, 70);
                    }
                }
                // La certezza si vede: le parole marce sfarfallano.
                let k = 0.45 + 0.55 * p.certezza + lampo;
                if p.certezza < 0.45 && hash01(tx, ty, (self.t * 12.0) as u32) < 0.12 {
                    continue;
                }
                let (alpha, luce) = match c.natura {
                    // Un nome che non regge è un fantasma di lettere.
                    Natura::Vuota => (110, 0.18),
                    _ => (255, 0.5),
                };
                let col = scala(c.voce.colore(), k);
                let (x, y) = (tx * TILE + 1, ty * TILE + 1);
                let inclina = c.voce == Voce::Memoria;
                self.font.draw_char(fb, x, y, c.lettera, col, alpha, inclina);
                self.font.draw_char(&mut self.luce, x, y, c.lettera, scala(col, luce), 255, inclina);
            }
        }
    }

    fn disegna_proposte(&mut self, fb: &mut Framebuffer) {
        let focus = self.focus;
        for (pi, p) in self.mondo.percetti.iter().enumerate() {
            let area = p.rect();
            let cx = area.center().x as i32;
            // Gli attributi: i sensi parlano per primi.
            let alpha_attr = match p.fase {
                Fase::Sentito(t) => (t / 0.5).min(1.0),
                Fase::Proposto => 0.45,
                _ => 0.0,
            };
            if alpha_attr > 0.0 {
                for (k, a) in p.def.attributi.iter().enumerate() {
                    let dy = (self.t * 1.3 + k as f32).sin() * 1.5;
                    let y = area.y as i32 - 10 - 9 * (p.def.attributi.len() - 1 - k) as i32 + dy as i32;
                    let x = dentro_schermo(cx - misura(a) / 2 + (k as i32 - 1) * 14, misura(a));
                    self.font.draw_text_alpha(fb, x, y, a, ATTRIBUTO, (alpha_attr * 200.0) as u8);
                }
            }
            if p.fase != Fase::Proposto {
                continue;
            }
            // Le proposte: tre parole che si contendono lo stesso spazio.
            for (j, pr) in p.def.proposte.iter().enumerate() {
                let in_primo_piano = j == p.focus;
                let alpha: u8 = if in_primo_piano { 255 } else { 120 };
                let col = pr.voce.colore();
                for k in 0..pr.celle {
                    let lettera = p.lettera(j, k, 0);
                    let (tx, ty) = pr.cella(k);
                    let fase = self.t * 11.0 + k as f32 * 1.7 + j as f32 * 2.3;
                    let tr = pr.voce.tremito() * if in_primo_piano { 1.0 } else { 1.6 };
                    let (dx, dy) = ((fase.sin() * tr).round() as i32, ((fase * 1.31).cos() * tr).round() as i32);
                    let (x, y) = (tx * TILE + 1 + dx, ty * TILE + 1 + dy);
                    let inclina = pr.voce == Voce::Memoria;
                    // Una proposta non è ancora reale: lettere retinate che sfrigolano.
                    let sfrigolio = (self.t * 8.0) as i32 + k;
                    self.font.draw_char_retinato(fb, x, y, lettera, col, alpha, inclina, sfrigolio);
                    if in_primo_piano {
                        self.font.draw_char_retinato(&mut self.luce, x, y, lettera, scala(col, 0.35), 255, inclina, sfrigolio);
                    }
                }
            }
            // Il nome della voce in primo piano, se è a fuoco.
            if focus == Some(pi) {
                let voce = p.def.proposte[p.focus].voce;
                let y = area.y as i32 - 10 - 9 * p.def.attributi.len() as i32;
                let testo = voce.nome();
                let cx = dentro_schermo(cx - misura(testo) / 2 - 10, misura(testo) + 20) + misura(testo) / 2 + 10;
                self.font.draw_centered(fb, cx, y, testo, voce.colore(), 220);
                let mezza = misura(testo) / 2;
                self.font.draw_text_alpha(fb, cx - mezza - 10, y, "<", voce.colore(), 120);
                self.font.draw_text_alpha(fb, cx + mezza + 5, y, ">", voce.colore(), 120);
            }
        }
        // Angoli attorno a ciò che è a fuoco.
        if let Some(i) = focus {
            let r = self.mondo.percetti[i].rect().expand(2.0);
            let col = rgb(150, 150, 170);
            let (x0, y0, x1, y1) = (r.x as i32, r.y as i32, r.right() as i32 - 1, r.bottom() as i32 - 1);
            for (x, y, sx, sy) in [(x0, y0, 1, 1), (x1, y0, -1, 1), (x0, y1, 1, -1), (x1, y1, -1, -1)] {
                for d in 0..3 {
                    fb.blend_pixel(x + d * sx, y, col, 150);
                    fb.blend_pixel(x, y + d * sy, col, 150);
                }
            }
        }
    }

    fn disegna_biglietti(&mut self, fb: &mut Framebuffer) {
        for (i, (col, _)) in self.mondo.stanza.biglietti.iter().enumerate() {
            let x = col * TILE + 2;
            let y = 17 * TILE - 3;
            let vicino = self.biglietto == Some(i);
            let c = if vicino { CARTA } else { scala(CARTA, 0.55) };
            fb.fill_rect(x, y, 5, 3, c);
            fb.set_pixel(x + 4, y, scala(c, 0.6));
            self.luce.fill_rect(x, y, 5, 3, scala(c, 0.25));
        }
    }

    fn disegna_particelle(&mut self, fb: &mut Framebuffer) {
        for p in &self.particelle.v {
            let k = (p.vita / p.durata).clamp(0.0, 1.0);
            let (x, y) = (p.pos.x as i32, p.pos.y as i32);
            if p.lettera == ' ' {
                let c = scala(p.colore, k);
                fb.add_pixel(x, y, c);
                self.luce.add_pixel(x, y, scala(c, 0.8));
            } else {
                // Le lettere cadute restano a terra e si spengono piano.
                let alpha = (k.min(0.5) * 2.0 * 200.0) as u8;
                let c = scala(p.colore, 0.75);
                self.font.draw_char(fb, x, y, p.lettera, c, alpha, p.inclina);
                self.font.draw_char(&mut self.luce, x, y, p.lettera, scala(c, 0.15 * k), 255, p.inclina);
            }
        }
    }

    fn disegna_giocatore(&mut self, fb: &mut Framebuffer) {
        let g = &self.giocatore;
        let (x, y) = (g.pos.x.round() as i32, g.pos.y.round() as i32);
        let corpo = rgb(232, 230, 240);
        let ombra = rgb(150, 148, 170);
        // Testa.
        fb.fill_rect(x + 1, y, 4, 4, corpo);
        // Busto.
        fb.fill_rect(x, y + 4, 6, 5, corpo);
        fb.fill_rect(x + if g.verso > 0.0 { 0 } else { 4 }, y + 5, 2, 3, ombra);
        // Gambe: alternate camminando.
        let fase = ((g.passo / 4.0) as i32) & 1;
        if !g.a_terra {
            fb.fill_rect(x + 1, y + 9, 2, 2, corpo);
            fb.fill_rect(x + 3, y + 9, 2, 3, corpo);
        } else if g.vel.x.abs() > 5.0 && fase == 1 {
            fb.fill_rect(x, y + 9, 2, 3, corpo);
            fb.fill_rect(x + 4, y + 9, 2, 2, corpo);
        } else {
            fb.fill_rect(x + 1, y + 9, 2, 3, corpo);
            fb.fill_rect(x + 3, y + 9, 2, 3, corpo);
        }
        // La benda rossa, con il nodo che svolazza dietro.
        let benda = rgb(220, 40, 52);
        fb.fill_rect(x, y + 1, 6, 2, benda);
        let dietro = if g.verso > 0.0 { x - 1 } else { x + 6 };
        let sv = ((self.t * 9.0).sin() * 1.2).round() as i32;
        fb.set_pixel(dietro, y + 1 + sv.max(0), benda);
        fb.set_pixel(dietro - g.verso as i32, y + 2 + sv, benda);
        self.luce.fill_rect(x, y + 1, 6, 2, rgb(40, 6, 8));
    }

    fn disegna_interfaccia(&mut self, fb: &mut Framebuffer, stats: &Stats) {
        let w = LARGHEZZA as i32;

        // Titolo iniziale.
        if self.t < 3.0 {
            let a = (if self.t < 1.8 { 1.0 } else { (3.0 - self.t) / 1.2 }).clamp(0.0, 1.0);
            testo_grande(&self.font, fb, w / 2, 38, "EGRESSO", rgb(240, 232, 214), (a * 255.0) as u8, 3);
            self.font.draw_centered(fb, w / 2, 66, "IL MONDO E' CIO' CHE DICI CHE SIA.", rgb(150, 140, 190), (a * 200.0) as u8);
        }

        // Il nome della stanza, entrando (non sopra il titolo iniziale).
        if self.t > 3.5 && self.t_stanza < 2.5 {
            let a = (if self.t_stanza < 1.5 { 1.0 } else { (2.5 - self.t_stanza) / 1.0 }).clamp(0.0, 1.0);
            testo_grande(&self.font, fb, w / 2, 30, self.mondo.stanza.nome, rgb(240, 232, 214), (a * 230.0) as u8, 2);
        }

        self.disegna_suggerimento(fb);

        // Il biglietto vicino (non mentre c'è il titolo).
        if let Some(i) = self.biglietto.filter(|_| self.t > 3.0) {
            let righe = self.mondo.stanza.biglietti[i].1;
            for (k, r) in righe.iter().enumerate() {
                self.font.draw_centered(fb, w / 2, 12 + k as i32 * 9, r, CARTA, if k == 0 { 130 } else { 220 });
            }
        }

        // La voce che commenta.
        if let Some(m) = &self.messaggio {
            let a = if m.t < 2.0 { 1.0 } else { (2.5 - m.t) / 0.5 };
            let tr = m.voce.tremito();
            let dx = ((self.t * 25.0).sin() * tr).round() as i32;
            self.font.draw_centered(fb, w / 2 + dx, 46, m.testo, m.voce.colore(), (a.clamp(0.0, 1.0) * 255.0) as u8);
        }

        match self.scena {
            Scena::Atrio(t) => {
                let a = (t / 0.8).min(1.0);
                for p in fb.pixel_mut() {
                    *p = mix(*p, rgb(0, 0, 0), (a * 255.0) as u8);
                }
                if t > 0.6 {
                    let b = ((t - 0.6) / 0.6).min(1.0);
                    self.font.draw_centered(fb, w / 2, 80, "L'ATRIO. DI NUOVO.", rgb(240, 232, 214), (b * 255.0) as u8);
                    if self.ritorni_all_atrio > 1 {
                        self.font.draw_centered(fb, w / 2, 94, "LA PORTA C'ERA. L'HAI DETTO TU.", rgb(150, 140, 190), (b * 200.0) as u8);
                    }
                }
            }
            Scena::Uscito(t) => {
                let a = (t / 1.5).min(1.0);
                for p in fb.pixel_mut() {
                    *p = mix(*p, rgb(255, 238, 205), (a * 255.0) as u8);
                }
                if t > 1.0 {
                    let b = ((t - 1.0) / 1.0).min(1.0);
                    testo_grande(&self.font, fb, w / 2, 56, "EGRESSO", rgb(60, 44, 40), (b * 255.0) as u8, 2);
                    self.font.draw_centered(fb, w / 2, 84, "NON SERVIVA UN NOME.", rgb(90, 64, 56), (b * 255.0) as u8);
                    if t > 2.0 {
                        self.font.draw_centered(fb, w / 2, 150, "INVIO PER RICOMINCIARE", rgb(150, 120, 100), 200);
                    }
                }
            }
            Scena::Gioco => {}
        }

        if self.debug {
            let mut buf = [0u8; 64];
            let r1 = scrivi(&mut buf, format_args!("FPS {:.0}", stats.fps));
            self.font.draw_text(fb, 2, 2, r1, rgb(120, 255, 120));
            let r2 = scrivi(&mut buf, format_args!("UPD {:.2} REN {:.2}", stats.ms_update, stats.ms_render));
            self.font.draw_text(fb, 2, 11, r2, rgb(120, 255, 120));
            let r3 = scrivi(&mut buf, format_args!("PART {} RIV {}", self.particelle.v.len(), self.mondo.rivelazioni.len()));
            self.font.draw_text(fb, 2, 20, r3, rgb(120, 255, 120));
        }
    }
}

/// Sposta una x perché un testo largo `larg` resti dentro lo schermo.
fn dentro_schermo(x: i32, larg: i32) -> i32 {
    x.clamp(2, LARGHEZZA as i32 - 2 - larg)
}

impl Egresso {
    /// Sopra il protagonista: quali tasti servono adesso, per ciò che è a
    /// fuoco. Ogni suggerimento sparisce dopo che il tasto è stato usato due volte.
    fn disegna_suggerimento(&mut self, fb: &mut Framebuffer) {
        if self.scena != Scena::Gioco || self.t < 2.5 {
            return;
        }
        let Some(i) = self.focus else { return };
        let p = &self.mondo.percetti[i];
        // Nell'Atrio si impara una cosa sola: dire.
        let r_ok = self.stanza > 0 && self.usati[2] < 2;
        let mut parti: [&str; 3] = [""; 3];
        let mut n = 0;
        match p.fase {
            Fase::Proposto => {
                if self.usati[0] < 2 {
                    parti[n] = "E DILLO";
                    n += 1;
                }
                if p.def.proposte.len() > 1 && self.usati[1] < 2 {
                    parti[n] = "Q ALTRA VOCE";
                    n += 1;
                }
                if r_ok {
                    parti[n] = "R LASCIA";
                    n += 1;
                }
            }
            Fase::Affermato if p.def.iniziale.is_none() && r_ok => {
                parti[n] = "R LASCIA ANDARE";
                n += 1;
            }
            _ => {}
        }
        if n == 0 {
            return;
        }
        let mut buf = [0u8; 64];
        let testo = match n {
            1 => scrivi(&mut buf, format_args!("{}", parti[0])),
            2 => scrivi(&mut buf, format_args!("{}  {}", parti[0], parti[1])),
            _ => scrivi(&mut buf, format_args!("{}  {}  {}", parti[0], parti[1], parti[2])),
        };
        let g = self.giocatore.rect();
        let larg = misura(testo);
        let x = dentro_schermo(g.center().x as i32 - larg / 2, larg);
        let y = (g.y as i32 - 12).max(2);
        // Un fondo scuro per leggerlo sopra le lettere luminose.
        fb.fill_rect(x - 2, y - 2, larg + 4, 11, rgb(6, 5, 12));
        self.font.draw_text_alpha(fb, x, y, testo, rgb(214, 206, 240), 230);
    }
}

/// Formatta in un buffer sullo stack: niente `String`, niente allocazioni.
fn scrivi<'a>(buf: &'a mut [u8; 64], args: std::fmt::Arguments) -> &'a str {
    use std::io::Write;
    let mut cur = std::io::Cursor::new(&mut buf[..]);
    let _ = cur.write_fmt(args);
    let n = cur.position() as usize;
    std::str::from_utf8(&buf[..n]).unwrap_or("")
}

/// Testo ingrandito di un fattore intero, centrato su `cx`.
#[allow(clippy::too_many_arguments)]
fn testo_grande(font: &Font, fb: &mut Framebuffer, cx: i32, y: i32, testo: &str, colore: u32, alpha: u8, s: i32) {
    let larg = misura(testo) * s;
    let mut x = cx - larg / 2;
    for c in testo.chars() {
        let m = font.glifo(c);
        for (r, bits) in m.iter().enumerate() {
            for col in 0..5 {
                if bits & (1 << (4 - col)) != 0 {
                    for dy in 0..s {
                        for dx in 0..s {
                            fb.blend_pixel(x + col * s + dx, y + r as i32 * s + dy, colore, alpha);
                        }
                    }
                }
            }
        }
        x += AVANZAMENTO * s;
    }
}

/// Dipinge una volta sola il dungeon vero (pixel art procedurale, già
/// illuminata dalle lanterne). Restituisce l'immagine e la sua parte emissiva.
pub fn genera_realta(mondo: &Mondo) -> (Vec<u32>, Vec<u32>) {
    let (w, h) = (LARGHEZZA as i32, ALTEZZA as i32);
    let mut img = vec![0u32; (w * h) as usize];
    let mut luce = vec![0u32; (w * h) as usize];
    let t = TILE;
    let varco_centro = Vec2::new((COLONNE * t) as f32, (16 * t) as f32);

    for y in 0..h {
        for x in 0..w {
            let (tx, ty) = (x / t, y / t);
            let reale = mondo.reale(tx, ty);
            let sopra_libero = !mondo.reale(tx, ty - 1).regge();
            let n = hash01(x, y, 7) * 0.12 - 0.06;
            let (mut r, mut g, mut b): (f32, f32, f32) = match reale {
                Reale::Vuoto => {
                    // Il muro di fondo: mattoni grandi e scuri.
                    let fila = y / 8;
                    let ox = if fila % 2 == 0 { 0 } else { 8 };
                    let mattone = hash01((x + ox) / 16, fila, 3);
                    let malta = (x + ox) % 16 == 0 || y % 8 == 0;
                    let k = if malta { 0.55 } else { 0.8 + 0.35 * mattone };
                    (34.0 * k, 28.0 * k, 46.0 * k)
                }
                Reale::Roccia => {
                    let fila = y / 4;
                    let ox = if fila % 2 == 0 { 0 } else { 4 };
                    let blocco = hash01((x + ox) / 8, fila, 11);
                    let malta = (x + ox) % 8 == 0 || y % 4 == 0;
                    let mut k = if malta { 0.6 } else { 0.8 + 0.4 * blocco };
                    if sopra_libero && y % t == 0 {
                        k = 1.5;
                    }
                    (70.0 * k, 60.0 * k, 88.0 * k)
                }
                Reale::Gradino => {
                    let malta = x % 16 == 0;
                    let k = if y % t == 0 {
                        1.5
                    } else if malta {
                        0.7
                    } else {
                        1.0
                    };
                    (92.0 * k, 82.0 * k, 104.0 * k)
                }
                Reale::Varco => {
                    let v = (y % (t * 2)) as f32 / (t * 2) as f32;
                    let c = (255.0, 240.0 - 35.0 * v, 200.0 - 50.0 * v);
                    luce[(y * w + x) as usize] = rgb(200, 170, 120);
                    c
                }
            };
            // Muschio sulle superfici libere.
            if matches!(reale, Reale::Roccia | Reale::Gradino) && sopra_libero && y % t < 2 && hash01(x, y, 5) < 0.45 {
                let m = hash01(x, y, 6);
                (r, g, b) = if m < 0.6 { (60.0, 118.0, 62.0) } else { (112.0, 170.0, 84.0) };
            }
            if reale != Reale::Varco {
                // Luce delle lanterne e del varco.
                let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                let mut l = 0.32;
                for &(lx, ly) in mondo.stanza.lanterne {
                    let c = Vec2::new((lx * t + 4) as f32, (ly * t + 3) as f32);
                    l += (1.0 - (p - c).len() / 100.0).max(0.0).powf(1.5) * 1.3;
                }
                let lv = (1.0 - (p - varco_centro).len() / 130.0).max(0.0).powf(1.4) * 1.2;
                r = r * (l * 1.12 + lv * 1.3) + 255.0 * n;
                g = g * (l + lv * 1.1) + 255.0 * n;
                b = b * (l * 0.82 + lv * 0.8) + 255.0 * n;
            }
            img[(y * w + x) as usize] = rgb(r.clamp(0.0, 255.0) as u8, g.clamp(0.0, 255.0) as u8, b.clamp(0.0, 255.0) as u8);
        }
    }

    // Le lanterne: catena e corpo luminoso.
    let mut fb = Framebuffer::new(LARGHEZZA, ALTEZZA);
    fb.pixel_mut().copy_from_slice(&img);
    for &(lx, ly) in mondo.stanza.lanterne {
        let (x, y) = (lx * t + 4, ly * t);
        fb.fill_rect(x, y - 20, 1, 20, rgb(90, 80, 70));
        fb.fill_rect(x - 2, y, 5, 1, rgb(110, 90, 60));
        fb.fill_rect(x - 2, y + 1, 5, 5, rgb(255, 214, 130));
        fb.fill_rect(x - 1, y + 2, 3, 3, rgb(255, 245, 210));
        fb.fill_rect(x - 2, y + 6, 5, 1, rgb(110, 90, 60));
        for yy in y + 1..y + 6 {
            for xx in x - 2..x + 3 {
                luce[(yy * w + xx) as usize] = rgb(255, 190, 100);
            }
        }
    }
    (fb.pixel().to_vec(), luce)
}
