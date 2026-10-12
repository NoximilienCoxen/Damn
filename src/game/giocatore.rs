//! Il protagonista: un platformer semplice ma preciso.
//!
//! Collisioni AABB contro la griglia, risolte un asse alla volta: prima ci
//! si muove in orizzontale e si esce dai muri, poi in verticale e si esce da
//! pavimenti e soffitti. Separare gli assi evita quasi tutti i casi strani
//! (angoli che "agganciano", teletrasporti).

use super::lessico::Natura;
use super::livello::{COLONNE, PARTENZA, TILE};
use super::mondo::Mondo;
use crate::engine::input::{Input, Key};
use crate::engine::math::{Rect, Vec2};
use crate::engine::time::DT;

pub const LARG: f32 = 6.0;
pub const ALT: f32 = 12.0;

const VELOCITA: f32 = 72.0;
const ACC_TERRA: f32 = 1100.0;
const ACC_ARIA: f32 = 700.0;
const ACC_BURRO: f32 = 90.0;
const GRAVITA: f32 = 640.0;
const CADUTA_MAX: f32 = 320.0;
const SALTO: f32 = 218.0;
const COYOTE: f32 = 0.1;
const BUFFER_SALTO: f32 = 0.12;

pub struct Giocatore {
    pub pos: Vec2,
    pub vel: Vec2,
    pub a_terra: bool,
    pub sotto: Option<Natura>,
    pub verso: f32,
    pub passo: f32,
    coyote: f32,
    buffer: f32,
    /// Ultimo punto in cui stava su qualcosa di nominato (per ripartire).
    pub sicuro: Vec2,
}

/// Cosa è successo al corpo in questo passo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Esito {
    Niente,
    Caduto,
}

impl Giocatore {
    pub fn new() -> Giocatore {
        let p = Vec2::new(PARTENZA.0, PARTENZA.1);
        Giocatore { pos: p, vel: Vec2::ZERO, a_terra: false, sotto: None, verso: 1.0, passo: 0.0, coyote: 0.0, buffer: 0.0, sicuro: p }
    }

    pub fn rect(&self) -> Rect {
        Rect::new(self.pos.x, self.pos.y, LARG, ALT)
    }

    pub fn riparti(&mut self) {
        self.pos = self.sicuro;
        self.vel = Vec2::ZERO;
    }

    pub fn update(&mut self, input: &Input, mondo: &mut Mondo, controlli: bool) -> Esito {
        let mut dir = 0.0;
        if controlli {
            if input.down(Key::Left) || input.down(Key::A) {
                dir -= 1.0;
            }
            if input.down(Key::Right) || input.down(Key::D) {
                dir += 1.0;
            }
        }
        if dir != 0.0 {
            self.verso = dir;
        }

        let acc = match (self.a_terra, self.sotto) {
            (true, Some(Natura::Scivolosa)) => ACC_BURRO,
            (true, _) => ACC_TERRA,
            (false, _) => ACC_ARIA,
        };
        self.vel.x = avvicina(self.vel.x, dir * VELOCITA, acc * DT);

        // Salto con "coyote time" (si può saltare un attimo dopo aver lasciato
        // il bordo) e buffer (il tasto premuto un attimo prima di atterrare vale).
        let salto = controlli && (input.pressed(Key::Space) || input.pressed(Key::W) || input.pressed(Key::Up));
        let tenuto = input.down(Key::Space) || input.down(Key::W) || input.down(Key::Up);
        self.buffer = if salto { BUFFER_SALTO } else { (self.buffer - DT).max(0.0) };
        self.coyote = if self.a_terra { COYOTE } else { (self.coyote - DT).max(0.0) };
        if self.buffer > 0.0 && self.coyote > 0.0 {
            self.vel.y = -SALTO;
            self.buffer = 0.0;
            self.coyote = 0.0;
        }
        if !tenuto && self.vel.y < -70.0 {
            self.vel.y = -70.0; // salto più basso se si lascia il tasto
        }
        self.vel.y = (self.vel.y + GRAVITA * DT).min(CADUTA_MAX);

        self.muovi_x(mondo);
        self.muovi_y(mondo);

        // Ai lati la stanza è chiusa.
        let max_x = (COLONNE * TILE) as f32 - LARG;
        if self.pos.x < 0.0 || self.pos.x > max_x {
            self.pos.x = self.pos.x.clamp(0.0, max_x);
            self.vel.x = 0.0;
        }
        if self.pos.y < 0.0 {
            self.pos.y = 0.0;
            self.vel.y = self.vel.y.max(0.0);
        }

        if self.a_terra {
            self.passo += self.vel.x.abs() * DT;
            // Su cosa poggiano i piedi? Una lettera nominata consuma la parola.
            let ty = ((self.pos.y + ALT + 0.5) / TILE as f32).floor() as i32;
            let mut su_nome = false;
            for tx in self.colonne() {
                if let Some(c) = mondo.cella(tx, ty) {
                    if c.natura.regge() {
                        mondo.calpesta(c.percetto);
                        su_nome = true;
                    }
                }
            }
            if su_nome {
                self.sicuro = self.pos;
            }
        }

        if self.pos.y > 190.0 {
            return Esito::Caduto;
        }
        Esito::Niente
    }

    fn colonne(&self) -> std::ops::RangeInclusive<i32> {
        let t = TILE as f32;
        (self.pos.x / t).floor() as i32..=((self.pos.x + LARG - 0.001) / t).floor() as i32
    }

    fn righe(&self) -> std::ops::RangeInclusive<i32> {
        let t = TILE as f32;
        (self.pos.y / t).floor() as i32..=((self.pos.y + ALT - 0.001) / t).floor() as i32
    }

    fn muovi_x(&mut self, mondo: &Mondo) {
        self.pos.x += self.vel.x * DT;
        let t = TILE as f32;
        for ty in self.righe() {
            for tx in self.colonne() {
                if mondo.solido(tx, ty).is_some() {
                    if self.vel.x > 0.0 {
                        self.pos.x = tx as f32 * t - LARG;
                    } else if self.vel.x < 0.0 {
                        self.pos.x = (tx + 1) as f32 * t;
                    }
                    self.vel.x = 0.0;
                }
            }
        }
    }

    fn muovi_y(&mut self, mondo: &Mondo) {
        self.pos.y += self.vel.y * DT;
        self.a_terra = false;
        self.sotto = None;
        let t = TILE as f32;
        for ty in self.righe() {
            for tx in self.colonne() {
                if let Some(n) = mondo.solido(tx, ty) {
                    if self.vel.y > 0.0 {
                        self.pos.y = ty as f32 * t - ALT;
                        self.a_terra = true;
                        self.sotto = Some(n);
                    } else if self.vel.y < 0.0 {
                        self.pos.y = (ty + 1) as f32 * t;
                    }
                    self.vel.y = 0.0;
                }
            }
        }
    }
}

impl Default for Giocatore {
    fn default() -> Self {
        Self::new()
    }
}

fn avvicina(v: f32, obiettivo: f32, passo: f32) -> f32 {
    if v < obiettivo {
        (v + passo).min(obiettivo)
    } else {
        (v - passo).max(obiettivo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sta_in_piedi_sul_pavimento_nominato() {
        let mut m = Mondo::new();
        let mut g = Giocatore::new();
        let input = Input::new();
        for _ in 0..60 {
            g.update(&input, &mut m, true);
        }
        assert!(g.a_terra);
        assert_eq!(g.pos.y, (17 * TILE) as f32 - ALT);
    }

    #[test]
    fn nel_vuoto_si_cade() {
        let mut m = Mondo::new();
        let mut g = Giocatore::new();
        g.pos.x = 150.0; // sopra l'abisso non nominato
        let input = Input::new();
        let mut caduto = false;
        for _ in 0..120 {
            caduto |= g.update(&input, &mut m, true) == Esito::Caduto;
        }
        assert!(caduto);
    }

    #[test]
    fn il_muro_ferma() {
        let mut m = Mondo::new();
        let mut g = Giocatore::new();
        let mut input = Input::new();
        input.set(Key::Left, true);
        for _ in 0..120 {
            g.update(&input, &mut m, true);
        }
        assert_eq!(g.pos.x, TILE as f32, "si ferma contro il MURO (colonna 0)");
    }
}
