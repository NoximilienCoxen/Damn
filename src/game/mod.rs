//! EGRESSO, il prototipo: una stanza, tre voci, un abisso che forse non lo è.
//!
//! Comandi: frecce/A-D muovi, Spazio/W salta, E afferma, Q cambia voce,
//! R lascia andare, F3 debug.

pub mod giocatore;
pub mod lessico;
pub mod livello;
pub mod mondo;
pub mod particelle;
mod render;

use crate::engine::app::{Game, Stats, ALTEZZA, LARGHEZZA};
use crate::engine::font::Font;
use crate::engine::framebuffer::Framebuffer;
use crate::engine::glow::Glow;
use crate::engine::input::{Input, Key};
use crate::engine::math::{Rect, Rng, Vec2};
use crate::engine::time::DT;

use giocatore::{Esito, Giocatore};
use lessico::{Natura, Voce};
use livello::{BIGLIETTI, TILE};
use mondo::{distanza_rect, Evento, Fase, Mondo, Reale};
use particelle::{Particella, Particelle};

/// Distanza massima (px) per mettere a fuoco una percezione.
const DIST_FOCUS: f32 = 30.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Scena {
    Gioco,
    /// Hai attraversato una PORTA nominata: si torna all'Atrio (secondi).
    Atrio(f32),
    /// Sei uscito davvero (secondi).
    Uscito(f32),
}

pub struct Messaggio {
    pub testo: &'static str,
    pub voce: Voce,
    pub t: f32,
}

pub struct Egresso {
    pub mondo: Mondo,
    pub giocatore: Giocatore,
    pub particelle: Particelle,
    pub scena: Scena,
    pub focus: Option<usize>,
    pub messaggio: Option<Messaggio>,
    pub biglietto: Option<usize>,
    pub debug: bool,
    pub t: f32,
    /// Quali comandi sono già stati usati (per far sparire i suggerimenti).
    pub usati: [bool; 3],
    pub ritorni_all_atrio: u32,
    rng: Rng,
    font: Font,
    luce: Framebuffer,
    glow: Glow,
    realta: Vec<u32>,
    realta_luce: Vec<u32>,
    polvere: Vec<(i32, i32, u8)>,
}

impl Egresso {
    pub fn new() -> Egresso {
        let mondo = Mondo::new();
        let (realta, realta_luce) = render::genera_realta(&mondo);
        let mut rng = Rng::new(0x5eed);
        let polvere = (0..90)
            .map(|_| ((rng.f32() * LARGHEZZA as f32) as i32, (rng.f32() * 130.0) as i32, (10.0 + rng.f32() * 22.0) as u8))
            .collect();
        Egresso {
            mondo,
            giocatore: Giocatore::new(),
            particelle: Particelle::new(),
            scena: Scena::Gioco,
            focus: None,
            messaggio: None,
            biglietto: None,
            debug: false,
            t: 0.0,
            usati: [false; 3],
            ritorni_all_atrio: 0,
            rng,
            font: Font::new(),
            luce: Framebuffer::new(LARGHEZZA, ALTEZZA),
            glow: Glow::new(LARGHEZZA, ALTEZZA),
            realta,
            realta_luce,
            polvere,
        }
    }

    fn ricomincia_stanza(&mut self) {
        self.mondo = Mondo::new();
        self.giocatore = Giocatore::new();
        self.particelle.clear();
        self.focus = None;
        self.scena = Scena::Gioco;
    }

    fn dici(&mut self, voce: Voce, testo: &'static str) {
        self.messaggio = Some(Messaggio { testo, voce, t: 0.0 });
    }

    /// La percezione su cui agiscono E, Q, R: prima quelle con proposte
    /// aperte, poi le parole nominate da te, infine quelle "di partenza".
    fn scegli_focus(&self) -> Option<usize> {
        let c = self.giocatore.rect().center();
        let mut migliore: Option<(u8, f32, usize)> = None;
        for (i, p) in self.mondo.percetti.iter().enumerate() {
            let classe = match p.fase {
                Fase::Proposto => 0,
                Fase::Affermato if p.def.iniziale.is_none() => 1,
                Fase::Affermato => 2,
                _ => continue,
            };
            let d = distanza_rect(c, &p.rect());
            if d > DIST_FOCUS {
                continue;
            }
            if migliore.is_none_or(|(mc, md, _)| (classe, d) < (mc, md)) {
                migliore = Some((classe, d, i));
            }
        }
        migliore.map(|(_, _, i)| i)
    }

    fn scintille_su(&mut self, i: usize, quante: usize) {
        let p = &self.mondo.percetti[i];
        let pr = &p.def.proposte[p.scelta];
        let colore = pr.voce.colore();
        for k in 0..pr.celle {
            let (tx, ty) = pr.cella(k);
            for _ in 0..quante {
                let pos = Vec2::new((tx * TILE) as f32 + self.rng.range(0.0, 8.0), (ty * TILE) as f32 + self.rng.range(0.0, 8.0));
                let vel = Vec2::new(self.rng.range(-12.0, 12.0), self.rng.range(-30.0, -5.0));
                self.particelle.spawn(Particella { pos, vel, vita: 0.5, durata: 0.5, colore, lettera: ' ', gravita: 0.0, inclina: false });
            }
        }
    }

    fn gestisci_eventi(&mut self) {
        for e in self.mondo.eventi.drain(..) {
            match e {
                Evento::LetteraCaduta { pos, lettera, voce } => {
                    let vel = Vec2::new(self.rng.range(-15.0, 15.0), self.rng.range(-40.0, -10.0));
                    self.particelle.spawn(Particella { pos, vel, vita: 6.0, durata: 6.0, colore: voce.colore(), lettera, gravita: 300.0, inclina: voce == Voce::Memoria });
                }
                Evento::LetteraSciolta { pos, voce, .. } => {
                    for _ in 0..7 {
                        let p = pos + Vec2::new(self.rng.range(0.0, 5.0), self.rng.range(0.0, 7.0));
                        let vel = Vec2::new(self.rng.range(-18.0, 18.0), self.rng.range(-45.0, -8.0));
                        let vita = self.rng.range(0.6, 1.6);
                        self.particelle.spawn(Particella { pos: p, vel, vita, durata: vita, colore: voce.colore(), lettera: ' ', gravita: -10.0, inclina: false });
                    }
                }
                Evento::Mutazione { .. } => {}
            }
        }
    }

    /// Scintille sul bordo delle rivelazioni aperte.
    fn bordo_rivelazioni(&mut self) {
        for k in 0..self.mondo.rivelazioni.len() {
            let r = self.mondo.rivelazioni[k];
            let raggio = r.raggio();
            if raggio < 4.0 {
                continue;
            }
            for _ in 0..2 {
                let a = self.rng.range(0.0, std::f32::consts::TAU);
                let pos = r.centro + Vec2::new(a.cos(), a.sin()) * (raggio + self.rng.range(-2.0, 3.0));
                let colore = if self.rng.f32() < 0.6 { crate::engine::framebuffer::rgb(255, 228, 180) } else { crate::engine::framebuffer::rgb(170, 235, 255) };
                let vita = self.rng.range(0.3, 0.8);
                self.particelle.spawn(Particella { pos, vel: Vec2::new(0.0, -8.0), vita, durata: vita, colore, lettera: ' ', gravita: 0.0, inclina: false });
            }
        }
    }

    /// Cosa sta toccando il corpo: porte, trappole, il varco.
    fn contatti(&mut self) {
        let corpo = self.giocatore.rect();
        let t = TILE as f32;
        let x0 = (corpo.x / t).floor() as i32;
        let x1 = ((corpo.right() + 1.0) / t).floor() as i32;
        let y0 = (corpo.y / t).floor() as i32;
        let y1 = ((corpo.bottom() - 0.5) / t).floor() as i32;
        for ty in y0..=y1 {
            for tx in x0..=x1 {
                let cella = Rect::new(tx as f32 * t, ty as f32 * t, t, t);
                if !cella.expand(0.5).overlaps(&corpo) {
                    continue;
                }
                if let Some(c) = self.mondo.cella(tx, ty) {
                    match c.natura {
                        Natura::Porta => {
                            self.scena = Scena::Atrio(0.0);
                            self.ritorni_all_atrio += 1;
                            return;
                        }
                        Natura::Trappola => {
                            self.giocatore.riparti();
                            self.dici(Voce::Paura, "TE L'AVEVO DETTO.");
                            return;
                        }
                        _ => {}
                    }
                }
                if self.mondo.reale(tx, ty) == Reale::Varco && self.mondo.rivelato(cella.center()) > 0.5 {
                    self.scena = Scena::Uscito(0.0);
                    return;
                }
            }
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
        if let Some(m) = &mut self.messaggio {
            m.t += DT;
            if m.t > 2.5 {
                self.messaggio = None;
            }
        }

        match self.scena {
            Scena::Atrio(t) => {
                self.scena = Scena::Atrio(t + DT);
                if t > 2.8 {
                    self.ricomincia_stanza();
                }
                return;
            }
            Scena::Uscito(t) => {
                self.scena = Scena::Uscito(t + DT);
                if t > 2.0 && (input.pressed(Key::Enter) || input.pressed(Key::Space)) {
                    *self = Egresso::new();
                }
                return;
            }
            Scena::Gioco => {}
        }

        if self.giocatore.update(input, &mut self.mondo, true) == Esito::Caduto {
            self.giocatore.riparti();
            self.dici(Voce::Paura, "VISTO? BURRONE.");
        }

        self.focus = self.scegli_focus();
        if let Some(i) = self.focus {
            if input.pressed(Key::Q) {
                self.mondo.prossima_voce(i);
                self.usati[1] = true;
            }
            if input.pressed(Key::E) && self.mondo.percetti[i].fase == Fase::Proposto {
                if self.mondo.afferma(i, self.giocatore.rect()) {
                    self.usati[0] = true;
                    self.scintille_su(i, 3);
                } else {
                    self.dici(Voce::Controllo, "NON QUI. CI SONO IO.");
                }
            }
            if input.pressed(Key::R) {
                self.mondo.lascia(i);
                self.usati[2] = true;
            }
        }

        self.mondo.update(DT, self.giocatore.rect());
        self.gestisci_eventi();
        self.bordo_rivelazioni();
        self.contatti();

        let cx = self.giocatore.rect().center().x;
        self.biglietto = BIGLIETTI.iter().position(|(col, _)| (cx - (*col * TILE + 4) as f32).abs() < 14.0);
        self.particelle.update(DT, &self.mondo);
    }

    fn traccia(&self) -> String {
        let g = &self.giocatore;
        let fasi: Vec<String> = self.mondo.percetti.iter().map(|p| format!("{:?}", p.fase).chars().take(4).collect()).collect();
        format!("pos {:6.1} {:6.1} vel {:6.1} {:6.1} terra {} focus {:?} {:?} {}", g.pos.x, g.pos.y, g.vel.x, g.vel.y, g.a_terra as u8, self.focus, self.scena, fasi.join(","))
    }

    fn render(&mut self, fb: &mut Framebuffer, stats: &Stats) {
        self.disegna(fb, stats);
    }
}
