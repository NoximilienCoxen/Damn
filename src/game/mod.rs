//! EGRESSO: la scoperta.
//!
//! Fuori, di notte. Una terra di colline che gira su se stessa, e da
//! qualche parte (ogni partita altrove) Egresso: una facciata di pietra in
//! una collina, con un arco buio. Nessuna freccia, nessun testo: ci si
//! arriva esplorando, e il mondo tira da quella parte (il vento che piega
//! l'erba verso l'arco, le lucciole più fitte, le rovine con le lettere).
//! Il nome del gioco compare solo quando lo trovi. Cosa c'è dentro è ancora
//! da inventare (vedi `docs/CONCEPT.md`).
//!
//! Comandi: frecce/A-D muovi, Spazio/W salta, F3 debug.

pub mod giocatore;
pub mod natura;
pub mod particelle;
mod render;
pub mod terreno;

use crate::engine::app::{Game, Stats, ALTEZZA, LARGHEZZA};
use crate::engine::font::Font;
use crate::engine::framebuffer::{rgb, Framebuffer};
use crate::engine::glow::Glow;
use crate::engine::input::{Input, Key};
use crate::engine::math::{hash01, Rng, Vec2};
use crate::engine::time::DT;

use giocatore::Giocatore;
use natura::Lucciole;
use particelle::{Particella, Particelle};
use terreno::{delta, Terreno, NOME};

/// Da quanto vicino all'arco la telecamera comincia a inquadrarlo.
const INQUADRA: f32 = 190.0;
/// Quanto bisogna stare davanti all'arco per essere presi dal buio.
const SOGLIA: f32 = 0.25;
const DURATA_SOGLIA: f32 = 3.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Scena {
    Fuori,
    /// Il buio dell'arco ti prende (secondi).
    Soglia(f32),
    /// Dentro (secondi).
    Dentro(f32),
}

pub struct Egresso {
    pub terreno: Terreno,
    pub giocatore: Giocatore,
    pub lucciole: Lucciole,
    pub foglie: Particelle,
    pub scena: Scena,
    pub t: f32,
    pub cam: f32,
    /// Secondi senza toccare niente.
    pub fermo: f32,
    /// Secondi passati davanti all'arco.
    davanti: f32,
    /// La più grande vicinanza raggiunta: le lettere accese restano accese.
    pub lettere: f32,
    pub debug: bool,
    rng: Rng,
    font: Font,
    luce: Framebuffer,
    glow: Glow,
}

impl Egresso {
    pub fn new() -> Egresso {
        Egresso::con_seme(1)
    }

    pub fn con_seme(seme: u32) -> Egresso {
        let terreno = Terreno::new(seme);
        let giocatore = Giocatore::new(terreno.partenza, &terreno);
        let mut rng = Rng::new(seme as u64 ^ 0x5eed_0000);
        let lucciole = Lucciole::new(&mut rng, terreno.partenza, &terreno);
        let mut g = Egresso {
            terreno,
            giocatore,
            lucciole,
            foglie: Particelle::new(),
            scena: Scena::Fuori,
            t: 0.0,
            cam: 0.0,
            fermo: 0.0,
            davanti: 0.0,
            lettere: 0.0,
            debug: false,
            rng,
            font: Font::new(),
            luce: Framebuffer::new(LARGHEZZA, ALTEZZA),
            glow: Glow::new(LARGHEZZA, ALTEZZA),
        };
        g.cam = g.cam_obiettivo();
        g
    }

    /// Distanza con segno dal protagonista all'arco.
    pub fn all_arco(&self) -> f32 {
        delta(self.giocatore.centro_x(), self.terreno.arco)
    }

    fn cam_obiettivo(&self) -> f32 {
        let c = self.giocatore.centro_x();
        let d = self.all_arco();
        // Vicino all'arco, l'inquadratura si sposta piano su di lui.
        let k = ((INQUADRA - d.abs()) / 120.0).clamp(0.0, 1.0) * 0.6;
        c + d * k - LARGHEZZA as f32 * 0.5
    }

    fn foglie_nel_vento(&mut self) {
        let c = self.giocatore.centro_x();
        let v = self.terreno.vento(c, self.t);
        if self.rng.f32() > v.abs() * 0.35 {
            return;
        }
        // Nascono dalla parte da cui viene il vento, fuori dallo schermo.
        let x = c - v.signum() * self.rng.range(150.0, 190.0);
        let y = self.terreno.suolo(x) - self.rng.range(4.0, 60.0);
        let vita = self.rng.range(3.0, 5.0);
        let colore = if self.rng.f32() < 0.5 { rgb(70, 110, 96) } else { rgb(120, 130, 90) };
        self.foglie.spawn(Particella { pos: Vec2::new(x, y), vel: Vec2::new(v * 70.0, 0.0), vita, durata: vita, colore, gravita: 6.0 });
    }

    fn muovi_foglie(&mut self) {
        let t = self.t;
        for p in &mut self.foglie.v {
            let v = self.terreno.vento(p.pos.x, t);
            p.vel.x += (v * 75.0 - p.vel.x) * (2.0 * DT);
            // Svolazzano.
            p.vel.y += (t * 5.0 + p.durata * 9.0).sin() * 30.0 * DT;
            let suolo = self.terreno.suolo(p.pos.x);
            if p.pos.y > suolo - 1.0 {
                p.pos.y = suolo - 1.0;
                p.vel.y = -p.vel.y.abs() * 0.3;
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
        let tasti = [Key::Left, Key::Right, Key::A, Key::D, Key::Space, Key::W, Key::Up];
        let attivo = tasti.iter().any(|&k| input.down(k));
        self.fermo = if attivo { 0.0 } else { self.fermo + DT };

        match self.scena {
            Scena::Fuori => {
                self.giocatore.update(input, &self.terreno, true);
                // Davanti all'arco, sulla piazza: il buio ti prende.
                let d = self.all_arco();
                self.davanti = if d.abs() < 9.0 && self.giocatore.a_terra { self.davanti + DT } else { 0.0 };
                if self.davanti > SOGLIA {
                    self.scena = Scena::Soglia(0.0);
                }
                self.lettere = self.lettere.max(((INQUADRA - d.abs()) / 150.0).clamp(0.0, 1.0));
            }
            Scena::Soglia(s) => {
                let arco = self.giocatore.centro_x() + self.all_arco();
                self.giocatore.verso_punto(arco, &self.terreno);
                self.scena = if s > DURATA_SOGLIA { Scena::Dentro(0.0) } else { Scena::Soglia(s + DT) };
            }
            Scena::Dentro(s) => {
                self.scena = Scena::Dentro(s + DT);
                if s > 6.0 && (input.pressed(Key::Space) || input.pressed(Key::Enter) || input.pressed(Key::W) || input.pressed(Key::Up)) {
                    // Un'altra notte: Egresso sarà altrove.
                    let seme = (hash01(self.terreno.seme as i32, 99, 0x1234) * 1e9) as u32;
                    *self = Egresso::con_seme(seme);
                }
                return;
            }
        }

        let c = self.giocatore.rect().center();
        let v = self.terreno.vicinanza(c.x);
        let quante = (12.0 + 50.0 * v * v) as usize;
        self.lucciole.update(&mut self.rng, &self.terreno, c, self.fermo, self.t, quante.min(natura::MAX));
        if self.scena == Scena::Fuori {
            self.foglie_nel_vento();
        }
        self.muovi_foglie();
        self.foglie.update(DT);

        let obiettivo = self.cam_obiettivo();
        self.cam += (obiettivo - self.cam) * (3.0 * DT).min(1.0);
    }

    fn render(&mut self, fb: &mut Framebuffer, stats: &Stats) {
        self.disegna(fb, stats);
    }

    fn comandi(&self) -> u32 {
        // Sul telefono: muoversi e saltare.
        0b11
    }

    /// 0: vento (quanto soffia dove sei); 1: richiamo (quanto sei vicino);
    /// 2: buio (si entra); 3: la voce sta parlando.
    fn suono(&self, canale: u32) -> f32 {
        let c = self.giocatore.centro_x();
        let fuori = match self.scena {
            Scena::Fuori => 1.0,
            Scena::Soglia(s) => (1.0 - s / DURATA_SOGLIA).max(0.0),
            Scena::Dentro(_) => 0.0,
        };
        match canale {
            0 => self.terreno.vento(c, self.t).abs().min(1.0) * fuori,
            1 => self.terreno.vicinanza(c).powi(2) * fuori,
            2 => 1.0 - fuori,
            3 => match self.scena {
                Scena::Dentro(s) if (1.5..3.5).contains(&s) => 1.0,
                _ => 0.0,
            },
            _ => 0.0,
        }
    }

    fn semina(&mut self, seme: u64) {
        let debug = self.debug;
        *self = Egresso::con_seme((seme as u32) | 1);
        self.debug = debug;
    }

    fn traccia(&self) -> String {
        let g = &self.giocatore;
        format!(
            "scena {:?} x {:7.1} y {:6.1} vel {:6.1} {:6.1} terra {} all'arco {:7.1} vicinanza {:.2} vento {:+.2}",
            self.scena,
            g.pos.x,
            g.pos.y,
            g.vel.x,
            g.vel.y,
            g.a_terra as u8,
            self.all_arco(),
            self.terreno.vicinanza(g.centro_x()),
            self.terreno.vento(g.centro_x(), self.t)
        )
    }
}

/// Le lettere del nome accese, data la vicinanza massima raggiunta (0..1).
pub fn lettere_accese(k: f32) -> usize {
    ((k * NOME.len() as f32).ceil() as usize).min(NOME.len())
}

#[cfg(test)]
mod tests {
    use super::*;

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

    /// Cammina verso l'arco, saltando quando un gradino ferma.
    fn cammina_verso_l_arco(g: &mut Egresso, secondi: f32) {
        let mut input = Input::new();
        for _ in 0..(secondi / DT) as usize {
            if g.scena != Scena::Fuori {
                return;
            }
            let d = g.all_arco();
            input.set(Key::Right, d > 0.0);
            input.set(Key::Left, d < 0.0);
            let bloccato = g.giocatore.a_terra && g.giocatore.vel.x == 0.0;
            input.set(Key::Space, bloccato || !g.giocatore.a_terra);
            g.update(&input);
            input.fine_passo();
        }
    }

    #[test]
    fn si_parte_lontano_e_senza_lettere() {
        let g = Egresso::new();
        assert!(g.all_arco().abs() > 800.0);
        assert_eq!(lettere_accese(g.lettere), 0);
    }

    #[test]
    fn camminando_verso_l_arco_lo_si_trova_e_si_entra() {
        for seme in [1, 2, 3, 4, 5, 6] {
            let mut g = Egresso::con_seme(seme);
            cammina_verso_l_arco(&mut g, 60.0);
            assert!(matches!(g.scena, Scena::Soglia(_)), "seme {seme}: {}", g.traccia());
            assert_eq!(lettere_accese(g.lettere), NOME.len());
            assert!(fino_a(&mut g, &[], 5.0, |g| matches!(g.scena, Scena::Dentro(_))));
        }
    }

    #[test]
    fn andando_dall_altra_parte_si_fa_il_giro() {
        let mut g = Egresso::con_seme(8);
        // Sempre dritto nella direzione sbagliata: il mondo gira, e prima o poi si arriva.
        let d = g.all_arco();
        let tasto = if d > 0.0 { Key::Left } else { Key::Right };
        let mut input = Input::new();
        input.set(tasto, true);
        let mut arrivato = false;
        for _ in 0..(120.0 / DT) as usize {
            let bloccato = g.giocatore.a_terra && g.giocatore.vel.x == 0.0;
            input.set(Key::Space, bloccato || !g.giocatore.a_terra);
            g.update(&input);
            input.fine_passo();
            if g.scena != Scena::Fuori {
                arrivato = true;
                break;
            }
        }
        assert!(arrivato, "{}", g.traccia());
    }

    #[test]
    fn dentro_si_puo_ricominciare_altrove() {
        let mut g = Egresso::con_seme(3);
        let arco = g.terreno.arco;
        g.scena = Scena::Dentro(7.0);
        let mut input = Input::new();
        input.set(Key::Space, true);
        g.update(&input);
        assert_eq!(g.scena, Scena::Fuori);
        assert_ne!(g.terreno.arco, arco);
    }

    #[test]
    fn il_suono_cresce_avvicinandosi() {
        let mut g = Egresso::con_seme(4);
        let lontano = g.suono(1);
        cammina_verso_l_arco(&mut g, 8.0);
        assert!(g.suono(1) > lontano);
    }
}
