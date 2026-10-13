//! Come si disegna la notte fuori da Egresso.
//!
//! Strati, dal fondo:
//! 1. cielo a gradiente, stelle che tremano, la luna;
//! 2. montagne lontane e colline con i pini (parallasse: scorrono più piano);
//! 3. il piano di gioco: alberi in silhouette, rovine, la facciata di Egresso;
//! 4. il suolo, il protagonista, l'erba davanti ai piedi, foglie e lucciole;
//! 5. glow: luna, lucciole e lettere accese sono disegnate anche nel buffer
//!    `luce`, che viene sfocato e sommato;
//! 6. l'arco: dopo il glow, perché il suo buio non riceve luce;
//! 7. il buio che prende tutto, la voce.
//!
//! Tutto è calcolato da funzioni della x: nessuna immagine, nessun asset.

use super::giocatore::{ALT, LARG};
use super::terreno::{delta, liscio, rumore, Albero, Rovina, NOME, PIAZZA};
use super::{lettere_accese, Egresso, Scena, DURATA_SOGLIA};
use crate::engine::app::Stats;
use crate::engine::font::{misura, AVANZAMENTO};
use crate::engine::framebuffer::{mix, rgb, scala, Framebuffer};
use crate::engine::math::hash01;

const W: i32 = 320;
const H: i32 = 180;

const CIELO_ALTO: u32 = rgb(4, 5, 16);
const CIELO_BASSO: u32 = rgb(30, 34, 70);
const LONTANO: u32 = rgb(14, 17, 38);
const LONTANO_BORDO: u32 = rgb(34, 40, 76);
const MEDIO: u32 = rgb(9, 11, 25);
const SAGOMA: u32 = rgb(5, 6, 14);
const BORDO_LUNARE: u32 = rgb(44, 52, 92);
const PIETRA: u32 = rgb(48, 49, 68);
const PIETRA_LUCE: u32 = rgb(92, 94, 122);
const PIETRA_SCURA: u32 = rgb(24, 24, 38);
const INCISO_ACCESO: u32 = rgb(255, 232, 180);
const LUNA: (i32, i32, i32) = (262, 30, 10);

/// La forma dell'arco, rispetto al suo centro in basso: dentro o fuori?
fn nell_arco(dx: i32, dy: i32) -> bool {
    // Largo 22, alto 36: rettangolo sotto, mezzo cerchio sopra.
    const R: i32 = 11;
    if dx.abs() > R || !(-36..=0).contains(&dy) {
        return false;
    }
    let sopra = dy + 36 - R; // < 0 nella parte curva
    sopra >= 0 || dx * dx + sopra * sopra <= R * R
}

impl Egresso {
    /// La x sullo schermo di un punto del mondo (il più vicino, nel giro).
    fn schermo_x(&self, x: f32) -> i32 {
        let centro = self.cam + W as f32 * 0.5;
        (centro + delta(centro, x) - self.cam).round() as i32
    }

    pub(super) fn disegna(&mut self, fb: &mut Framebuffer, stats: &Stats) {
        self.luce.clear(0);
        if let Scena::Dentro(s) = self.scena {
            self.disegna_dentro(fb, s);
            return;
        }
        self.disegna_cielo(fb);
        self.disegna_lontano(fb);
        self.disegna_alberi(fb);
        self.disegna_rovine(fb);
        self.disegna_facciata(fb);
        self.disegna_suolo(fb);
        self.disegna_giocatore(fb, 255);
        self.disegna_erba(fb);
        self.disegna_foglie(fb);
        self.disegna_lucciole(fb);

        self.glow.apply(&self.luce, fb, 2, 1.1);
        self.glow.apply(&self.luce, fb, 8, 1.5);

        self.disegna_buio_dell_arco(fb);
        self.disegna_soglia(fb);
        // All'inizio la notte si accende piano.
        if self.t < 2.5 {
            let a = (1.0 - self.t / 2.5).clamp(0.0, 1.0);
            for p in fb.pixel_mut() {
                *p = scala(*p, 1.0 - a);
            }
        }
        if self.debug {
            self.disegna_debug(fb, stats);
        }
    }

    fn disegna_cielo(&mut self, fb: &mut Framebuffer) {
        for y in 0..H {
            let c = mix(CIELO_ALTO, CIELO_BASSO, (liscio(y as f32 / 140.0) * 255.0) as u8);
            fb.fill_rect(0, y, W, 1, c);
        }
        // Stelle: scorrono appena, e tremano.
        for i in 0..150 {
            let sx = hash01(i, 1, 77) * W as f32;
            let y = (hash01(i, 2, 77).powf(1.6) * 120.0) as i32;
            let x = (sx - self.cam * 0.02).rem_euclid(W as f32) as i32;
            let b = hash01(i, 3, 77);
            let tremo = 0.6 + 0.4 * (self.t * (1.0 + 3.0 * b) + i as f32).sin();
            let k = (40.0 + 180.0 * b * b) * tremo;
            fb.add_pixel(x, y, rgb((k * 0.9) as u8, (k * 0.92) as u8, k as u8));
        }
        // La luna, con un alone.
        let (lx, ly, r) = LUNA;
        for dy in -r * 4..=r * 4 {
            for dx in -r * 4..=r * 4 {
                let d = ((dx * dx + dy * dy) as f32).sqrt();
                if d <= r as f32 {
                    let cratere = hash01((dx + 20) / 3, (dy + 20) / 3, 5) < 0.18;
                    let ombra = (dx as f32 / r as f32) * -0.15; // illuminata da destra
                    let base = if cratere { 0.82 } else { 1.0 };
                    fb.set_pixel(lx + dx, ly + dy, scala(rgb(236, 234, 214), base + ombra));
                    self.luce.set_pixel(lx + dx, ly + dy, rgb(70, 70, 60));
                } else {
                    let a = (1.0 - (d - r as f32) / (r * 3) as f32).max(0.0);
                    fb.blend_pixel(lx + dx, ly + dy, rgb(120, 130, 170), (a * a * 40.0) as u8);
                }
            }
        }
    }

    /// Le montagne lontane e le colline coi pini, in parallasse.
    fn disegna_lontano(&mut self, fb: &mut Framebuffer) {
        let s = self.terreno.seme;
        for x in 0..W {
            let wx = self.cam * 0.12 + x as f32;
            let h = (92.0 + rumore(wx / 70.0, s ^ 11) * 20.0 + rumore(wx / 23.0, s ^ 12) * 6.0) as i32;
            fb.fill_rect(x, h, 1, H - h, LONTANO);
            fb.set_pixel(x, h, LONTANO_BORDO);
        }
        for x in 0..W {
            let wx = self.cam * 0.35 + x as f32;
            let h = (118.0 + rumore(wx / 50.0, s ^ 21) * 13.0 + rumore(wx / 17.0, s ^ 22) * 4.0) as i32;
            fb.fill_rect(x, h, 1, H - h, MEDIO);
        }
        // Pini sulle colline di mezzo.
        let base = self.cam * 0.35;
        let (c0, c1) = ((base / 22.0).floor() as i32 - 1, ((base + W as f32) / 22.0).floor() as i32 + 1);
        for c in c0..=c1 {
            if hash01(c, 1, s ^ 23) > 0.55 {
                continue;
            }
            let wx = (c as f32 + hash01(c, 2, s ^ 23)) * 22.0;
            let x = (wx - base) as i32;
            let h = (118.0 + rumore(wx / 50.0, s ^ 21) * 13.0 + rumore(wx / 17.0, s ^ 22) * 4.0) as i32;
            let alto = 8 + (hash01(c, 3, s ^ 23) * 12.0) as i32;
            for r in 0..alto {
                let mezza = (r * 3 / 10).max(if r > 1 { 1 } else { 0 });
                fb.fill_rect(x - mezza, h - alto + r + 1, mezza * 2 + 1, 1, MEDIO);
            }
        }
    }

    fn disegna_alberi(&mut self, fb: &mut Framebuffer) {
        let alberi: Vec<(f32, Albero)> = self.terreno.alberi_tra(self.cam, self.cam + W as f32).collect();
        for (giro, a) in alberi {
            match a {
                Albero::Pino { x, h } => {
                    let sx = self.schermo_x(giro + x);
                    let base = self.terreno.suolo(giro + x) as i32 + 1;
                    let h = h as i32;
                    fb.fill_rect(sx - 1, base - 6, 2, 6, SAGOMA);
                    // Tre piani di rami, ognuno più largo di quello sopra.
                    for piano in 0..3 {
                        let cima = base - h + piano * h / 4;
                        let alto = h / 2;
                        let largo = 5 + piano * 2;
                        for r in 0..alto {
                            let mezza = r * largo / alto;
                            fb.fill_rect(sx - mezza, cima + r, mezza * 2 + 1, 1, SAGOMA);
                            // La luna tocca solo le punte dei rami, a destra.
                            if r > alto - 3 {
                                fb.set_pixel(sx + mezza, cima + r, BORDO_LUNARE);
                            }
                        }
                    }
                }
                Albero::Chioma { x, r } => {
                    let sx = self.schermo_x(giro + x);
                    let base = self.terreno.suolo(giro + x) as i32 + 1;
                    let r = r as i32;
                    let cy = base - r * 2;
                    fb.fill_rect(sx - 1, cy, 3, base - cy, SAGOMA);
                    for dy in -r..=r {
                        for dx in -r - 2..=r + 2 {
                            let ang = (dy as f32).atan2(dx as f32);
                            let bordo = r as f32 + (hash01((ang * 6.0) as i32, 0, giro as u32 ^ r as u32) - 0.5) * 3.0;
                            let d = ((dx * dx + dy * dy) as f32).sqrt();
                            if d <= bordo {
                                let luce = d > bordo - 1.2 && dx > 0 && dy < r / 2;
                                fb.set_pixel(sx + dx, cy + dy, if luce { BORDO_LUNARE } else { SAGOMA });
                            }
                        }
                    }
                }
            }
        }
    }

    fn disegna_rovine(&mut self, fb: &mut Framebuffer) {
        let rovine: Vec<(f32, Rovina)> = self.terreno.rovine_tra(self.cam, self.cam + W as f32).collect();
        let chi = self.giocatore.centro_x();
        for (giro, r) in rovine {
            match r {
                Rovina::Colonna { x, h } => {
                    let sx = self.schermo_x(giro + x);
                    let base = self.terreno.suolo(giro + x) as i32 + 1;
                    for dx in 0..6 {
                        let rotta = (hash01(dx, giro as i32 + x as i32, 3) * 4.0) as i32;
                        let cima = base - h as i32 + rotta;
                        let c = match dx {
                            0 => PIETRA_LUCE,
                            2 | 4 => PIETRA_SCURA,
                            _ => PIETRA,
                        };
                        fb.fill_rect(sx - 3 + dx, cima, 1, base - cima, c);
                    }
                    fb.fill_rect(sx - 4, base - 2, 8, 2, PIETRA);
                    self.muschio(fb, sx - 4, base - 3, 8);
                }
                Rovina::Blocco { x, w, h } => {
                    let sx = self.schermo_x(giro + x);
                    let base = self.terreno.suolo(giro + x) as i32 + 1;
                    let (w, h) = (w as i32, h as i32);
                    fb.fill_rect(sx - w / 2, base - h, w, h, PIETRA);
                    fb.fill_rect(sx - w / 2, base - h, w, 1, PIETRA_LUCE);
                    fb.fill_rect(sx + w / 2 - 1, base - h, 1, h, PIETRA_SCURA);
                    self.muschio(fb, sx - w / 2, base - h - 1, w);
                }
                Rovina::Lapide { x, lettera } => {
                    let sx = self.schermo_x(giro + x);
                    let base = self.terreno.suolo(giro + x) as i32 + 1;
                    let (x0, y0) = (sx - 5, base - 12);
                    for dy in 0..12 {
                        for dx in 0..10 {
                            let angolo = dy < 2 && (dx < 2 - dy || dx > 7 + dy);
                            if !angolo {
                                let c = if dx == 0 || dy == 0 { PIETRA_LUCE } else { PIETRA };
                                fb.set_pixel(x0 + dx, y0 + dy, c);
                            }
                        }
                    }
                    // La lettera incisa: da vicino si accende appena.
                    let vicino = (1.0 - (delta(chi, giro + x)).abs() / 40.0).clamp(0.0, 1.0);
                    let col = mix(PIETRA_SCURA, INCISO_ACCESO, (vicino * 200.0) as u8);
                    self.font.draw_char(fb, x0 + 2, y0 + 3, lettera, col, 255, false);
                    if vicino > 0.0 {
                        self.font.draw_char(&mut self.luce, x0 + 2, y0 + 3, lettera, scala(rgb(160, 130, 80), vicino), 255, false);
                    }
                }
            }
        }
    }

    fn muschio(&self, fb: &mut Framebuffer, x: i32, y: i32, w: i32) {
        for dx in 0..w {
            if hash01(x + dx + self.cam as i32, y, 9) < 0.5 {
                fb.set_pixel(x + dx, y + (hash01(dx, y, 10) * 2.0) as i32, rgb(56, 96, 70));
            }
        }
    }

    /// La facciata di Egresso, nella collina, dietro la piazza.
    fn disegna_facciata(&mut self, fb: &mut Framebuffer) {
        let a = self.schermo_x(self.terreno.arco);
        if !(-120..W + 120).contains(&a) {
            return;
        }
        let p = self.terreno.piazza_y as i32;
        // La collina in cui è scavata: un dorso scuro dietro la facciata.
        let lato = self.terreno.lato as i32;
        for dx in -150..=150 {
            let verso_collina = dx * lato;
            let k = dx as f32 / 150.0;
            let mut cima = p - 100 + (k * k * 70.0) as i32;
            if verso_collina > 0 {
                // Dalla parte della collina si fonde con la sua cima.
                cima = cima.min(p - 50 - (verso_collina as f32 * 0.1) as i32).max(p - 100);
            }
            let x = a + dx;
            fb.fill_rect(x, cima, 1, p - cima, rgb(11, 14, 24));
            fb.set_pixel(x, cima, rgb(52, 88, 76));
            if hash01(x + self.cam as i32, 0, 61) < 0.4 {
                fb.set_pixel(x, cima - 1, rgb(40, 72, 62));
            }
        }
        // Il muro: blocchi sfalsati, illuminati dall'alto.
        for y in p - 66..p {
            let fila = (p - y) / 6;
            let ox = if fila % 2 == 0 { 0 } else { 6 };
            for x in a - 74..a + 74 {
                let gx = x - a + 74 + ox;
                let giunto = gx % 12 == 0 || (p - y) % 6 == 0;
                let alto = (p - y) % 6 == 5;
                let crepa = hash01(gx / 12, fila, 31) < 0.08 && (gx % 12) == 6;
                let c = if giunto || crepa {
                    PIETRA_SCURA
                } else if alto {
                    PIETRA_LUCE
                } else {
                    scala(PIETRA, 0.85 + 0.25 * hash01(gx / 12, fila, 30))
                };
                fb.set_pixel(x, y, c);
            }
        }
        // Cornice e frontone.
        fb.fill_rect(a - 80, p - 68, 160, 2, PIETRA_LUCE);
        for r in 0..16 {
            let mezza = 80 - r * 5;
            let y = p - 69 - r;
            fb.fill_rect(a - mezza, y, mezza * 2, 1, PIETRA);
            fb.set_pixel(a - mezza, y, PIETRA_LUCE);
            fb.set_pixel(a + mezza - 1, y, PIETRA_LUCE);
        }
        // Due pilastri ai lati.
        for lato in [-1, 1] {
            let x = a + lato * 52;
            fb.fill_rect(x - 4, p - 58, 8, 58, scala(PIETRA, 1.15));
            fb.fill_rect(x - 4, p - 58, 1, 58, PIETRA_LUCE);
            fb.fill_rect(x + 3, p - 58, 1, 58, PIETRA_SCURA);
            fb.fill_rect(x - 6, p - 61, 12, 3, PIETRA_LUCE);
            fb.fill_rect(x - 6, p - 3, 12, 3, PIETRA);
        }
        // La cornice dell'arco: conci più chiari.
        for dy in -40..=0 {
            for dx in -15..=15 {
                if !nell_arco(dx, dy) && (nell_arco(dx + 3, dy) || nell_arco(dx - 3, dy) || nell_arco(dx, dy + 3)) {
                    let concio = ((dx + 20) / 4 + (dy + 50) / 5) % 2 == 0;
                    fb.set_pixel(a + dx, p + dy, if concio { PIETRA_LUCE } else { scala(PIETRA_LUCE, 0.8) });
                }
            }
        }
        // Il nome, inciso. Le lettere si accendono avvicinandosi, e restano accese.
        let accese = lettere_accese(self.lettere);
        let larg = misura(NOME) * 2;
        let mut x = a - larg / 2;
        for (i, c) in NOME.chars().enumerate() {
            let glifo = self.font.glifo(c);
            let acceso = i < accese;
            let tremo = 0.85 + 0.15 * (self.t * 3.0 + i as f32).sin();
            for (r, bits) in glifo.iter().enumerate() {
                for col in 0..5 {
                    if bits & (1 << (4 - col)) == 0 {
                        continue;
                    }
                    for (ddx, ddy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                        let (px, py) = (x + col * 2 + ddx, p - 62 + r as i32 * 2 + ddy);
                        if acceso {
                            fb.set_pixel(px, py, scala(INCISO_ACCESO, tremo));
                            self.luce.set_pixel(px, py, scala(rgb(200, 150, 80), tremo));
                        } else {
                            fb.set_pixel(px, py, PIETRA_SCURA);
                            fb.set_pixel(px, py + 1, scala(PIETRA_LUCE, 0.8));
                        }
                    }
                }
            }
            x += AVANZAMENTO * 2;
        }
        // Edera che pende dalla cornice.
        for k in 0..9 {
            let hx = hash01(k, 1, self.terreno.seme ^ 41);
            let x = a - 76 + (hx * 152.0) as i32;
            let lungo = 4 + (hash01(k, 2, self.terreno.seme ^ 41) * 18.0) as i32;
            for y in 0..lungo {
                let ondeggia = ((self.t * 1.5 + k as f32 + y as f32 * 0.3).sin() * (y as f32 / 12.0)).round() as i32;
                fb.set_pixel(x + ondeggia, p - 66 + y, rgb(40, 82, 58));
                if y % 3 == 1 {
                    fb.set_pixel(x + ondeggia + 1, p - 66 + y, rgb(70, 120, 80));
                }
            }
        }
    }

    fn disegna_suolo(&mut self, fb: &mut Framebuffer) {
        let arco = self.terreno.arco;
        for x in 0..W {
            let wx = self.cam + x as f32;
            let g = self.terreno.suolo(wx) as i32;
            let piazza = delta(arco, wx).abs() <= PIAZZA;
            let gx = wx.floor() as i32;
            for y in g.max(0)..H {
                let prof = y - g;
                let c = if piazza && prof < 4 {
                    // Lastre di pietra.
                    let giunto = (gx.rem_euclid(14)) == 0;
                    if prof == 0 {
                        PIETRA_LUCE
                    } else if giunto {
                        PIETRA_SCURA
                    } else {
                        PIETRA
                    }
                } else if prof == 0 {
                    rgb(70, 112, 96)
                } else if prof < 3 {
                    rgb(34, 62, 56)
                } else {
                    let sasso = hash01(gx, y, 13) < 0.06;
                    let base = if sasso { rgb(30, 32, 48) } else { rgb(15, 18, 30) };
                    scala(base, 1.0 - (prof as f32 / 120.0).min(0.6))
                };
                fb.set_pixel(x, y, c);
            }
            // Vicino a dove il suolo cade a picco (il fronte della collina): roccia.
            for dk in -3i32..=3 {
                let prima = self.terreno.suolo(wx + dk as f32 - 1.0) as i32;
                let dopo = self.terreno.suolo(wx + dk as f32) as i32;
                if (prima - dopo).abs() > 6 {
                    let alto = prima.min(dopo);
                    let basso = prima.max(dopo);
                    for y in alto.max(g)..basso {
                        let c = if hash01(gx / 2, y / 3, 17) < 0.5 { PIETRA } else { PIETRA_SCURA };
                        fb.set_pixel(x, y, scala(c, 0.8 - dk.abs() as f32 * 0.1));
                    }
                }
            }
        }
    }

    /// L'erba davanti ai piedi: si piega col vento, e si scosta al passaggio.
    fn disegna_erba(&mut self, fb: &mut Framebuffer) {
        let arco = self.terreno.arco;
        let chi = self.giocatore.centro_x();
        let piedi = self.giocatore.pos.y + ALT;
        for x in (0..W).step_by(2) {
            let wx = (self.cam + x as f32).floor();
            if delta(arco, wx).abs() <= PIAZZA {
                continue;
            }
            let gx = wx as i32;
            let g = self.terreno.suolo(wx);
            let alto = 2.0 + hash01(gx, 0, 51) * 5.0;
            let vento = self.terreno.vento(wx, self.t);
            let mut piega = vento * 2.4 + (self.t * 2.3 + wx * 0.11).sin() * (0.3 + vento.abs() * 0.6);
            let dx = wx - chi;
            if dx.abs() < 8.0 && (piedi - g).abs() < 6.0 {
                piega += dx.signum() * (8.0 - dx.abs()) / 8.0 * 3.0;
            }
            let n = alto as i32;
            for r in 0..n {
                let k = r as f32 / alto;
                let ox = (piega * k * k).round() as i32;
                let c = mix(rgb(30, 58, 52), rgb(96, 150, 124), (k * 255.0) as u8);
                fb.set_pixel(x + ox, g as i32 - r, c);
            }
        }
    }

    fn disegna_foglie(&mut self, fb: &mut Framebuffer) {
        for f in &self.foglie.v {
            let a = ((f.vita / f.durata) * 4.0).min(1.0);
            let x = self.schermo_x(f.pos.x);
            let y = f.pos.y as i32;
            fb.blend_pixel(x, y, f.colore, (a * 255.0) as u8);
            if (self.t * 8.0 + f.durata * 13.0).sin() > 0.0 {
                fb.blend_pixel(x + 1, y, f.colore, (a * 160.0) as u8);
            }
        }
    }

    fn disegna_lucciole(&mut self, fb: &mut Framebuffer) {
        for l in &self.lucciole.v {
            if l.luce <= 0.01 {
                continue;
            }
            let b = l.luce * (0.5 + 0.5 * (self.t * 2.6 + l.fase * 3.0).sin().max(-0.4));
            if b <= 0.0 {
                continue;
            }
            let x = self.schermo_x(l.pos.x);
            let y = l.pos.y as i32;
            let c = scala(rgb(226, 255, 150), b);
            fb.add_pixel(x, y, c);
            self.luce.fill_rect(x, y, 2, 2, scala(rgb(200, 255, 120), b));
        }
    }

    fn disegna_giocatore(&mut self, fb: &mut Framebuffer, alpha: u8) {
        let g = &self.giocatore;
        let x = self.schermo_x(g.pos.x + LARG * 0.5) - (LARG * 0.5) as i32;
        let y = g.pos.y.round() as i32;
        let corpo = rgb(232, 230, 240);
        let ombra = rgb(150, 148, 170);
        let mut r = |dx: i32, dy: i32, w: i32, h: i32, c: u32| {
            for yy in 0..h {
                for xx in 0..w {
                    fb.blend_pixel(x + dx + xx, y + dy + yy, c, alpha);
                }
            }
        };
        r(1, 0, 4, 4, corpo);
        r(if g.verso > 0.0 { 3 } else { 2 }, 1, 1, 1, rgb(30, 26, 40));
        r(0, 4, 6, 5, corpo);
        r(if g.verso > 0.0 { 0 } else { 4 }, 5, 2, 3, ombra);
        let fase = ((g.passo / 4.0) as i32) & 1;
        if !g.a_terra {
            r(1, 9, 2, 2, corpo);
            r(3, 9, 2, 3, corpo);
        } else if g.vel.x.abs() > 5.0 && fase == 1 {
            r(0, 9, 2, 3, corpo);
            r(4, 9, 2, 2, corpo);
        } else {
            r(1, 9, 2, 3, corpo);
            r(3, 9, 2, 3, corpo);
        }
        // La sciarpa rossa, mossa dal vento.
        let rosso = rgb(220, 40, 52);
        r(0, 4, 6, 1, rosso);
        let vento = self.terreno.vento(g.centro_x(), self.t);
        let verso = if vento.abs() > 0.2 { vento.signum() } else { -g.verso };
        let lungo = 2 + (vento.abs() * 3.0) as i32;
        for k in 1..=lungo {
            let sv = ((self.t * 9.0 + k as f32).sin() * 0.8).round() as i32;
            let bx = if verso > 0.0 { 5 + k } else { -k };
            r(bx, 4 + sv + k / 3, 1, 1, rosso);
        }
    }

    /// Dopo il glow: l'arco resta nero. Chi ci passa davanti resta davanti.
    fn disegna_buio_dell_arco(&mut self, fb: &mut Framebuffer) {
        let a = self.schermo_x(self.terreno.arco);
        if !(-20..W + 20).contains(&a) {
            return;
        }
        let p = self.terreno.piazza_y as i32;
        for dy in -36..0 {
            for dx in -11..=11 {
                if nell_arco(dx, dy) {
                    // Un respiro di buio: appena più chiaro ai bordi.
                    let bordo = (11 - dx.abs()).min(dy + 36);
                    let k = if bordo < 2 { 0.25 } else { 0.0 };
                    if let Some(c) = fb.get_pixel(a + dx, p + dy) {
                        fb.set_pixel(a + dx, p + dy, scala(c, k * 0.4));
                    }
                }
            }
        }
        // Il protagonista davanti al buio (mentre entra, svanisce).
        let alpha = match self.scena {
            Scena::Soglia(s) => ((1.0 - (s - 0.6) / 1.2).clamp(0.0, 1.0) * 255.0) as u8,
            _ => 255,
        };
        if delta(self.giocatore.centro_x(), self.terreno.arco).abs() < 18.0 {
            self.disegna_giocatore(fb, alpha);
            self.disegna_erba(fb);
        }
    }

    /// Il buio dell'arco si allarga e prende tutto.
    fn disegna_soglia(&mut self, fb: &mut Framebuffer) {
        let Scena::Soglia(s) = self.scena else { return };
        let a = self.schermo_x(self.terreno.arco);
        let cy = self.terreno.piazza_y as i32 - 16;
        let k = (s / DURATA_SOGLIA).clamp(0.0, 1.0);
        let raggio = 360.0 * (1.0 - k).powf(1.6);
        for y in 0..H {
            for x in 0..W {
                let d = (((x - a) * (x - a) + (y - cy) * (y - cy)) as f32).sqrt();
                let buio = ((d - raggio) / 40.0 + k * 0.3).clamp(0.0, 1.0);
                if buio > 0.0 {
                    let i = (y * W + x) as usize;
                    fb.pixel_mut()[i] = scala(fb.pixel_mut()[i], 1.0 - buio);
                }
            }
        }
    }

    fn disegna_dentro(&mut self, fb: &mut Framebuffer, s: f32) {
        fb.clear(rgb(0, 0, 0));
        // La voce: lettera per lettera, lenta.
        const TESTO: &str = "BENTORNATO.";
        const PASSO: i32 = 7;
        if s > 1.5 {
            let n = (((s - 1.5) * 7.0) as usize).min(TESTO.len());
            let larg = TESTO.len() as i32 * PASSO - 2;
            let x0 = (W - larg) / 2;
            let colore = rgb(226, 220, 255);
            for (i, c) in TESTO.chars().take(n).enumerate() {
                let onda = ((self.t * 1.6 + i as f32 * 0.5).sin() * 1.2).round() as i32;
                let (x, y) = (x0 + i as i32 * PASSO, 84 + onda);
                self.font.draw_char(fb, x, y, c, colore, 255, false);
                self.font.draw_char(&mut self.luce, x, y, c, scala(colore, 0.5), 255, false);
            }
        }
        self.glow.apply(&self.luce, fb, 2, 1.2);
        self.glow.apply(&self.luce, fb, 9, 1.6);
        if s > 6.0 {
            let a = ((s - 6.0) / 1.5).min(1.0);
            self.font.draw_centered(fb, W / 2, 160, "SALTA PER UN'ALTRA NOTTE", rgb(90, 86, 116), (a * 200.0) as u8);
        }
    }

    fn disegna_debug(&mut self, fb: &mut Framebuffer, stats: &Stats) {
        let mut buf = [0u8; 64];
        let r1 = scrivi(&mut buf, format_args!("FPS {:.0} UPD {:.2} REN {:.2}", stats.fps, stats.ms_update, stats.ms_render));
        self.font.draw_text(fb, 2, 2, r1, rgb(120, 255, 120));
        let d = self.all_arco();
        let v = self.terreno.vicinanza(self.giocatore.centro_x());
        let r2 = scrivi(&mut buf, format_args!("ARCO {d:.0} VICINO {v:.2} SEME {}", self.terreno.seme));
        self.font.draw_text(fb, 2, 11, r2, rgb(120, 255, 120));
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
