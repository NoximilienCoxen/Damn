//! Le lucciole: poche, curiose, e più fitte vicino a Egresso.
//!
//! Ne esiste un numero fisso (`MAX`), tutte attorno al protagonista: quando
//! una resta troppo indietro rinasce poco più avanti, nel buio. Quante sono
//! accese dipende da quanto si è vicini a Egresso.
//! Vagano, si avvicinano a chi è fermo, si lasciano portare dal vento verso
//! l'arco ma non entrano: davanti al buio dell'arco si fermano.

use super::terreno::{delta, Terreno};
use crate::engine::math::{Rng, Vec2};
use crate::engine::time::DT;

pub const MAX: usize = 64;
const RAGGIO: f32 = 230.0;

#[derive(Clone, Copy, Debug)]
pub struct Lucciola {
    pub pos: Vec2,
    pub vel: Vec2,
    pub fase: f32,
    /// 0..1: si accendono e si spengono piano.
    pub luce: f32,
}

pub struct Lucciole {
    pub v: Vec<Lucciola>,
}

impl Lucciole {
    pub fn new(rng: &mut Rng, attorno: f32, terreno: &Terreno) -> Lucciole {
        let v = (0..MAX)
            .map(|_| {
                let x = attorno + rng.range(-RAGGIO, RAGGIO);
                Lucciola {
                    pos: Vec2::new(x, terreno.suolo(x) - rng.range(8.0, 50.0)),
                    vel: Vec2::ZERO,
                    fase: rng.range(0.0, std::f32::consts::TAU),
                    luce: 0.0,
                }
            })
            .collect();
        Lucciole { v }
    }

    /// `quante`: quante devono essere accese; `chi`: centro del protagonista.
    pub fn update(&mut self, rng: &mut Rng, terreno: &Terreno, chi: Vec2, fermo: f32, t: f32, quante: usize) {
        let arco = terreno.arco;
        for (i, l) in self.v.iter_mut().enumerate() {
            let viva = i < quante;
            l.luce = if viva { (l.luce + DT * 0.6).min(1.0) } else { (l.luce - DT * 0.8).max(0.0) };
            // Troppo lontana: rinasce dall'altra parte, spenta.
            let dx = l.pos.x - chi.x;
            if dx.abs() > RAGGIO {
                let x = chi.x - dx.signum() * rng.range(RAGGIO * 0.4, RAGGIO * 0.95);
                l.pos = Vec2::new(x, terreno.suolo(x) - rng.range(8.0, 50.0));
                l.vel = Vec2::ZERO;
                l.luce = 0.0;
                continue;
            }
            let f = l.fase;
            // Vagare.
            let mut acc = Vec2::new((t * 0.9 + f).sin() * 30.0, (t * 1.3 + f * 1.7).cos() * 26.0);
            // Il vento le porta.
            acc.x += terreno.vento(l.pos.x, t) * 40.0;
            // Curiose: se stai fermo si avvicinano e ti girano attorno.
            let verso_te = chi - l.pos;
            let d = verso_te.len().max(1.0);
            if d < 70.0 {
                let k = 10.0 + 50.0 * (fermo / 2.0).min(1.0);
                let giro = Vec2::new(-verso_te.y, verso_te.x) * (1.0 / d);
                acc += verso_te * (k * (d - 16.0) / (d * d)) + giro * 14.0;
            }
            // Restano sopra l'erba, sotto la cima degli alberi.
            let suolo = terreno.suolo(l.pos.x);
            if l.pos.y > suolo - 5.0 {
                acc.y -= 120.0;
            }
            if l.pos.y < suolo - 70.0 {
                acc.y += 40.0;
            }
            // Davanti all'arco si fermano: il buio non lo attraversano.
            let da = delta(arco, l.pos.x);
            if da.abs() < 16.0 && l.pos.y > terreno.piazza_y - 40.0 {
                acc.x += da.signum() * 260.0;
            }
            l.vel += acc * DT;
            l.vel = l.vel * (1.0 - 1.6 * DT);
            l.pos += l.vel * DT;
        }
    }
}
