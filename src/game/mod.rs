//! EGRESSO, il prototipo: l'Atrio (dove si impara a nominare) e l'Abisso
//! (tre voci, un abisso che forse non lo è, una porta e due finali).
//!
//! Comandi: frecce/A-D muovi, Spazio/W salta, E afferma, Q cambia voce,
//! R lascia andare, F3 debug.

pub mod giocatore;
pub mod lessico;
pub mod livello;
pub mod mondo;
pub mod particelle;
mod render;
pub mod sala;

use crate::engine::app::{Game, Stats, ALTEZZA, LARGHEZZA};
use crate::engine::font::Font;
use crate::engine::framebuffer::Framebuffer;
use crate::engine::glow::Glow;
use crate::engine::input::{Input, Key};
use crate::engine::math::{Rect, Rng, Vec2};
use crate::engine::time::DT;

use giocatore::{Esito, Giocatore};
use lessico::{Natura, Voce};
use livello::{COLONNE, STANZE, TILE};
use mondo::{distanza_rect, Evento, Fase, Mondo, Reale};
use particelle::{Particella, Particelle};
use sala::{Battuta, Sala};

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
    /// Indice in `STANZE` della stanza attuale.
    pub stanza: usize,
    /// Secondi da quando si è entrati nella stanza (per il suo nome a schermo).
    pub t_stanza: f32,
    /// Quante volte sono stati usati E, Q, R (i suggerimenti si spengono).
    pub usati: [u8; 3],
    pub cadute: u32,
    pub ritorni_all_atrio: u32,
    /// Percezione appena affermata e secondi di lampo rimasti.
    pub lampo: Option<(usize, f32)>,
    /// La voce del dungeon.
    pub sala: Sala,
    /// Secondi senza toccare niente.
    pub fermo: f32,
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
        let mondo = Mondo::new(STANZE[0]);
        let (realta, realta_luce) = render::genera_realta(&mondo);
        let mut rng = Rng::new(0x5eed);
        let polvere =
            (0..90).map(|_| ((rng.f32() * LARGHEZZA as f32) as i32, (rng.f32() * 130.0) as i32, (10.0 + rng.f32() * 22.0) as u8)).collect();
        Egresso {
            mondo,
            giocatore: Giocatore::new(STANZE[0].partenza),
            particelle: Particelle::new(),
            scena: Scena::Gioco,
            focus: None,
            messaggio: None,
            biglietto: None,
            debug: false,
            t: 0.0,
            stanza: 0,
            t_stanza: 0.0,
            usati: [0; 3],
            cadute: 0,
            ritorni_all_atrio: 0,
            lampo: None,
            sala: Sala::new(),
            fermo: 0.0,
            rng,
            font: Font::new(),
            luce: Framebuffer::new(LARGHEZZA, ALTEZZA),
            glow: Glow::new(LARGHEZZA, ALTEZZA),
            realta,
            realta_luce,
            polvere,
        }
    }

    /// Entra nella stanza `i` (dall'inizio). La texture della realtà viene
    /// ridipinta: è l'unica allocazione, e succede solo al cambio di stanza.
    fn vai_stanza(&mut self, i: usize) {
        let stanza = STANZE[i];
        self.stanza = i;
        self.t_stanza = 0.0;
        self.mondo = Mondo::new(stanza);
        let (realta, luce) = render::genera_realta(&self.mondo);
        self.realta = realta;
        self.realta_luce = luce;
        self.giocatore = Giocatore::new(stanza.partenza);
        self.particelle.clear();
        self.focus = None;
        self.lampo = None;
        self.scena = Scena::Gioco;
        match i {
            0 if self.ritorni_all_atrio > 0 => self.sala.di(Battuta::RitornoAllAtrio),
            1 => self.sala.di(Battuta::Abisso),
            _ => {}
        }
    }

    /// Il protagonista è precipitato: riparte, e una voce commenta.
    fn caduto(&mut self) {
        let x = self.giocatore.rect().center().x;
        // Era sopra una parola solo proposta?
        let su_proposta = self.mondo.percetti.iter().any(|p| p.fase == Fase::Proposto && p.rect().x <= x && x <= p.rect().right());
        self.cadute += 1;
        if self.cadute == 1 {
            self.dici(Voce::Controllo, "QUELLO CHE NON DICO, NON C'È.");
            self.sala.di(Battuta::PrimaCaduta);
        } else if su_proposta {
            self.dici(Voce::Controllo, "NON L'AVEVO DETTO.");
        } else {
            self.dici(Voce::Paura, "VISTO? BURRONE.");
        }
        self.giocatore.riparti();
    }

    /// Le lettere solo proposte si sbriciolano sotto i piedi di chi ci passa.
    fn sbriciola_proposte(&mut self) {
        let g = self.giocatore.rect();
        let t = TILE as f32;
        let ty = ((g.bottom() + 1.0) / t).floor() as i32;
        let (x0, x1) = ((g.x / t).floor() as i32, ((g.right() - 0.01) / t).floor() as i32);
        for tx in x0..=x1 {
            if let Some((_, voce)) = self.mondo.proposta_in(tx, ty) {
                if self.rng.f32() < 0.5 {
                    let pos = Vec2::new((tx * TILE) as f32 + self.rng.range(0.0, 7.0), (ty * TILE) as f32 + self.rng.range(0.0, 5.0));
                    let vel = Vec2::new(self.rng.range(-6.0, 6.0), self.rng.range(10.0, 30.0));
                    let vita = self.rng.range(0.5, 1.0);
                    self.particelle.spawn(Particella {
                        pos,
                        vel,
                        vita,
                        durata: vita,
                        colore: voce.colore(),
                        lettera: ' ',
                        gravita: 200.0,
                        inclina: false,
                    });
                }
            }
        }
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
                    self.particelle.spawn(Particella {
                        pos,
                        vel,
                        vita: 6.0,
                        durata: 6.0,
                        colore: voce.colore(),
                        lettera,
                        gravita: 300.0,
                        inclina: voce == Voce::Memoria,
                    });
                }
                Evento::LetteraSciolta { pos, voce, .. } => {
                    for _ in 0..7 {
                        let p = pos + Vec2::new(self.rng.range(0.0, 5.0), self.rng.range(0.0, 7.0));
                        let vel = Vec2::new(self.rng.range(-18.0, 18.0), self.rng.range(-45.0, -8.0));
                        let vita = self.rng.range(0.6, 1.6);
                        self.particelle.spawn(Particella {
                            pos: p,
                            vel,
                            vita,
                            durata: vita,
                            colore: voce.colore(),
                            lettera: ' ',
                            gravita: -10.0,
                            inclina: false,
                        });
                    }
                }
                Evento::Mutazione { percetto } => {
                    // La Sala commenta i nomi che marciscono.
                    let p = &self.mondo.percetti[percetto as usize];
                    let pr = &p.def.proposte[p.scelta];
                    let natura = pr.lemma.stati[p.stato].natura;
                    if std::ptr::eq(pr.lemma, &lessico::BURRONE) && natura == Natura::Scivolosa {
                        self.sala.di(Battuta::Burro);
                    } else if natura == Natura::Vuota {
                        self.sala.di(Battuta::NomeMarcito);
                    }
                }
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
                let colore = if self.rng.f32() < 0.6 {
                    crate::engine::framebuffer::rgb(255, 228, 180)
                } else {
                    crate::engine::framebuffer::rgb(170, 235, 255)
                };
                let vita = self.rng.range(0.3, 0.8);
                self.particelle.spawn(Particella {
                    pos,
                    vel: Vec2::new(0.0, -8.0),
                    vita,
                    durata: vita,
                    colore,
                    lettera: ' ',
                    gravita: 0.0,
                    inclina: false,
                });
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
        self.sala.update(DT);

        match self.scena {
            Scena::Atrio(t) => {
                self.scena = Scena::Atrio(t + DT);
                if t > 2.8 {
                    self.vai_stanza(0);
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

        self.t_stanza += DT;
        if self.t > 3.2 && self.stanza == 0 {
            self.sala.di(Battuta::Benvenuto);
        }
        // Chi non fa niente, a volte, fa la cosa giusta.
        let attivo =
            input.any_pressed() || [Key::Left, Key::Right, Key::A, Key::D, Key::Space, Key::W, Key::Up].iter().any(|&k| input.down(k));
        self.fermo = if attivo { 0.0 } else { self.fermo + DT };
        if self.fermo > 8.0 && self.stanza > 0 {
            self.sala.di(Battuta::Fermo);
        }
        if let Some((i, t)) = self.lampo {
            self.lampo = if t - DT > 0.0 { Some((i, t - DT)) } else { None };
        }

        if self.giocatore.update(input, &mut self.mondo, true) == Esito::Caduto {
            self.caduto();
        }
        self.sbriciola_proposte();

        // Oltre il bordo destro: la stanza successiva.
        let max_x = (COLONNE * TILE) as f32 - giocatore::LARG;
        if self.mondo.stanza.uscita_destra && self.giocatore.pos.x >= max_x - 0.5 && self.stanza + 1 < STANZE.len() {
            self.vai_stanza(self.stanza + 1);
            return;
        }

        self.focus = self.scegli_focus();
        if let Some(i) = self.focus {
            if input.pressed(Key::Q) {
                self.mondo.prossima_voce(i);
                self.usati[1] = self.usati[1].saturating_add(1);
            }
            if input.pressed(Key::E) && self.mondo.percetti[i].fase == Fase::Proposto {
                if self.mondo.afferma(i, self.giocatore.rect()) {
                    self.usati[0] = self.usati[0].saturating_add(1);
                    self.lampo = Some((i, 0.35));
                    self.sala.di(Battuta::PrimoNome);
                    self.scintille_su(i, 3);
                } else {
                    self.dici(Voce::Controllo, "NON QUI. CI SONO IO.");
                }
            }
            if input.pressed(Key::R) {
                self.mondo.lascia(i);
                self.sala.di(Battuta::PrimoLasciare);
                self.usati[2] = self.usati[2].saturating_add(1);
            }
        }

        self.mondo.update(DT, self.giocatore.rect());
        self.gestisci_eventi();
        self.bordo_rivelazioni();
        self.contatti();

        let cx = self.giocatore.rect().center().x;
        self.biglietto = self.mondo.stanza.biglietti.iter().position(|(col, _)| (cx - (*col * TILE + 4) as f32).abs() < 14.0);
        self.particelle.update(DT, &self.mondo);
    }

    fn traccia(&self) -> String {
        let g = &self.giocatore;
        let fasi: Vec<String> = self.mondo.percetti.iter().map(|p| format!("{:?}", p.fase).chars().take(4).collect()).collect();
        format!(
            "stanza {} pos {:6.1} {:6.1} vel {:6.1} {:6.1} terra {} focus {:?} {:?} {}",
            self.stanza,
            g.pos.x,
            g.pos.y,
            g.vel.x,
            g.vel.y,
            g.a_terra as u8,
            self.focus,
            self.scena,
            fasi.join(",")
        )
    }

    fn render(&mut self, fb: &mut Framebuffer, stats: &Stats) {
        self.disegna(fb, stats);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Simula `passi` update con i tasti `giu` tenuti premuti; `tocca` viene
    /// premuto a ogni passo (e rilasciato subito).
    fn gioca(g: &mut Egresso, passi: usize, giu: &[Key], tocca: Option<Key>) {
        let mut input = Input::new();
        for _ in 0..passi {
            for &k in giu {
                input.set(k, true);
            }
            if let Some(k) = tocca {
                input.set(k, true);
            }
            g.update(&input);
            input.fine_passo();
            if let Some(k) = tocca {
                input.set(k, false);
            }
        }
    }

    #[test]
    fn nell_atrio_si_cade_nel_vuoto_non_detto() {
        let mut g = Egresso::new();
        gioca(&mut g, 400, &[Key::Right], None);
        assert_eq!(g.stanza, 0, "senza dire PONTE non si passa");
        assert!(g.cadute >= 1);
    }

    #[test]
    fn dicendo_ponte_si_arriva_all_abisso() {
        let mut g = Egresso::new();
        // Fino al bordo del vuoto, poi aspetta le proposte.
        while g.giocatore.pos.x < 92.0 {
            gioca(&mut g, 1, &[Key::Right], None);
        }
        gioca(&mut g, 90, &[], None);
        assert_eq!(g.mondo.percetti[2].fase, Fase::Proposto);
        gioca(&mut g, 1, &[], Some(Key::E));
        assert_eq!(g.mondo.percetti[2].fase, Fase::Affermato);
        for _ in 0..400 {
            if g.stanza == 1 {
                break;
            }
            gioca(&mut g, 1, &[Key::Right], None);
        }
        assert_eq!(g.stanza, 1, "oltre il bordo destro c'è l'Abisso");
        assert_eq!(g.cadute, 0, "il PONTE detto regge");
    }

    #[test]
    fn la_porta_nominata_riporta_all_atrio() {
        let mut g = Egresso::new();
        g.vai_stanza(1);
        g.giocatore.pos.x = 280.0;
        gioca(&mut g, 90, &[], None);
        assert_eq!(g.mondo.percetti[4].fase, Fase::Proposto);
        gioca(&mut g, 1, &[], Some(Key::E));
        gioca(&mut g, 60, &[Key::Right], None);
        assert!(matches!(g.scena, Scena::Atrio(_)));
        gioca(&mut g, 200, &[], None);
        assert_eq!(g.stanza, 0);
        assert_eq!(g.scena, Scena::Gioco);
    }
}
