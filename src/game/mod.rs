//! EGRESSO, il prototipo della Sala che cresce: la Riva, l'Atrio e la
//! Galleria delle Correnti (vedi `docs/CONCEPT.md`).
//!
//! Le regole, tutte fisiche e prevedibili:
//! - **più forzi, più la Sala si allunga** (spingere contro muri, venti e
//!   correnti, saltare a ripetizione); attraversarla senza forzare la
//!   riporta alla sua misura;
//! - **dentro una corrente, chi si muove resta dov'è; chi si lascia andare
//!   viene portato** (vedi `giocatore.rs`).
//!
//! Comandi: frecce/A-D muovi, Spazio/W salta, F3 debug.

pub mod giocatore;
pub mod mondo;
pub mod particelle;
mod render;
pub mod sala;
pub mod stanze;

use crate::engine::app::{Game, Stats, ALTEZZA, LARGHEZZA};
use crate::engine::font::Font;
use crate::engine::framebuffer::{rgb, Framebuffer};
use crate::engine::glow::Glow;
use crate::engine::input::{Input, Key};
use crate::engine::math::{Rng, Vec2};
use crate::engine::time::DT;

use giocatore::{Esito, Giocatore, Moto};
use mondo::Mondo;
use particelle::{Particella, Particelle};
use sala::{Battuta, Sala};
use stanze::{Fluido, ATRIO_I, GALLERIA_I, INDIZIO, RIVA_I, STANZE, TILE};

/// Moduli aggiunti al secondo mentre si forza.
const CRESCITA: f32 = 1.3;
/// Moduli tolti per ogni modulo percorso senza forzare.
const RITIRO: f32 = 0.5;
const MAX_MODULI: f32 = 30.0;
/// Dove si entra da una porta (le porte sono tutte alla stessa altezza).
const Y_PORTA: f32 = 96.0 - giocatore::ALT;
/// Dove ti svegli: la colonna della Riva (mappa base).
const COLONNA_INIZIO: i32 = 30;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Scena {
    Gioco,
    /// Sei uscito (secondi).
    Uscito(f32),
}

pub struct Egresso {
    pub mondo: Mondo,
    pub giocatore: Giocatore,
    pub particelle: Particelle,
    pub sala: Sala,
    pub scena: Scena,
    pub stanza: usize,
    /// Quanto è allungata ogni stanza (in moduli, con la parte frazionaria).
    pub moduli: [f32; STANZE.len()],
    /// Moduli creati in tutta la partita, forzando.
    pub sforzo_totale: f32,
    pub t: f32,
    pub t_stanza: f32,
    /// Telecamera: x del bordo sinistro dello schermo, in pixel del mondo.
    pub cam: f32,
    pub scossa: f32,
    /// Quanto sta forzando adesso (0..1, smussato): la lanterna brucia più forte.
    pub foga: f32,
    /// Scritte lette, una maschera di bit per stanza.
    pub lette: [u32; STANZE.len()],
    lettura: Option<(usize, f32)>,
    pub ha_capito: bool,
    t_muro: f32,
    pub debug: bool,
    rng: Rng,
    font: Font,
    luce: Framebuffer,
    glow: Glow,
    /// La stanza dipinta una volta (si ridipinge quando cambia forma).
    sfondo: Vec<u32>,
    sfondo_luce: Vec<u32>,
}

impl Egresso {
    pub fn new() -> Egresso {
        let moduli = STANZE.map(|s| s.moduli_iniziali);
        let mondo = Mondo::new(STANZE[RIVA_I], moduli[RIVA_I] as i32);
        let x = (mondo.x_mondo(COLONNA_INIZIO) * TILE) as f32;
        let (sfondo, sfondo_luce) = render::dipingi(&mondo);
        let mut g = Egresso {
            giocatore: Giocatore::new(x, Y_PORTA),
            mondo,
            particelle: Particelle::new(),
            sala: Sala::new(),
            scena: Scena::Gioco,
            stanza: RIVA_I,
            moduli,
            sforzo_totale: 0.0,
            t: 0.0,
            t_stanza: 0.0,
            cam: 0.0,
            scossa: 0.0,
            foga: 0.0,
            lette: [0; STANZE.len()],
            lettura: None,
            ha_capito: false,
            t_muro: 0.0,
            debug: false,
            rng: Rng::new(0x5eed),
            font: Font::new(),
            luce: Framebuffer::new(LARGHEZZA, ALTEZZA),
            glow: Glow::new(LARGHEZZA, ALTEZZA),
            sfondo,
            sfondo_luce,
        };
        g.giocatore.verso = 1.0;
        g.cam = g.cam_obiettivo();
        g
    }

    /// Comincia dalla stanza `i`, entrando da sinistra (per provare le stanze).
    pub fn entra(&mut self, i: usize) {
        if i < STANZE.len() && i != self.stanza {
            self.vai_stanza(i, -1);
        }
    }

    fn cam_obiettivo(&self) -> f32 {
        let max = (self.mondo.larg_px() - LARGHEZZA as f32).max(0.0);
        (self.giocatore.pos.x + giocatore::LARG * 0.5 - LARGHEZZA as f32 * 0.5).clamp(0.0, max)
    }

    /// Passa nella stanza `i`, entrando dal lato `da` (-1 sinistra, 1 destra).
    fn vai_stanza(&mut self, i: usize, da: i8) {
        self.stanza = i;
        self.t_stanza = 0.0;
        self.mondo = Mondo::new(STANZE[i], self.moduli[i] as i32);
        let x = if da < 0 { 1.0 } else { self.mondo.larg_px() - giocatore::LARG - 1.0 };
        let vx = self.giocatore.vel.x;
        let verso = self.giocatore.verso;
        self.giocatore = Giocatore::new(x, Y_PORTA);
        self.giocatore.vel.x = vx;
        self.giocatore.verso = verso;
        (self.sfondo, self.sfondo_luce) = render::dipingi(&self.mondo);
        self.particelle.clear();
        self.lettura = None;
        self.cam = self.cam_obiettivo();
        match i {
            ATRIO_I => self.sala.di(Battuta::Atrio),
            GALLERIA_I => self.sala.di(Battuta::Galleria),
            RIVA_I => self.sala.di(Battuta::Ritorno),
            _ => {}
        }
    }

    /// La Sala si allunga o si accorcia: `n` copie del modulo.
    fn ridimensiona(&mut self, n: i32) {
        let Some((inizio, larg)) = self.mondo.tratto_modulo() else { return };
        let vecchio = self.mondo.moduli;
        let w = (self.mondo.stanza.modulo.map_or(0, |m| m.1) * TILE) as f32;
        let delta = (n - vecchio) as f32 * w;
        let fine = inizio + larg;
        let g = &mut self.giocatore;
        if g.pos.x >= fine {
            // Oltre il tratto ripetuto: il mondo scorre, e tu con lui.
            g.pos.x += delta;
            self.cam += delta;
        } else if g.pos.x > inizio {
            // Dentro il tratto: le copie sono identiche, basta restarci dentro.
            g.pos.x = g.pos.x.min(fine + delta - giocatore::LARG - 0.5);
        }
        self.particelle.sposta_oltre(fine, delta);
        self.mondo = Mondo::new(self.mondo.stanza, n);
        (self.sfondo, self.sfondo_luce) = render::dipingi(&self.mondo);
        self.scossa = 0.3;
        // Polvere dove la Sala è cambiata.
        let bordo = if g.pos.x >= fine { fine } else { fine + delta.min(0.0) };
        for _ in 0..40 {
            let pos = Vec2::new(bordo + self.rng.range(-4.0, 4.0), self.rng.range(20.0, 100.0));
            let vel = Vec2::new(self.rng.range(-10.0, 10.0), self.rng.range(-5.0, 20.0));
            let vita = self.rng.range(0.6, 1.4);
            self.particelle.spawn(Particella { pos, vel, vita, durata: vita, colore: rgb(140, 120, 150), gravita: 30.0 });
        }
        if n > vecchio {
            self.sala.di(Battuta::Cresce);
        } else {
            self.sala.di(Battuta::Rallenti);
        }
    }

    /// La regola dello sforzo, un passo alla volta.
    fn sforzo(&mut self, moto: &Moto) {
        self.foga += ((moto.sforzo.min(1.0)) - self.foga) * (6.0 * DT).min(1.0);
        let Some((_, w)) = self.mondo.stanza.modulo else { return };
        let m = &mut self.moduli[self.stanza];
        if moto.sforzo > 0.0 {
            let prima = *m;
            *m = (*m + CRESCITA * DT).min(MAX_MODULI);
            self.sforzo_totale += *m - prima;
        } else {
            let percorso = self.giocatore.vel.x.abs() * DT;
            *m = (*m - percorso / (w * TILE) as f32 * RITIRO).max(0.0);
        }
        let n = self.moduli[self.stanza].floor() as i32;
        if n != self.mondo.moduli {
            self.ridimensiona(n);
        }
    }

    /// La Sala commenta quello che fa il corpo.
    fn commenti(&mut self, moto: &Moto) {
        self.t_muro = if moto.contro_muro { self.t_muro + DT } else { 0.0 };
        if self.t_muro > 1.0 {
            self.sala.di(Battuta::Muro);
        }
        let g = &self.giocatore;
        if g.in_acqua && g.rilassato {
            let sup = self.mondo.superficie(g.rect().center());
            if g.pos.y > sup + 14.0 {
                self.sala.di(Battuta::Affonda);
            }
            // Vicino all'arco.
            if self.stanza == RIVA_I && g.pos.x < (self.mondo.x_mondo(8) * TILE) as f32 {
                self.sala.di(Battuta::Arco);
            }
        }
        if moto.portato == Some(Fluido::Aria) && g.vel.y < -40.0 {
            self.sala.di(Battuta::Vento);
        }
    }

    /// Scritte: si leggono stando vicini per un attimo.
    fn leggi(&mut self) {
        let c = self.giocatore.rect().center();
        let mut vicina = None;
        for (i, s) in self.mondo.stanza.scritte.iter().enumerate() {
            let r = render::rect_scritta(&self.mondo, s);
            let d = render::distanza_rect(c, &r);
            if d < 28.0 {
                vicina = Some(i);
            }
        }
        self.lettura = match (vicina, self.lettura) {
            (Some(i), Some((j, t))) if i == j => Some((i, t + DT)),
            (Some(i), _) => Some((i, 0.0)),
            (None, _) => None,
        };
        if let Some((i, t)) = self.lettura {
            if t > 0.6 {
                self.lette[self.stanza] |= 1 << i;
                if (self.stanza, i) == INDIZIO {
                    self.ha_capito = true;
                }
            }
        }
    }

    fn particelle_del_corpo(&mut self) {
        let g = &self.giocatore;
        let c = g.rect().center();
        if self.foga > 0.5 && self.rng.f32() < 0.5 {
            let pos = c + Vec2::new(self.rng.range(-3.0, 3.0), self.rng.range(-6.0, 0.0));
            let vel = Vec2::new(self.rng.range(-20.0, 20.0), self.rng.range(-40.0, -10.0));
            self.particelle.spawn(Particella { pos, vel, vita: 0.4, durata: 0.4, colore: rgb(255, 170, 80), gravita: 120.0 });
        }
        if g.in_acqua && self.rng.f32() < 0.08 {
            let pos = Vec2::new(c.x + self.rng.range(-2.0, 2.0), g.pos.y);
            let vita = self.rng.range(0.8, 1.6);
            self.particelle.spawn(Particella {
                pos,
                vel: Vec2::new(0.0, -18.0),
                vita,
                durata: vita,
                colore: rgb(150, 200, 230),
                gravita: -6.0,
            });
        }
    }
}

impl Default for Egresso {
    fn default() -> Self {
        Self::new()
    }
}

impl Game for Egresso {
    fn update(&mut self, input: &Input) {
        self.t += DT;
        if input.pressed(Key::F3) {
            self.debug = !self.debug;
        }
        self.sala.update(DT);
        self.particelle.update(DT);
        self.scossa = (self.scossa - DT).max(0.0);

        if let Scena::Uscito(t) = self.scena {
            self.scena = Scena::Uscito(t + DT);
            if t > 2.5 && (input.pressed(Key::Enter) || input.pressed(Key::Space)) {
                *self = Egresso::new();
            }
            return;
        }

        self.t_stanza += DT;
        if self.t > 1.5 && self.stanza == RIVA_I {
            self.sala.di(Battuta::Benvenuto);
        }

        let moto = self.giocatore.update(input, &self.mondo, self.t);
        match moto.esito {
            Esito::Varco => {
                self.scena = Scena::Uscito(0.0);
                return;
            }
            Esito::Bordo(d) => {
                let st = self.mondo.stanza;
                let dest = if d < 0 { st.sinistra } else { st.destra };
                if let Some(i) = dest {
                    self.vai_stanza(i, -d);
                    return;
                }
            }
            Esito::Niente => {}
        }
        self.sforzo(&moto);
        self.commenti(&moto);
        self.leggi();
        self.particelle_del_corpo();

        let obiettivo = self.cam_obiettivo();
        self.cam += (obiettivo - self.cam) * (5.0 * DT).min(1.0);
    }

    fn render(&mut self, fb: &mut Framebuffer, stats: &Stats) {
        self.disegna(fb, stats);
    }

    fn comandi(&self) -> u32 {
        // Sul telefono: solo muoversi e saltare.
        0b11
    }

    fn traccia(&self) -> String {
        let g = &self.giocatore;
        format!(
            "stanza {} pos {:6.1} {:6.1} vel {:6.1} {:6.1} terra {} acqua {} moduli {:?} foga {:.2}",
            self.stanza,
            g.pos.x,
            g.pos.y,
            g.vel.x,
            g.vel.y,
            g.a_terra as u8,
            g.in_acqua as u8,
            self.moduli.map(|m| (m * 10.0).round() / 10.0),
            self.foga
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Gioca `secondi` tenendo premuti `tasti`.
    fn gioca(g: &mut Egresso, tasti: &[Key], secondi: f32) {
        let mut input = Input::new();
        for &k in tasti {
            input.set(k, true);
        }
        for _ in 0..(secondi / DT) as usize {
            g.update(&input);
            input.fine_passo();
            if g.scena != Scena::Gioco {
                return;
            }
        }
    }

    /// Gioca finché `fatto` non è vero (al massimo `secondi`).
    fn fino_a(g: &mut Egresso, tasti: &[Key], secondi: f32, fatto: impl Fn(&Egresso) -> bool) -> bool {
        let mut input = Input::new();
        for &k in tasti {
            input.set(k, true);
        }
        for _ in 0..(secondi / DT) as usize {
            if fatto(g) {
                return true;
            }
            g.update(&input);
            input.fine_passo();
        }
        fatto(g)
    }

    #[test]
    fn si_comincia_sulla_riva_con_la_sala_gia_lunga() {
        let g = Egresso::new();
        assert_eq!(g.stanza, RIVA_I);
        assert_eq!(g.mondo.moduli, 8);
        assert!(g.cam > 0.0, "l'arco è fuori dallo schermo");
    }

    #[test]
    fn camminando_a_destra_si_arriva_all_atrio() {
        let mut g = Egresso::new();
        assert!(fino_a(&mut g, &[Key::Right], 5.0, |g| g.stanza == ATRIO_I));
    }

    #[test]
    fn forzare_contro_la_raffica_allunga_l_atrio() {
        let mut g = Egresso::new();
        g.vai_stanza(ATRIO_I, -1);
        // Tiene premuto a destra per 10 secondi, raffiche comprese.
        gioca(&mut g, &[Key::Right], 10.0);
        assert!(g.mondo.moduli > 0, "moduli: {}", g.mondo.moduli);
    }

    #[test]
    fn camminando_tra_le_raffiche_l_atrio_si_attraversa() {
        let mut g = Egresso::new();
        g.vai_stanza(ATRIO_I, -1);
        let mut input = Input::new();
        let d = &stanze::ATRIO.correnti[0];
        for _ in 0..(40.0 / DT) as usize {
            // Cammina solo quando non soffia e non sta per soffiare; se no si ripara.
            let x = g.giocatore.rect().center().x;
            let (tx, ty) = Mondo::tile(Vec2::new(x, 90.0));
            let calmo = !mondo::attiva(d, g.t) && mondo::preavviso(d, g.t) == 0.0;
            let riparato = g.mondo.riparato(tx, ty, d) && g.mondo.solido(tx + 1, ty);
            let vai = calmo || !riparato && !mondo::attiva(d, g.t);
            input.set(Key::Right, vai);
            // Davanti a un blocco si salta (tenendo premuto, per saltare alto).
            let blocco = g.mondo.solido(Mondo::tile(Vec2::new(x + 6.0, 90.0)).0, ty);
            input.set(Key::Space, vai && (blocco || !g.giocatore.a_terra));
            g.update(&input);
            input.fine_passo();
            if g.stanza == GALLERIA_I {
                break;
            }
        }
        assert_eq!(g.stanza, GALLERIA_I);
        assert_eq!(g.moduli[ATRIO_I].floor(), 0.0, "l'Atrio non è cresciuto");
    }

    #[test]
    fn vicino_alla_riva_l_acqua_e_ferma() {
        let mut g = Egresso::new();
        assert!(fino_a(&mut g, &[Key::Left], 6.0, |g| g.giocatore.in_acqua));
        gioca(&mut g, &[], 20.0);
        assert_eq!(g.scena, Scena::Gioco);
        assert!(g.giocatore.in_acqua);
    }

    #[test]
    fn dalla_riva_l_aria_porta_sulla_mensola() {
        let mut g = Egresso::new();
        // Camminare piano accorcia la Riva: la colonna va ricalcolata a ogni passo.
        assert!(fino_a(&mut g, &[Key::Left], 6.0, |g| g.giocatore.pos.x < (g.mondo.x_mondo(26) * TILE) as f32 + 6.0));
        // Fermo nella colonna d'aria: sale.
        assert!(fino_a(&mut g, &[], 4.0, |g| g.giocatore.pos.y < 30.0));
        // A sinistra, sulla mensola.
        assert!(fino_a(&mut g, &[Key::Left], 3.0, |g| g.giocatore.a_terra && g.giocatore.pos.y == 48.0 - giocatore::ALT));
    }

    #[test]
    fn lasciandosi_portare_dal_canale_si_esce() {
        let mut g = Egresso::new();
        // Sulla mensola, cammina a sinistra fino a cadere nel canale...
        g.giocatore.pos = Vec2::new((g.mondo.x_mondo(14) * TILE) as f32, 48.0 - giocatore::ALT);
        assert!(fino_a(&mut g, &[Key::Left], 10.0, |g| g.giocatore.in_acqua));
        // ...e non tocca più niente.
        assert!(fino_a(&mut g, &[], 40.0, |g| matches!(g.scena, Scena::Uscito(_))));
    }

    #[test]
    fn nuotando_verso_l_arco_non_si_esce() {
        let mut g = Egresso::new();
        assert!(fino_a(&mut g, &[Key::Left], 6.0, |g| g.giocatore.in_acqua));
        gioca(&mut g, &[Key::Left], 30.0);
        assert_eq!(g.scena, Scena::Gioco);
    }

    #[test]
    fn leggere_la_nuotatrice_fa_capire() {
        let mut g = Egresso::new();
        g.vai_stanza(GALLERIA_I, -1);
        let s = &stanze::GALLERIA.scritte[INDIZIO.1];
        g.giocatore.pos = Vec2::new((s.x * TILE + 20) as f32, 28.0);
        gioca(&mut g, &[], 1.5);
        assert!(g.ha_capito);
    }
}
