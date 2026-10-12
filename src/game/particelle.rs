//! Particelle: punti di luce e lettere che cadono.
//!
//! Un unico `Vec` preallocato con capacità fissa: le particelle morte si
//! tolgono con `swap_remove` (O(1), l'ordine non conta) e se il pool è pieno
//! le nuove vengono semplicemente ignorate. Nessuna allocazione nel loop.

use super::livello::TILE;
use super::mondo::Mondo;
use crate::engine::math::Vec2;

pub const MAX: usize = 3000;

#[derive(Clone, Copy, Debug)]
pub struct Particella {
    pub pos: Vec2,
    pub vel: Vec2,
    pub vita: f32,
    pub durata: f32,
    pub colore: u32,
    /// `' '` = un punto di luce; altrimenti una lettera che cade.
    pub lettera: char,
    pub gravita: f32,
    pub inclina: bool,
}

pub struct Particelle {
    pub v: Vec<Particella>,
}

impl Particelle {
    pub fn new() -> Particelle {
        Particelle { v: Vec::with_capacity(MAX) }
    }

    pub fn spawn(&mut self, p: Particella) {
        if self.v.len() < MAX {
            self.v.push(p);
        }
    }

    pub fn clear(&mut self) {
        self.v.clear();
    }

    pub fn update(&mut self, dt: f32, mondo: &Mondo) {
        let mut i = 0;
        while i < self.v.len() {
            let p = &mut self.v[i];
            p.vita -= dt;
            if p.vita <= 0.0 || p.pos.y > 200.0 {
                self.v.swap_remove(i);
                continue;
            }
            p.vel.y += p.gravita * dt;
            p.pos += p.vel * dt;
            // Le lettere che cadono si posano su ciò che regge.
            if p.lettera != ' ' && p.vel.y > 0.0 {
                let tx = ((p.pos.x + 2.5) / TILE as f32).floor() as i32;
                let ty = ((p.pos.y + 7.0) / TILE as f32).floor() as i32;
                if mondo.solido(tx, ty).is_some() {
                    p.pos.y = (ty * TILE) as f32 - 7.0;
                    p.vel.y = 0.0;
                    p.vel.x *= 0.7;
                }
            }
            i += 1;
        }
    }
}

impl Default for Particelle {
    fn default() -> Self {
        Self::new()
    }
}
