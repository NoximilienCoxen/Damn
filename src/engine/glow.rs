//! Glow: la luce delle lettere che si diffonde nel buio.
//!
//! Il gioco disegna le sorgenti luminose in un framebuffer a parte
//! ("emissivo"). Qui lo sfochiamo con un box blur separabile (prima per
//! righe, poi per colonne) e lo sommiamo all'immagine. Il box blur con somma
//! scorrevole costa uguale per qualunque raggio: O(pixel), non O(pixel × raggio).
//! Due passate di box blur approssimano già bene una gaussiana.
//!
//! Tutti i buffer sono allocati una volta sola in `new`: nel game loop non si
//! alloca niente.

use super::framebuffer::{canali, rgb, Framebuffer};

pub struct Glow {
    w: usize,
    h: usize,
    // Un buffer per canale, in u32 (virgola fissa ×256) per non saturare durante le somme.
    a: [Vec<u32>; 3],
    b: [Vec<u32>; 3],
}

impl Glow {
    pub fn new(w: usize, h: usize) -> Glow {
        let v = || vec![0u32; w * h];
        Glow { w, h, a: [v(), v(), v()], b: [v(), v(), v()] }
    }

    /// Somma a `dst` la versione sfocata di `src`, moltiplicata per `intensita`.
    pub fn apply(&mut self, src: &Framebuffer, dst: &mut Framebuffer, raggio: usize, intensita: f32) {
        assert_eq!((src.larghezza(), src.altezza()), (self.w, self.h));
        for (i, &p) in src.pixel().iter().enumerate() {
            let (r, g, b) = canali(p);
            // Virgola fissa (×256): senza, dividere per la finestra a ogni
            // passata azzererebbe le luci deboli.
            self.a[0][i] = (r as u32) << 8;
            self.a[1][i] = (g as u32) << 8;
            self.a[2][i] = (b as u32) << 8;
        }
        for c in 0..3 {
            // Due passate: a → b → a (orizzontale+verticale) per due volte.
            for _ in 0..2 {
                blur_righe(&self.a[c], &mut self.b[c], self.w, self.h, raggio);
                blur_colonne(&self.b[c], &mut self.a[c], self.w, self.h, raggio);
            }
        }
        let k = (intensita * 256.0) as u32;
        for (i, p) in dst.pixel_mut().iter_mut().enumerate() {
            let (r, g, b) = canali(*p);
            let add = |base: u8, v: u32| (base as u32 + ((v as u64 * k as u64) >> 16) as u32).min(255) as u8;
            *p = rgb(add(r, self.a[0][i]), add(g, self.a[1][i]), add(b, self.a[2][i]));
        }
    }
}

/// Media mobile orizzontale di raggio `r` (finestra di 2r+1 pixel).
fn blur_righe(src: &[u32], dst: &mut [u32], w: usize, h: usize, r: usize) {
    let finestra = (2 * r + 1) as u32;
    for y in 0..h {
        let riga = &src[y * w..(y + 1) * w];
        let out = &mut dst[y * w..(y + 1) * w];
        let mut somma: u32 = 0;
        // Fuori dai bordi contiamo zero (buio).
        for &v in riga.iter().take(r.min(w)) {
            somma += v;
        }
        for x in 0..w {
            if x + r < w {
                somma += riga[x + r];
            }
            out[x] = somma / finestra;
            if x >= r {
                somma -= riga[x - r];
            }
        }
    }
}

/// Media mobile verticale di raggio `r`.
fn blur_colonne(src: &[u32], dst: &mut [u32], w: usize, h: usize, r: usize) {
    let finestra = (2 * r + 1) as u32;
    for x in 0..w {
        let mut somma: u32 = 0;
        for y in 0..r.min(h) {
            somma += src[y * w + x];
        }
        for y in 0..h {
            if y + r < h {
                somma += src[(y + r) * w + x];
            }
            dst[y * w + x] = somma / finestra;
            if y >= r {
                somma -= src[(y - r) * w + x];
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_punto_si_allarga_e_resta_simmetrico() {
        let mut src = Framebuffer::new(21, 21);
        src.set_pixel(10, 10, rgb(255, 255, 255));
        let mut dst = Framebuffer::new(21, 21);
        let mut g = Glow::new(21, 21);
        g.apply(&src, &mut dst, 2, 8.0);
        let c = dst.get_pixel(10, 10).unwrap();
        assert!(c > 0, "il centro deve illuminarsi");
        assert_eq!(dst.get_pixel(8, 10), dst.get_pixel(12, 10));
        assert_eq!(dst.get_pixel(10, 8), dst.get_pixel(10, 12));
        assert_eq!(dst.get_pixel(0, 0), Some(0), "lontano resta buio");
    }
}
