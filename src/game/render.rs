//! Come si disegna EGRESSO.
//!
//! La stanza (roccia, muri, lanterne, l'arco) viene dipinta **una volta** in
//! un'immagine larga quanto il mondo, già illuminata dalle luci fisse; si
//! ridipinge solo quando la stanza cambia forma. Ogni frame:
//! 1. si copia la finestra della telecamera;
//! 2. la lanterna del protagonista schiarisce intorno a lui (brucia più forte
//!    quando forza);
//! 3. acqua e correnti, visibili come scie che scorrono;
//! 4. le scritte sui muri, leggibili in proporzione alla luce;
//! 5. particelle, protagonista, la voce della Sala;
//! 6. glow: ciò che emette luce è disegnato anche nel buffer `luce`;
//! 7. interfaccia.

use super::mondo::{self, Mondo};
use super::stanze::{Fluido, Inchiostro, Scritta, RIGHE, TILE};
use super::{Egresso, Scena};
use crate::engine::app::{Stats, LARGHEZZA};
use crate::engine::font::{misura, Font, AVANZAMENTO};
use crate::engine::framebuffer::{mix, rgb, scala, Framebuffer};
use crate::engine::math::{hash01, Rect, Vec2};

const ALTEZZA_MONDO: i32 = RIGHE * TILE;
const INTERLINEA: i32 = 9;
const RAGGIO_LANTERNA: f32 = 100.0;

/// Distanza di un punto da un rettangolo (0 se è dentro).
pub fn distanza_rect(p: Vec2, r: &Rect) -> f32 {
    let dx = (r.x - p.x).max(p.x - r.right()).max(0.0);
    let dy = (r.y - p.y).max(p.y - r.bottom()).max(0.0);
    (dx * dx + dy * dy).sqrt()
}

/// Dove sta una scritta nel mondo, in pixel.
pub fn rect_scritta(m: &Mondo, s: &Scritta) -> Rect {
    let larg = s.righe.iter().map(|r| misura(r)).max().unwrap_or(0);
    let x = m.x_mondo(s.x) * TILE;
    Rect::new(x as f32, (s.y * TILE) as f32, larg as f32, (s.righe.len() as i32 * INTERLINEA) as f32)
}

/// Le lanterne della stanza, in pixel del mondo.
fn lanterne(m: &Mondo) -> impl Iterator<Item = Vec2> + '_ {
    m.stanza.lanterne.iter().map(|&(x, y)| Vec2::new((m.x_mondo(x) * TILE + 4) as f32, (y * TILE + 3) as f32))
}

/// Il centro dell'arco verso fuori, se la stanza ce l'ha.
fn arco(m: &Mondo) -> Option<Vec2> {
    let righe: Vec<i32> = (0..RIGHE).filter(|&y| m.cella(0, y) == b'O').collect();
    let (a, b) = (*righe.first()?, *righe.last()?);
    Some(Vec2::new(4.0, ((a + b + 1) * TILE) as f32 * 0.5))
}

fn luce_lanterna(p: Vec2, c: Vec2) -> f32 {
    (1.0 - (p - c).len() / RAGGIO_LANTERNA).max(0.0).powf(1.5) * 1.3
}

/// Dipinge la stanza intera: l'immagine e la sua parte emissiva.
pub fn dipingi(m: &Mondo) -> (Vec<u32>, Vec<u32>) {
    let w = m.larg * TILE;
    let h = ALTEZZA_MONDO;
    let mut img = vec![0u32; (w * h) as usize];
    let mut luce = vec![0u32; (w * h) as usize];
    let t = TILE;
    let luci: Vec<Vec2> = lanterne(m).collect();
    let arco = arco(m);

    for y in 0..h {
        for x in 0..w {
            let (tx, ty) = (x / t, y / t);
            let cella = m.cella(tx, ty);
            let sopra_libero = !m.solido(tx, ty - 1);
            let n = hash01(x, y, 7) * 0.06 - 0.03;
            // Le texture usano la colonna della mappa base: le copie del modulo si somigliano.
            let bx = m.x_base(tx) * t + x % t;
            let (mut r, mut g, mut b): (f32, f32, f32) = match cella {
                b'#' => {
                    let fila = y / 4;
                    let ox = if fila % 2 == 0 { 0 } else { 4 };
                    let blocco = hash01((bx + ox) / 8, fila, 11);
                    let malta = (bx + ox) % 8 == 0 || y % 4 == 0;
                    let mut k = if malta { 0.6 } else { 0.8 + 0.4 * blocco };
                    if sopra_libero && y % t == 0 {
                        k = 1.5;
                    }
                    (92.0 * k, 80.0 * k, 104.0 * k)
                }
                b'O' => {
                    let v = (y % (t * 2)) as f32 / (t * 2) as f32;
                    luce[(y * w + x) as usize] = rgb(200, 170, 120);
                    (255.0, 240.0 - 35.0 * v, 200.0 - 50.0 * v)
                }
                _ => {
                    // Il muro di fondo: mattoni grandi e scuri.
                    let fila = y / 8;
                    let ox = if fila % 2 == 0 { 0 } else { 8 };
                    let mattone = hash01((bx + ox) / 16, fila, 3);
                    let malta = (bx + ox) % 16 == 0 || y % 8 == 0;
                    let k = if malta { 0.55 } else { 0.8 + 0.35 * mattone };
                    (24.0 * k, 19.0 * k, 34.0 * k)
                }
            };
            // Muschio sulle superfici libere.
            if cella == b'#' && sopra_libero && y % t < 2 && hash01(x, y, 5) < 0.45 {
                let mm = hash01(x, y, 6);
                (r, g, b) = if mm < 0.6 { (60.0, 118.0, 62.0) } else { (112.0, 170.0, 84.0) };
            }
            if cella != b'O' {
                let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                let l = 0.3 + luci.iter().map(|&c| luce_lanterna(p, c)).sum::<f32>();
                // La luce di fuori, che entra dall'arco.
                let lv = arco.map_or(0.0, |a| (1.0 - (p - a).len() / 150.0).max(0.0).powf(1.4) * 1.4);
                r = r * (l * 1.12 + lv * 1.3) + 255.0 * n;
                g = g * (l + lv * 1.1) + 255.0 * n;
                b = b * (l * 0.82 + lv * 0.8) + 255.0 * n;
            }
            img[(y * w + x) as usize] = rgb(r.clamp(0.0, 255.0) as u8, g.clamp(0.0, 255.0) as u8, b.clamp(0.0, 255.0) as u8);
        }
    }

    // Le lanterne: catena e corpo luminoso.
    for c in luci {
        let (x, y) = (c.x as i32, c.y as i32 - 3);
        let mut px = |xx: i32, yy: i32, col: u32, l: bool| {
            if (0..w).contains(&xx) && (0..h).contains(&yy) {
                img[(yy * w + xx) as usize] = col;
                if l {
                    luce[(yy * w + xx) as usize] = rgb(255, 190, 100);
                }
            }
        };
        for yy in y - 20..y {
            px(x, yy, rgb(90, 80, 70), false);
        }
        for xx in x - 2..x + 3 {
            px(xx, y, rgb(110, 90, 60), false);
            px(xx, y + 6, rgb(110, 90, 60), false);
            for yy in y + 1..y + 6 {
                let centro = (xx - x).abs() <= 1 && (2..5).contains(&(yy - y));
                px(xx, yy, if centro { rgb(255, 245, 210) } else { rgb(255, 214, 130) }, true);
            }
        }
    }
    (img, luce)
}

impl Egresso {
    pub(super) fn disegna(&mut self, fb: &mut Framebuffer, stats: &Stats) {
        fb.clear(rgb(0, 0, 0));
        self.luce.clear(0);
        let scossa = if self.scossa > 0.0 { (((self.t * 50.0) as i32 % 3) - 1) * 2 } else { 0 };
        let cam = self.cam.round() as i32 + scossa;

        self.disegna_stanza(fb, cam);
        self.disegna_lanterna(fb, cam);
        self.disegna_acqua(fb, cam);
        self.disegna_correnti(fb, cam);
        self.disegna_scritte(fb, cam);
        self.disegna_particelle(fb, cam);
        self.disegna_giocatore(fb, cam);
        self.disegna_sala(fb);

        self.glow.apply(&self.luce, fb, 2, 1.2);
        self.glow.apply(&self.luce, fb, 9, 1.6);

        self.disegna_interfaccia(fb, stats);
    }

    fn disegna_stanza(&mut self, fb: &mut Framebuffer, cam: i32) {
        let lw = LARGHEZZA as i32;
        let w = self.mondo.larg * TILE;
        for y in 0..ALTEZZA_MONDO {
            for x in 0..lw {
                let wx = cam + x;
                if !(0..w).contains(&wx) {
                    continue;
                }
                let i = (y * w + wx) as usize;
                let o = (y * lw + x) as usize;
                fb.pixel_mut()[o] = self.sfondo[i];
                let l = self.sfondo_luce[i];
                if l != 0 {
                    self.luce.pixel_mut()[o] = l;
                }
            }
        }
    }

    fn centro_lanterna(&self) -> Vec2 {
        let g = &self.giocatore;
        Vec2::new(g.pos.x + if g.verso > 0.0 { 7.0 } else { -1.0 }, g.pos.y + 7.0)
    }

    fn raggio_lanterna(&self) -> f32 {
        60.0 + self.foga * 26.0 + (self.t * 9.0).sin() * 1.5
    }

    /// Luce della sola lanterna del protagonista in un punto (0..~1.6).
    fn luce_giocatore(&self, p: Vec2) -> f32 {
        let r = self.raggio_lanterna();
        (1.0 - (p - self.centro_lanterna()).len() / r).max(0.0).powf(1.2) * (1.0 + self.foga * 0.6)
    }

    /// Luce totale in un punto: lanterne fisse e la tua.
    fn lume(&self, p: Vec2) -> f32 {
        lanterne(&self.mondo).map(|c| luce_lanterna(p, c)).sum::<f32>() + self.luce_giocatore(p)
    }

    fn disegna_lanterna(&mut self, fb: &mut Framebuffer, cam: i32) {
        let c = self.centro_lanterna();
        let r = self.raggio_lanterna();
        let (cx, cy) = (c.x as i32 - cam, c.y as i32);
        let ri = r as i32 + 1;
        for y in (cy - ri).max(0)..(cy + ri).min(ALTEZZA_MONDO) {
            for x in (cx - ri).max(0)..(cx + ri).min(LARGHEZZA as i32) {
                let k = self.luce_giocatore(Vec2::new((x + cam) as f32, y as f32));
                if k <= 0.0 {
                    continue;
                }
                let i = (y * LARGHEZZA as i32 + x) as usize;
                let p = fb.pixel_mut()[i];
                let caldo = scala(p, 1.0 + 1.7 * k);
                fb.pixel_mut()[i] = crate::engine::framebuffer::add(caldo, rgb((k * 22.0) as u8, (k * 12.0) as u8, 0));
            }
        }
    }

    fn disegna_acqua(&mut self, fb: &mut Framebuffer, cam: i32) {
        let tx0 = cam.div_euclid(TILE);
        let tx1 = (cam + LARGHEZZA as i32).div_euclid(TILE);
        for ty in 0..RIGHE {
            for tx in tx0..=tx1 {
                if self.mondo.cella(tx, ty) != b'~' {
                    continue;
                }
                let cima = !self.mondo.acqua(tx, ty - 1);
                let mut prof = 0;
                while self.mondo.acqua(tx, ty - prof - 1) {
                    prof += 1;
                }
                let alpha = (120 + prof * 14).min(210) as u8;
                for py in 0..TILE {
                    for px in 0..TILE {
                        let wx = tx * TILE + px;
                        let (x, y) = (wx - cam, ty * TILE + py);
                        if cima {
                            let onda = (self.t * 2.2 + wx as f32 * 0.17).sin() * 1.2 + (self.t * 1.3 + wx as f32 * 0.05).sin() * 0.8;
                            let pelo = 1.5 + onda;
                            if (py as f32) < pelo {
                                continue;
                            }
                            if (py as f32) < pelo + 1.0 {
                                fb.blend_pixel(x, y, rgb(150, 200, 235), 170);
                                self.luce.blend_pixel(x, y, rgb(20, 40, 60), 255);
                                continue;
                            }
                        }
                        fb.blend_pixel(x, y, rgb(10, 30, 60), alpha);
                    }
                }
            }
        }
    }

    /// Le correnti si vedono: scie che scorrono nella loro direzione.
    fn disegna_correnti(&mut self, fb: &mut Framebuffer, cam: i32) {
        for d in self.mondo.stanza.correnti {
            let forza = if mondo::attiva(d, self.t) { 1.0 } else { mondo::preavviso(d, self.t) * 0.5 };
            if forza <= 0.0 {
                continue;
            }
            let a = self.mondo.area(d);
            let (vx, vy) = d.vel;
            let (col, alpha) = match d.fluido {
                Fluido::Acqua => (rgb(120, 180, 225), 110.0),
                Fluido::Aria => (rgb(215, 220, 240), 160.0),
            };
            let orizzontale = vx != 0.0;
            // Una corsia ogni riga (o colonna) di tile, qualche scia per corsia.
            let (corsie, lungo) = if orizzontale { (a.h as i32 / TILE, a.w) } else { (a.w as i32 / TILE, a.h) };
            let v = if orizzontale { vx } else { vy };
            let per_corsia = (lungo / 24.0).ceil() as i32;
            for c in 0..corsie {
                for k in 0..per_corsia {
                    let h = hash01(k, c, 41);
                    let vel = v * (0.6 + 0.6 * hash01(k, c, 42));
                    let s = (h * lungo + self.t * vel).rem_euclid(lungo);
                    let trasv = (c * TILE) as f32 + hash01(k, c, 43) * 7.0;
                    let p = if orizzontale { Vec2::new(a.x + s, a.y + trasv) } else { Vec2::new(a.x + trasv, a.y + s) };
                    let (tx, ty) = Mondo::tile(p);
                    if self.mondo.riparato(tx, ty, d) || self.mondo.solido(tx, ty) {
                        continue;
                    }
                    let dir = if v > 0.0 { 1.0 } else { -1.0 };
                    const SCIA: i32 = 7;
                    for q in 0..SCIA {
                        let dietro = q as f32 * dir;
                        let pp = if orizzontale { Vec2::new(p.x - dietro, p.y) } else { Vec2::new(p.x, p.y - dietro) };
                        let al = (alpha * forza * (1.0 - q as f32 / SCIA as f32)) as u8;
                        fb.blend_pixel(pp.x as i32 - cam, pp.y as i32, col, al);
                    }
                }
            }
        }
    }

    fn disegna_scritte(&mut self, fb: &mut Framebuffer, cam: i32) {
        for s in self.mondo.stanza.scritte {
            let r = rect_scritta(&self.mondo, s);
            let c = r.center();
            if r.right() < cam as f32 || r.x > (cam + LARGHEZZA as i32) as f32 {
                continue;
            }
            let (x, y) = (r.x as i32 - cam, r.y as i32);
            match s.inchiostro {
                Inchiostro::Ottone => {
                    // Una targa: si legge con qualunque luce.
                    let l = self.lume(c).min(1.0);
                    let lastra = scala(rgb(150, 112, 48), 0.25 + 0.75 * l);
                    fb.fill_rect(x - 2, y - 2, r.w as i32 + 4, r.h as i32 + 2, lastra);
                    let bordo = scala(rgb(214, 170, 90), 0.25 + 0.75 * l);
                    for dx in -2..r.w as i32 + 2 {
                        fb.set_pixel(x + dx, y - 2, bordo);
                        fb.set_pixel(x + dx, y + r.h as i32 - 1, scala(bordo, 0.5));
                    }
                    for (i, riga) in s.righe.iter().enumerate() {
                        self.font.draw_text(fb, x, y + i as i32 * INTERLINEA, riga, scala(rgb(52, 34, 14), 0.4 + 0.6 * l));
                    }
                }
                Inchiostro::Gesso => {
                    let a = (self.lume(c) * 1.3).clamp(0.0, 1.0);
                    for (i, riga) in s.righe.iter().enumerate() {
                        self.font.draw_text_alpha(fb, x, y + i as i32 * INTERLINEA, riga, rgb(228, 222, 206), (a * 210.0) as u8);
                    }
                }
                Inchiostro::Carbone => {
                    // Il carbone si legge solo con la tua lanterna, da vicino.
                    let a = (self.luce_giocatore(c) * 2.2).clamp(0.0, 1.0);
                    for (i, riga) in s.righe.iter().enumerate() {
                        self.font.draw_text_alpha(fb, x, y + i as i32 * INTERLINEA, riga, rgb(160, 156, 166), (a * 230.0) as u8);
                    }
                }
            }
        }
    }

    fn disegna_particelle(&mut self, fb: &mut Framebuffer, cam: i32) {
        for p in &self.particelle.v {
            let k = (p.vita / p.durata).clamp(0.0, 1.0);
            let (x, y) = (p.pos.x as i32 - cam, p.pos.y as i32);
            let c = scala(p.colore, k);
            fb.add_pixel(x, y, c);
            self.luce.add_pixel(x, y, scala(c, 0.6));
        }
    }

    fn disegna_giocatore(&mut self, fb: &mut Framebuffer, cam: i32) {
        let g = &self.giocatore;
        let (x, y) = (g.pos.x.round() as i32 - cam, g.pos.y.round() as i32);
        let corpo = rgb(232, 230, 240);
        let ombra = rgb(150, 148, 170);
        // Testa, con un occhio.
        fb.fill_rect(x + 1, y, 4, 4, corpo);
        fb.set_pixel(x + if g.verso > 0.0 { 3 } else { 2 }, y + 1, rgb(30, 26, 40));
        // Busto.
        fb.fill_rect(x, y + 4, 6, 5, corpo);
        fb.fill_rect(x + if g.verso > 0.0 { 0 } else { 4 }, y + 5, 2, 3, ombra);
        // Gambe: alternate camminando, raccolte in aria, molli in acqua.
        let fase = ((g.passo / 4.0) as i32) & 1;
        if g.in_acqua {
            let f = ((self.t * if g.rilassato { 2.0 } else { 9.0 }) as i32) & 1;
            fb.fill_rect(x + f, y + 9, 2, 3, corpo);
            fb.fill_rect(x + 3 + f, y + 9, 2, 2, corpo);
        } else if !g.a_terra {
            fb.fill_rect(x + 1, y + 9, 2, 2, corpo);
            fb.fill_rect(x + 3, y + 9, 2, 3, corpo);
        } else if g.vel.x.abs() > 5.0 && fase == 1 {
            fb.fill_rect(x, y + 9, 2, 3, corpo);
            fb.fill_rect(x + 4, y + 9, 2, 2, corpo);
        } else {
            fb.fill_rect(x + 1, y + 9, 2, 3, corpo);
            fb.fill_rect(x + 3, y + 9, 2, 3, corpo);
        }
        // La sciarpa rossa, che svolazza dietro.
        let rosso = rgb(220, 40, 52);
        fb.fill_rect(x, y + 4, 6, 1, rosso);
        let dietro = if g.verso > 0.0 { x - 1 } else { x + 6 };
        let sv = ((self.t * 9.0).sin() * 1.2).round() as i32;
        fb.set_pixel(dietro, y + 4 + sv.max(0), rosso);
        fb.set_pixel(dietro - g.verso as i32, y + 5 + sv, rosso);
        // La lanterna in mano.
        let l = self.centro_lanterna();
        let (lx, ly) = (l.x as i32 - cam, l.y as i32);
        let fiamma = mix(rgb(255, 214, 130), rgb(255, 120, 60), (self.foga * 255.0) as u8);
        fb.fill_rect(lx - 1, ly - 1, 2, 3, fiamma);
        self.luce.fill_rect(lx - 1, ly - 1, 2, 3, scala(rgb(255, 190, 100), 0.6 + self.foga));
    }

    /// La voce della Sala: al centro, lettera per lettera, con un respiro
    /// lento e un alone.
    fn disegna_sala(&mut self, fb: &mut Framebuffer) {
        let Some((testo, n, alpha)) = self.sala.visibile() else { return };
        if self.scena != Scena::Gioco {
            return;
        }
        const PASSO: i32 = 7;
        let colore = rgb(226, 220, 255);
        let larg = testo.chars().count() as i32 * PASSO - 2;
        let x0 = (LARGHEZZA as i32 - larg) / 2;
        let a = (alpha * 255.0) as u8;
        for (i, c) in testo.chars().take(n).enumerate() {
            let onda = ((self.t * 1.8 + i as f32 * 0.45).sin() * 1.2).round() as i32;
            let (x, y) = (x0 + i as i32 * PASSO, 50 + onda);
            // Un'ombra sotto, per leggerla anche sopra le scritte dei muri.
            self.font.draw_char(fb, x + 1, y + 1, c, rgb(0, 0, 0), a, false);
            self.font.draw_char(fb, x, y, c, colore, a, false);
            self.font.draw_char(&mut self.luce, x, y, c, scala(colore, 0.45 * alpha), 255, false);
        }
    }

    fn disegna_interfaccia(&mut self, fb: &mut Framebuffer, stats: &Stats) {
        let w = LARGHEZZA as i32;

        // Titolo iniziale.
        if self.t < 3.5 {
            let a = (if self.t < 2.3 { 1.0 } else { (3.5 - self.t) / 1.2 }).clamp(0.0, 1.0);
            testo_grande(&self.font, fb, w / 2, 20, "EGRESSO", rgb(240, 232, 214), (a * 255.0) as u8, 3);
            self.font.draw_centered(fb, w / 2, 46, "UNA SALA CHE TI VUOLE BENE.", rgb(150, 140, 190), (a * 200.0) as u8);
        }

        // Il nome della stanza, entrando.
        if self.t > 4.0 && self.t_stanza < 2.5 {
            let a = (if self.t_stanza < 1.5 { 1.0 } else { 2.5 - self.t_stanza }).clamp(0.0, 1.0);
            testo_grande(&self.font, fb, w / 2, 22, self.mondo.stanza.nome, rgb(240, 232, 214), (a * 230.0) as u8, 2);
        }

        if let Scena::Uscito(t) = self.scena {
            let a = (t / 1.5).min(1.0);
            for p in fb.pixel_mut() {
                *p = mix(*p, rgb(255, 238, 205), (a * 255.0) as u8);
            }
            if t > 1.0 {
                let b = ((t - 1.0) / 1.0).min(1.0);
                let (testo, testo2) =
                    if self.ha_capito { ("L'ACQUA SAPEVA DOVE ANDARE.", "") } else { ("SEI USCITO.", "NESSUNO TE L'AVEVA DETTO.") };
                testo_grande(&self.font, fb, w / 2, 50, "EGRESSO", rgb(60, 44, 40), (b * 255.0) as u8, 2);
                self.font.draw_centered(fb, w / 2, 80, testo, rgb(90, 64, 56), (b * 255.0) as u8);
                self.font.draw_centered(fb, w / 2, 92, testo2, rgb(90, 64, 56), (b * 255.0) as u8);
                if t > 1.8 {
                    let mut buf = [0u8; 64];
                    let passi = (self.sforzo_totale * 6.0).round() as i32;
                    let r = scrivi(&mut buf, format_args!("LA SALA SI ERA ALLUNGATA DI {passi} PASSI."));
                    self.font.draw_centered(fb, w / 2, 112, r, rgb(150, 120, 100), 220);
                }
                if t > 2.5 {
                    self.font.draw_centered(fb, w / 2, 150, "INVIO PER RICOMINCIARE", rgb(150, 120, 100), 200);
                }
            }
        }

        if self.debug {
            let mut buf = [0u8; 64];
            let r1 = scrivi(&mut buf, format_args!("FPS {:.0} UPD {:.2} REN {:.2}", stats.fps, stats.ms_update, stats.ms_render));
            self.font.draw_text(fb, 2, 2, r1, rgb(120, 255, 120));
            let m = self.moduli;
            let r2 = scrivi(&mut buf, format_args!("MODULI {:.1} {:.1} {:.1} FOGA {:.2}", m[0], m[1], m[2], self.foga));
            self.font.draw_text(fb, 2, 11, r2, rgb(120, 255, 120));
            let r3 = scrivi(&mut buf, format_args!("PART {} LETTE {:b}", self.particelle.v.len(), self.lette[self.stanza]));
            self.font.draw_text(fb, 2, 20, r3, rgb(120, 255, 120));
        }
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
