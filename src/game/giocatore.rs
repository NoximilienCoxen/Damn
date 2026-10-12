//! Il protagonista: un platformer preciso, più due regole del corpo.
//!
//! **Dentro una corrente, chi si muove resta dov'è; chi si lascia andare
//! viene portato.**
//! - In acqua, finché tieni premuto qualcosa nuoti: resti a galla, controlli
//!   la direzione, e la corrente ti frena appena. Se lasci tutto, il corpo
//!   affonda piano fino a una profondità di equilibrio e la corrente di
//!   fondo ti porta con sé.
//! - Nell'aria che soffia vale lo stesso: con i tasti premuti la fisica è
//!   quella di sempre; senza, il vento ti prende.
//!
//! E una regola dello sforzo: spingere contro un muro, contro una corrente,
//! o saltare a ripetizione è **forzare**. Il gioco lo misura (`Moto::sforzo`)
//! e la Sala ci reagisce allungandosi.
//!
//! Collisioni AABB contro la griglia, un asse alla volta.

use super::mondo::Mondo;
use super::stanze::{Fluido, TILE};
use crate::engine::input::{Input, Key};
use crate::engine::math::{Rect, Vec2};
use crate::engine::time::DT;

pub const LARG: f32 = 6.0;
pub const ALT: f32 = 12.0;

const VELOCITA: f32 = 72.0;
const NUOTO: f32 = 56.0;
const ACC_TERRA: f32 = 1100.0;
const ACC_ARIA: f32 = 700.0;
const GRAVITA: f32 = 640.0;
const CADUTA_MAX: f32 = 320.0;
pub const SALTO: f32 = 218.0;
const COYOTE: f32 = 0.1;
const BUFFER_SALTO: f32 = 0.12;
/// Quanto sotto la superficie si ferma un corpo che si lascia andare.
pub const PROFONDITA_QUIETE: f32 = 26.0;
/// Salti "di foga": oltre questa soglia saltare è forzare.
const FOGA_MAX: f32 = 2.2;

pub struct Giocatore {
    pub pos: Vec2,
    pub vel: Vec2,
    pub a_terra: bool,
    pub verso: f32,
    pub passo: f32,
    pub in_acqua: bool,
    pub rilassato: bool,
    coyote: f32,
    buffer: f32,
    foga: f32,
}

/// Cosa è successo al corpo in questo passo.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Moto {
    pub esito: Esito,
    /// Quanto ha forzato in questo passo (0 = per niente).
    pub sforzo: f32,
    pub contro_muro: bool,
    /// Portato da una corrente (lasciandosi andare).
    pub portato: Option<Fluido>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Esito {
    Niente,
    /// Ha toccato l'arco: fuori.
    Varco,
    /// Ha passato il bordo sinistro (-1) o destro (1).
    Bordo(i8),
}

impl Giocatore {
    pub fn new(x: f32, y: f32) -> Giocatore {
        Giocatore {
            pos: Vec2::new(x, y),
            vel: Vec2::ZERO,
            a_terra: false,
            verso: 1.0,
            passo: 0.0,
            in_acqua: false,
            rilassato: true,
            coyote: 0.0,
            buffer: 0.0,
            foga: 0.0,
        }
    }

    pub fn rect(&self) -> Rect {
        Rect::new(self.pos.x, self.pos.y, LARG, ALT)
    }

    pub fn update(&mut self, input: &Input, mondo: &Mondo, t: f32) -> Moto {
        let mut dir = 0.0;
        if input.down(Key::Left) || input.down(Key::A) {
            dir -= 1.0;
        }
        if input.down(Key::Right) || input.down(Key::D) {
            dir += 1.0;
        }
        if dir != 0.0 {
            self.verso = dir;
        }
        let salto = input.pressed(Key::Space) || input.pressed(Key::W) || input.pressed(Key::Up);
        let tenuto = input.down(Key::Space) || input.down(Key::W) || input.down(Key::Up);
        self.rilassato = dir == 0.0 && !tenuto;

        let centro = self.rect().center();
        self.in_acqua = mondo.acqua_in(centro);
        let corrente = mondo.corrente(centro, t);
        let c = corrente.map_or(Vec2::ZERO, |c| c.vel);
        let mut sforzo = 0.0;
        let mut portato = None;

        self.foga = (self.foga - DT * 0.9).max(0.0);
        if salto {
            self.foga += 1.0;
        }
        if self.foga > FOGA_MAX {
            sforzo += 1.0;
        }

        if self.in_acqua {
            let superficie = mondo.superficie(centro);
            self.coyote = 0.0;
            self.buffer = 0.0;
            if self.rilassato {
                // Il corpo si abbandona: scende fin dove galleggia, e va dove va l'acqua.
                let quiete = superficie + PROFONDITA_QUIETE;
                let vy = ((quiete - self.pos.y) * 2.0).clamp(-30.0, 25.0) + c.y;
                self.vel.y = avvicina(self.vel.y, vy, 140.0 * DT);
                self.vel.x = avvicina(self.vel.x, c.x, 170.0 * DT);
                if c != Vec2::ZERO {
                    portato = Some(Fluido::Acqua);
                }
            } else {
                // Nuotare: testa fuori, e la corrente frena ma non comanda.
                let galla = superficie - 4.0;
                let vy = ((galla - self.pos.y) * 4.0).clamp(-70.0, 60.0);
                self.vel.y = avvicina(self.vel.y, vy, 420.0 * DT);
                self.vel.x = avvicina(self.vel.x, dir * NUOTO + c.x * 0.6, 520.0 * DT);
                if dir * c.x < 0.0 {
                    sforzo += 1.0;
                }
                if salto && self.pos.y <= galla + 3.0 {
                    self.vel.y = -SALTO * 0.9;
                }
            }
        } else {
            let aria = corrente.filter(|c| c.fluido == Fluido::Aria);
            let acc = if self.a_terra { ACC_TERRA } else { ACC_ARIA };
            if self.rilassato && aria.is_some() {
                portato = Some(Fluido::Aria);
            }
            // Orizzontale: il vento porta chi si lascia andare, frena chi va contro.
            if self.rilassato && c.x != 0.0 {
                self.vel.x = avvicina(self.vel.x, c.x, 420.0 * DT);
            } else {
                self.vel.x = avvicina(self.vel.x, dir * VELOCITA + c.x * 0.6, acc * DT);
                if dir * c.x < 0.0 {
                    sforzo += 1.0;
                }
            }
            // Salto con "coyote time" e buffer.
            self.buffer = if salto { BUFFER_SALTO } else { (self.buffer - DT).max(0.0) };
            self.coyote = if self.a_terra { COYOTE } else { (self.coyote - DT).max(0.0) };
            if self.buffer > 0.0 && self.coyote > 0.0 {
                self.vel.y = -SALTO;
                self.buffer = 0.0;
                self.coyote = 0.0;
            }
            // Verticale: una corrente che sale porta chi si lascia andare.
            if self.rilassato && c.y != 0.0 {
                self.vel.y = avvicina(self.vel.y, c.y, 520.0 * DT);
            } else {
                if !tenuto && self.vel.y < -70.0 {
                    self.vel.y = -70.0; // salto più basso se si lascia il tasto
                }
                // Chi si dimena in una corrente che sale non viene portato, ma cade piano.
                let g = if c.y < 0.0 { GRAVITA * 0.3 } else { GRAVITA };
                self.vel.y = (self.vel.y + g * DT).min(CADUTA_MAX);
            }
        }

        let urto = self.muovi_x(mondo);
        self.muovi_y(mondo);
        let contro_muro = dir != 0.0 && urto == dir;
        if contro_muro {
            sforzo += 1.0;
        }

        if self.pos.y < 0.0 {
            self.pos.y = 0.0;
            self.vel.y = self.vel.y.max(0.0);
        }
        if self.a_terra {
            self.passo += self.vel.x.abs() * DT;
        }

        let esito = if self.tocca_varco(mondo) {
            Esito::Varco
        } else if self.pos.x < -LARG * 0.5 {
            Esito::Bordo(-1)
        } else if self.pos.x > mondo.larg_px() - LARG * 0.5 {
            Esito::Bordo(1)
        } else {
            Esito::Niente
        };
        Moto { esito, sforzo, contro_muro, portato }
    }

    fn tocca_varco(&self, mondo: &Mondo) -> bool {
        let r = self.rect();
        let (x0, y0) = Mondo::tile(Vec2::new(r.x, r.y));
        let (x1, y1) = Mondo::tile(Vec2::new(r.right() - 0.01, r.bottom() - 0.01));
        (y0..=y1).any(|ty| (x0..=x1).any(|tx| mondo.cella(tx, ty) == b'O'))
    }

    fn colonne(&self) -> std::ops::RangeInclusive<i32> {
        let t = TILE as f32;
        (self.pos.x / t).floor() as i32..=((self.pos.x + LARG - 0.001) / t).floor() as i32
    }

    fn righe(&self) -> std::ops::RangeInclusive<i32> {
        let t = TILE as f32;
        (self.pos.y / t).floor() as i32..=((self.pos.y + ALT - 0.001) / t).floor() as i32
    }

    /// Restituisce il verso in cui ha urtato un muro (0 se niente).
    fn muovi_x(&mut self, mondo: &Mondo) -> f32 {
        let verso = self.vel.x.signum();
        self.pos.x += self.vel.x * DT;
        let t = TILE as f32;
        let mut urto = 0.0;
        for ty in self.righe() {
            for tx in self.colonne() {
                if mondo.solido(tx, ty) {
                    if self.vel.x > 0.0 {
                        self.pos.x = tx as f32 * t - LARG;
                    } else if self.vel.x < 0.0 {
                        self.pos.x = (tx + 1) as f32 * t;
                    }
                    self.vel.x = 0.0;
                    urto = verso;
                }
            }
        }
        // Fermo contro un muro, spingendo: anche con velocità nulla conta.
        if urto == 0.0 && self.vel.x == 0.0 {
            for ty in self.righe() {
                if mondo.solido(((self.pos.x + LARG + 0.5) / t).floor() as i32, ty) {
                    urto = 1.0;
                }
                if mondo.solido(((self.pos.x - 0.5) / t).floor() as i32, ty) {
                    urto = -1.0;
                }
            }
        }
        urto
    }

    fn muovi_y(&mut self, mondo: &Mondo) {
        self.pos.y += self.vel.y * DT;
        self.a_terra = false;
        let t = TILE as f32;
        for ty in self.righe() {
            for tx in self.colonne() {
                if mondo.solido(tx, ty) {
                    if self.vel.y > 0.0 {
                        self.pos.y = ty as f32 * t - ALT;
                        self.a_terra = true;
                    } else if self.vel.y < 0.0 {
                        self.pos.y = (ty + 1) as f32 * t;
                    }
                    self.vel.y = 0.0;
                }
            }
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
    use crate::game::stanze::{ATRIO, GALLERIA};

    fn passi(g: &mut Giocatore, m: &Mondo, input: &mut Input, n: usize, t0: f32) -> Vec<Moto> {
        (0..n)
            .map(|i| {
                let r = g.update(input, m, t0 + i as f32 * DT);
                input.fine_passo();
                r
            })
            .collect()
    }

    #[test]
    fn sta_in_piedi_sul_pavimento() {
        let m = Mondo::new(&ATRIO, 0);
        let mut g = Giocatore::new(20.0, 70.0);
        passi(&mut g, &m, &mut Input::new(), 60, 2.0);
        assert!(g.a_terra);
        assert_eq!(g.pos.y, 96.0 - ALT);
    }

    #[test]
    fn chi_nuota_resta_a_galla_e_urta_il_labbro() {
        let m = Mondo::new(&GALLERIA, 0);
        // In acqua, a sinistra del labbro (colonne 16-17), nuotando verso destra.
        let mut g = Giocatore::new(100.0, 108.0);
        let mut input = Input::new();
        input.set(Key::Right, true);
        let moti = passi(&mut g, &m, &mut input, 240, 0.0);
        assert!(g.pos.x + LARG <= 16.0 * 8.0 + 0.01, "fermo prima del labbro: {}", g.pos.x);
        assert!(g.pos.y < 13.0 * 8.0, "a galla: {}", g.pos.y);
        assert!(moti.iter().any(|m| m.contro_muro && m.sforzo > 0.0));
    }

    #[test]
    fn chi_si_lascia_andare_passa_sotto() {
        let m = Mondo::new(&GALLERIA, 0);
        let mut g = Giocatore::new(100.0, 108.0);
        passi(&mut g, &m, &mut Input::new(), 360, 0.0);
        assert!(g.pos.x > 18.0 * 8.0, "portato oltre il labbro: {}", g.pos.x);
    }

    #[test]
    fn la_colonna_d_aria_porta_in_alto_chi_non_tocca_niente() {
        let m = Mondo::new(&GALLERIA, 0);
        let mut g = Giocatore::new(35.0 * 8.0, 84.0);
        let moti = passi(&mut g, &m, &mut Input::new(), 120, 0.0);
        assert!(g.pos.y < 5.0 * 8.0, "in alto: {}", g.pos.y);
        assert!(moti.iter().any(|m| m.portato == Some(Fluido::Aria)));
    }

    #[test]
    fn contro_la_raffica_si_forza() {
        let m = Mondo::new(&ATRIO, 0);
        let mut g = Giocatore::new(8.0 * 8.0, 84.0);
        let mut input = Input::new();
        input.set(Key::Right, true);
        // t = 0.1: la raffica soffia.
        let moti = passi(&mut g, &m, &mut input, 30, 0.1);
        assert!(moti.iter().skip(5).all(|m| m.sforzo > 0.0 || m.contro_muro));
    }
}
