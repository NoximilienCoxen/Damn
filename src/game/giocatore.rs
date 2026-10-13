//! Il protagonista: cammina e salta su un suolo fatto di colline.
//!
//! Il suolo è una funzione della x (`Terreno::suolo`): niente griglia.
//! Un gradino più alto di `GRADINO` è un muro; in discesa i piedi restano
//! attaccati al suolo, così si scende dalle colline senza saltellare.

use super::terreno::Terreno;
use crate::engine::input::{Input, Key};
use crate::engine::math::{Rect, Vec2};
use crate::engine::time::DT;

pub const LARG: f32 = 6.0;
pub const ALT: f32 = 12.0;

const VELOCITA: f32 = 64.0;
const ACC_TERRA: f32 = 900.0;
const ACC_ARIA: f32 = 600.0;
const GRAVITA: f32 = 640.0;
const CADUTA_MAX: f32 = 320.0;
const SALTO: f32 = 210.0;
const COYOTE: f32 = 0.1;
const BUFFER_SALTO: f32 = 0.12;
/// Il gradino più alto che si sale camminando.
const GRADINO: f32 = 5.0;
/// In discesa, i piedi seguono il suolo fino a questa distanza.
const AGGANCIO: f32 = 4.0;

pub struct Giocatore {
    /// Angolo in alto a sinistra. La x non ha limiti: il mondo gira.
    pub pos: Vec2,
    pub vel: Vec2,
    pub a_terra: bool,
    pub verso: f32,
    pub passo: f32,
    coyote: f32,
    buffer: f32,
}

impl Giocatore {
    pub fn new(x: f32, terreno: &Terreno) -> Giocatore {
        let y = terreno.suolo(x + LARG * 0.5) - ALT;
        Giocatore { pos: Vec2::new(x, y), vel: Vec2::ZERO, a_terra: true, verso: 1.0, passo: 0.0, coyote: 0.0, buffer: 0.0 }
    }

    pub fn rect(&self) -> Rect {
        Rect::new(self.pos.x, self.pos.y, LARG, ALT)
    }

    pub fn centro_x(&self) -> f32 {
        self.pos.x + LARG * 0.5
    }

    /// `comandi`: falso quando il gioco toglie il controllo (si entra in Egresso).
    pub fn update(&mut self, input: &Input, terreno: &Terreno, comandi: bool) {
        let mut dir = 0.0;
        if comandi {
            if input.down(Key::Left) || input.down(Key::A) {
                dir -= 1.0;
            }
            if input.down(Key::Right) || input.down(Key::D) {
                dir += 1.0;
            }
        }
        self.passi(dir, input, terreno, comandi);
    }

    /// Cammina da solo verso `x` (senza tasti).
    pub fn verso_punto(&mut self, x: f32, terreno: &Terreno) {
        let d = x - self.centro_x();
        let dir = if d.abs() < 1.0 { 0.0 } else { d.signum() * (d.abs() / 12.0).min(1.0) };
        self.passi(dir, &Input::new(), terreno, false);
    }

    fn passi(&mut self, dir: f32, input: &Input, terreno: &Terreno, comandi: bool) {
        if dir != 0.0 {
            self.verso = dir.signum();
        }
        let acc = if self.a_terra { ACC_TERRA } else { ACC_ARIA };
        self.vel.x = avvicina(self.vel.x, dir * VELOCITA, acc * DT);

        let salto = comandi && (input.pressed(Key::Space) || input.pressed(Key::W) || input.pressed(Key::Up));
        let tenuto = comandi && (input.down(Key::Space) || input.down(Key::W) || input.down(Key::Up));
        self.buffer = if salto { BUFFER_SALTO } else { (self.buffer - DT).max(0.0) };
        self.coyote = if self.a_terra { COYOTE } else { (self.coyote - DT).max(0.0) };
        if self.buffer > 0.0 && self.coyote > 0.0 {
            self.vel.y = -SALTO;
            self.buffer = 0.0;
            self.coyote = 0.0;
            self.a_terra = false;
        }
        if !tenuto && self.vel.y < -70.0 {
            self.vel.y = -70.0;
        }
        self.vel.y = (self.vel.y + GRAVITA * DT).min(CADUTA_MAX);

        // Orizzontale: un gradino troppo alto ferma.
        let piedi = self.pos.y + ALT;
        let nx = self.pos.x + self.vel.x * DT;
        let suolo_nuovo = terreno.suolo(nx + LARG * 0.5);
        if suolo_nuovo < piedi - GRADINO {
            self.vel.x = 0.0;
        } else {
            self.pos.x = nx;
        }

        // Verticale.
        let era_a_terra = self.a_terra;
        self.pos.y += self.vel.y * DT;
        let suolo = terreno.suolo(self.centro_x());
        let piedi = self.pos.y + ALT;
        if piedi >= suolo || (era_a_terra && self.vel.y >= 0.0 && suolo - piedi <= AGGANCIO) {
            self.pos.y = suolo - ALT;
            self.vel.y = 0.0;
            self.a_terra = true;
        } else {
            self.a_terra = false;
        }
        if self.a_terra {
            self.passo += self.vel.x.abs() * DT;
        }
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
    use crate::game::terreno::{COLLINA, PIAZZA};

    #[test]
    fn cammina_sulle_colline_senza_staccarsi() {
        let t = Terreno::new(2);
        let mut g = Giocatore::new(t.partenza, &t);
        let mut input = Input::new();
        input.set(Key::Right, true);
        for _ in 0..600 {
            g.update(&input, &t, true);
            assert!(g.a_terra || g.vel.y < 0.0 || (g.pos.y + ALT - t.suolo(g.centro_x())).abs() < 40.0);
        }
        assert!(g.pos.x > t.partenza + 300.0, "è andato avanti: {}", g.pos.x);
    }

    #[test]
    fn il_fronte_della_collina_e_un_muro() {
        let t = Terreno::new(3);
        // Sulla piazza, verso la collina.
        let x = t.arco + t.lato * (PIAZZA - 30.0);
        let mut g = Giocatore::new(x, &t);
        let mut input = Input::new();
        input.set(if t.lato > 0.0 { Key::Right } else { Key::Left }, true);
        for _ in 0..240 {
            g.update(&input, &t, true);
        }
        assert_eq!(g.pos.y + ALT, t.piazza_y, "resta sulla piazza");
        const { assert!(COLLINA > GRADINO) };
    }
}
