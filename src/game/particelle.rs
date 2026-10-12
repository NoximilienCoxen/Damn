//! Particelle: punti di luce, polvere, bolle.
//!
//! Un unico `Vec` preallocato con capacità fissa: le particelle morte si
//! tolgono con `swap_remove` (O(1), l'ordine non conta) e se il pool è pieno
//! le nuove vengono semplicemente ignorate. Nessuna allocazione nel loop.
//! Le posizioni sono in pixel del mondo (non dello schermo).

use crate::engine::math::Vec2;

pub const MAX: usize = 3000;

#[derive(Clone, Copy, Debug)]
pub struct Particella {
    pub pos: Vec2,
    pub vel: Vec2,
    pub vita: f32,
    pub durata: f32,
    pub colore: u32,
    pub gravita: f32,
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

    /// Sposta di `dx` tutte le particelle oltre `da_x` (quando la stanza si allunga).
    pub fn sposta_oltre(&mut self, da_x: f32, dx: f32) {
        for p in &mut self.v {
            if p.pos.x >= da_x {
                p.pos.x += dx;
            }
        }
    }

    pub fn update(&mut self, dt: f32) {
        let mut i = 0;
        while i < self.v.len() {
            let p = &mut self.v[i];
            p.vita -= dt;
            if p.vita <= 0.0 {
                self.v.swap_remove(i);
                continue;
            }
            p.vel.y += p.gravita * dt;
            p.pos += p.vel * dt;
            i += 1;
        }
    }
}

impl Default for Particelle {
    fn default() -> Self {
        Self::new()
    }
}
