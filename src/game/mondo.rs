//! Il mondo di una stanza: la griglia già allungata dai moduli, e le correnti.
//!
//! La mappa di una `Stanza` è la sua forma "a riposo". Se la stanza ha un
//! modulo, il mondo ne contiene `moduli` copie in più: le colonne dopo il
//! modulo scivolano a destra. Tutto ciò che è definito sulla mappa base
//! (correnti, scritte, lanterne) passa da `x_mondo` per sapere dov'è finito.

use super::stanze::{DefCorrente, Fluido, Stanza, COLONNE, RIGHE, TILE};
use crate::engine::math::{Rect, Vec2};

pub struct Mondo {
    pub stanza: &'static Stanza,
    pub moduli: i32,
    /// Larghezza in tile.
    pub larg: i32,
    celle: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Corrente {
    pub vel: Vec2,
    pub fluido: Fluido,
}

/// Quanti tile sopravento si guardano per capire se si è al riparo.
const RIPARO: i32 = 3;
/// Secondi di preavviso prima di una raffica.
pub const PREAVVISO: f32 = 0.8;

impl Mondo {
    pub fn new(stanza: &'static Stanza, moduli: i32) -> Mondo {
        let moduli = if stanza.modulo.is_some() { moduli.max(0) } else { 0 };
        let w = stanza.modulo.map_or(0, |m| m.1);
        let larg = COLONNE + moduli * w;
        let mut m = Mondo { stanza, moduli, larg, celle: Vec::with_capacity((larg * RIGHE) as usize) };
        for y in 0..RIGHE {
            let riga = stanza.mappa[y as usize].as_bytes();
            for x in 0..larg {
                let b = m.x_base(x);
                m.celle.push(riga[b as usize]);
            }
        }
        m
    }

    /// Da colonna del mondo a colonna della mappa base.
    pub fn x_base(&self, x: i32) -> i32 {
        match self.stanza.modulo {
            Some((s, w)) if x >= s => {
                if x < s + (self.moduli + 1) * w {
                    s + (x - s) % w
                } else {
                    x - self.moduli * w
                }
            }
            _ => x,
        }
    }

    /// Da colonna base a colonna del mondo, per un punto (dentro il modulo:
    /// la prima copia).
    pub fn x_mondo(&self, x: i32) -> i32 {
        match self.stanza.modulo {
            Some((s, w)) if x >= s + w => x + self.moduli * w,
            _ => x,
        }
    }

    /// Come `x_mondo`, ma per l'estremo destro di un'area: un'area che
    /// copre il modulo si allunga con lui.
    pub fn x1_mondo(&self, x: i32) -> i32 {
        match self.stanza.modulo {
            Some((s, w)) if x >= s => x + self.moduli * w,
            _ => x,
        }
    }

    /// Dove comincia (px) e quanto è largo il tratto ripetuto, adesso.
    pub fn tratto_modulo(&self) -> Option<(f32, f32)> {
        let (s, w) = self.stanza.modulo?;
        Some(((s * TILE) as f32, ((self.moduli + 1) * w * TILE) as f32))
    }

    pub fn larg_px(&self) -> f32 {
        (self.larg * TILE) as f32
    }

    /// Fuori dalla griglia: sopra e sotto è roccia, ai lati aria (le porte).
    pub fn cella(&self, tx: i32, ty: i32) -> u8 {
        if !(0..RIGHE).contains(&ty) {
            return b'#';
        }
        if !(0..self.larg).contains(&tx) {
            return b'.';
        }
        self.celle[(ty * self.larg + tx) as usize]
    }

    pub fn solido(&self, tx: i32, ty: i32) -> bool {
        self.cella(tx, ty) == b'#'
    }

    pub fn acqua(&self, tx: i32, ty: i32) -> bool {
        matches!(self.cella(tx, ty), b'~' | b'O')
    }

    pub fn tile(p: Vec2) -> (i32, i32) {
        ((p.x / TILE as f32).floor() as i32, (p.y / TILE as f32).floor() as i32)
    }

    pub fn acqua_in(&self, p: Vec2) -> bool {
        let (tx, ty) = Mondo::tile(p);
        self.acqua(tx, ty)
    }

    /// La quota (px) della superficie sopra il punto `p`, che sta in acqua.
    pub fn superficie(&self, p: Vec2) -> f32 {
        let (tx, mut ty) = Mondo::tile(p);
        while self.acqua(tx, ty - 1) {
            ty -= 1;
        }
        (ty * TILE) as f32
    }

    /// L'area di una corrente, in pixel del mondo.
    pub fn area(&self, d: &DefCorrente) -> Rect {
        let (x0, y0, x1, y1) = d.area;
        let (x0, x1) = (self.x_mondo(x0), self.x1_mondo(x1));
        let t = TILE as f32;
        Rect::new(x0 as f32 * t, y0 as f32 * t, (x1 - x0 + 1) as f32 * t, (y1 - y0 + 1) as f32 * t)
    }

    /// Un tile è al riparo da una corrente orizzontale se c'è roccia poco
    /// sopravento, sulla stessa riga.
    pub fn riparato(&self, tx: i32, ty: i32, d: &DefCorrente) -> bool {
        if d.fluido != Fluido::Aria || d.vel.0 == 0.0 {
            return false;
        }
        let verso = if d.vel.0 < 0.0 { 1 } else { -1 };
        (1..=RIPARO).any(|k| self.solido(tx + verso * k, ty))
    }

    /// La corrente che agisce nel punto `p` all'istante `t`, se c'è.
    pub fn corrente(&self, p: Vec2, t: f32) -> Option<Corrente> {
        let (tx, ty) = Mondo::tile(p);
        for d in self.stanza.correnti {
            let a = self.area(d);
            if p.x < a.x || p.x >= a.right() || p.y < a.y || p.y >= a.bottom() {
                continue;
            }
            if !attiva(d, t) || self.riparato(tx, ty, d) {
                continue;
            }
            return Some(Corrente { vel: Vec2::new(d.vel.0, d.vel.1), fluido: d.fluido });
        }
        None
    }
}

/// Una corrente a raffiche soffia solo per una parte del suo periodo.
pub fn attiva(d: &DefCorrente, t: f32) -> bool {
    match d.raffiche {
        None => true,
        Some((periodo, durata)) => t.rem_euclid(periodo) < durata,
    }
}

/// 0..1 mentre una raffica sta per arrivare (si vede prima di sentirla).
pub fn preavviso(d: &DefCorrente, t: f32) -> f32 {
    match d.raffiche {
        None => 0.0,
        Some((periodo, _)) => {
            let f = t.rem_euclid(periodo);
            ((f - (periodo - PREAVVISO)) / PREAVVISO).clamp(0.0, 1.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::stanze::{ATRIO, GALLERIA, RIVA};

    #[test]
    fn i_moduli_allungano_la_stanza() {
        let m0 = Mondo::new(&ATRIO, 0);
        let m3 = Mondo::new(&ATRIO, 3);
        assert_eq!(m0.larg, 40);
        assert_eq!(m3.larg, 40 + 3 * 6);
        // La porta in fondo è sempre l'ultima colonna, e la riga è la stessa.
        for y in 0..RIGHE {
            assert_eq!(m0.cella(39, y), m3.cella(39 + 18, y));
            assert_eq!(m0.cella(3, y), m3.cella(3, y));
        }
        // Le copie del modulo sono uguali all'originale.
        for y in 0..RIGHE {
            for k in 0..4 {
                assert_eq!(m3.cella(18 + k * 6, y), m0.cella(18, y));
            }
        }
    }

    #[test]
    fn mappatura_andata_e_ritorno() {
        let m = Mondo::new(&RIVA, 5);
        for x in 0..COLONNE {
            assert_eq!(m.x_base(m.x_mondo(x)), x);
        }
    }

    #[test]
    fn la_superficie_e_in_cima_all_acqua() {
        let m = Mondo::new(&GALLERIA, 0);
        let p = Vec2::new(10.0 * 8.0, 18.0 * 8.0);
        assert!(m.acqua_in(p));
        assert_eq!(m.superficie(p), 13.0 * 8.0);
    }

    #[test]
    fn dietro_un_blocco_il_vento_non_arriva() {
        let m = Mondo::new(&ATRIO, 0);
        let d = &ATRIO.correnti[0];
        // Subito a sinistra del blocco in colonna 12 (il vento viene da destra).
        assert!(m.riparato(11, 11, d));
        assert!(!m.riparato(8, 11, d));
        // In raffica, al riparo non c'è corrente.
        let t = 0.1;
        assert!(attiva(d, t));
        assert!(m.corrente(Vec2::new(11.0 * 8.0 + 4.0, 11.0 * 8.0 + 4.0), t).is_none());
        assert!(m.corrente(Vec2::new(8.0 * 8.0 + 4.0, 11.0 * 8.0 + 4.0), t).is_some());
    }
}
