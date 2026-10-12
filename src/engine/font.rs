//! Un font bitmap 5×7 scritto a mano: nessun file, solo stringhe.
//!
//! Ogni glifo è 7 righe da 5 caratteri: `#` = pixel acceso, `.` = spento.
//! All'avvio le stringhe vengono convertite una volta sola in maschere di bit
//! (`[u8; 7]`, un bit per colonna), così disegnare un carattere costa pochissimo.
//! Le minuscole vengono disegnate come maiuscole.

use super::framebuffer::Framebuffer;

pub const GLIFO_W: i32 = 5;
pub const GLIFO_H: i32 = 7;
/// Spazio orizzontale occupato da un carattere (glifo + 1 pixel di spaziatura).
pub const AVANZAMENTO: i32 = 6;

/// (carattere, righe)
const GLIFI: &[(char, [&str; 7])] = &[
    ('A', [".###.", "#...#", "#...#", "#####", "#...#", "#...#", "#...#"]),
    ('B', ["####.", "#...#", "#...#", "####.", "#...#", "#...#", "####."]),
    ('C', [".###.", "#...#", "#....", "#....", "#....", "#...#", ".###."]),
    ('D', ["####.", "#...#", "#...#", "#...#", "#...#", "#...#", "####."]),
    ('E', ["#####", "#....", "#....", "####.", "#....", "#....", "#####"]),
    ('F', ["#####", "#....", "#....", "####.", "#....", "#....", "#...."]),
    ('G', [".###.", "#...#", "#....", "#.###", "#...#", "#...#", ".####"]),
    ('H', ["#...#", "#...#", "#...#", "#####", "#...#", "#...#", "#...#"]),
    ('I', [".###.", "..#..", "..#..", "..#..", "..#..", "..#..", ".###."]),
    ('J', ["..###", "...#.", "...#.", "...#.", "...#.", "#..#.", ".##.."]),
    ('K', ["#...#", "#..#.", "#.#..", "##...", "#.#..", "#..#.", "#...#"]),
    ('L', ["#....", "#....", "#....", "#....", "#....", "#....", "#####"]),
    ('M', ["#...#", "##.##", "#.#.#", "#.#.#", "#...#", "#...#", "#...#"]),
    ('N', ["#...#", "#...#", "##..#", "#.#.#", "#..##", "#...#", "#...#"]),
    ('O', [".###.", "#...#", "#...#", "#...#", "#...#", "#...#", ".###."]),
    ('P', ["####.", "#...#", "#...#", "####.", "#....", "#....", "#...."]),
    ('Q', [".###.", "#...#", "#...#", "#...#", "#.#.#", "#..#.", ".##.#"]),
    ('R', ["####.", "#...#", "#...#", "####.", "#.#..", "#..#.", "#...#"]),
    ('S', [".####", "#....", "#....", ".###.", "....#", "....#", "####."]),
    ('T', ["#####", "..#..", "..#..", "..#..", "..#..", "..#..", "..#.."]),
    ('U', ["#...#", "#...#", "#...#", "#...#", "#...#", "#...#", ".###."]),
    ('V', ["#...#", "#...#", "#...#", "#...#", "#...#", ".#.#.", "..#.."]),
    ('W', ["#...#", "#...#", "#...#", "#.#.#", "#.#.#", "#.#.#", ".#.#."]),
    ('X', ["#...#", "#...#", ".#.#.", "..#..", ".#.#.", "#...#", "#...#"]),
    ('Y', ["#...#", "#...#", ".#.#.", "..#..", "..#..", "..#..", "..#.."]),
    ('Z', ["#####", "....#", "...#.", "..#..", ".#...", "#....", "#####"]),
    ('0', [".###.", "#...#", "#..##", "#.#.#", "##..#", "#...#", ".###."]),
    ('1', ["..#..", ".##..", "..#..", "..#..", "..#..", "..#..", ".###."]),
    ('2', [".###.", "#...#", "....#", "...#.", "..#..", ".#...", "#####"]),
    ('3', ["####.", "....#", "....#", ".###.", "....#", "....#", "####."]),
    ('4', ["...#.", "..##.", ".#.#.", "#..#.", "#####", "...#.", "...#."]),
    ('5', ["#####", "#....", "####.", "....#", "....#", "#...#", ".###."]),
    ('6', [".###.", "#....", "#....", "####.", "#...#", "#...#", ".###."]),
    ('7', ["#####", "....#", "...#.", "..#..", ".#...", ".#...", ".#..."]),
    ('8', [".###.", "#...#", "#...#", ".###.", "#...#", "#...#", ".###."]),
    ('9', [".###.", "#...#", "#...#", ".####", "....#", "....#", ".###."]),
    ('.', [".....", ".....", ".....", ".....", ".....", ".##..", ".##.."]),
    (',', [".....", ".....", ".....", ".....", ".##..", "..#..", ".#..."]),
    (':', [".....", ".##..", ".##..", ".....", ".##..", ".##..", "....."]),
    ('!', ["..#..", "..#..", "..#..", "..#..", "..#..", ".....", "..#.."]),
    ('?', [".###.", "#...#", "....#", "...#.", "..#..", ".....", "..#.."]),
    ('\'', ["..#..", "..#..", ".#...", ".....", ".....", ".....", "....."]),
    ('"', [".#.#.", ".#.#.", ".....", ".....", ".....", ".....", "....."]),
    ('-', [".....", ".....", ".....", ".###.", ".....", ".....", "....."]),
    ('+', [".....", "..#..", "..#..", "#####", "..#..", "..#..", "....."]),
    ('=', [".....", ".....", "#####", ".....", "#####", ".....", "....."]),
    ('/', ["....#", "...#.", "...#.", "..#..", ".#...", ".#...", "#...."]),
    ('(', ["...#.", "..#..", ".#...", ".#...", ".#...", "..#..", "...#."]),
    (')', [".#...", "..#..", "...#.", "...#.", "...#.", "..#..", ".#..."]),
    ('[', [".###.", ".#...", ".#...", ".#...", ".#...", ".#...", ".###."]),
    (']', [".###.", "...#.", "...#.", "...#.", "...#.", "...#.", ".###."]),
    ('>', [".#...", "..#..", "...#.", "....#", "...#.", "..#..", ".#..."]),
    ('<', ["...#.", "..#..", ".#...", "#....", ".#...", "..#..", "...#."]),
    ('%', ["##..#", "##..#", "...#.", "..#..", ".#...", "#..##", "#..##"]),
    ('_', [".....", ".....", ".....", ".....", ".....", ".....", "#####"]),
    ('·', [".....", ".....", ".....", "..#..", ".....", ".....", "....."]),
    // Accentate (in italiano le maiuscole accentate servono poco, ma "È" sì).
    ('À', [".#...", "..#..", ".###.", "#...#", "#####", "#...#", "#...#"]),
    ('È', [".#...", "..#..", "#####", "#....", "####.", "#....", "#####"]),
    ('É', ["...#.", "..#..", "#####", "#....", "####.", "#....", "#####"]),
    ('Ì', [".#...", "..#..", ".###.", "..#..", "..#..", "..#..", ".###."]),
    ('Ò', [".#...", "..#..", ".###.", "#...#", "#...#", "#...#", ".###."]),
    ('Ù', [".#...", "..#..", "#...#", "#...#", "#...#", "#...#", ".###."]),
];

/// Glifo di riserva per i caratteri sconosciuti.
const IGNOTO: [u8; 7] = [0b11111, 0b10001, 0b10101, 0b10101, 0b10101, 0b10001, 0b11111];

pub struct Font {
    /// Maschere per i caratteri ASCII (indice = codice del carattere).
    ascii: [Option<[u8; 7]>; 128],
    /// Gli altri (accentati), cercati in un piccolo elenco.
    extra: Vec<(char, [u8; 7])>,
}

impl Default for Font {
    fn default() -> Self {
        Self::new()
    }
}

impl Font {
    pub fn new() -> Font {
        let mut f = Font { ascii: [None; 128], extra: Vec::new() };
        for (c, righe) in GLIFI {
            let mut m = [0u8; 7];
            for (r, riga) in righe.iter().enumerate() {
                for (col, ch) in riga.chars().enumerate() {
                    if ch == '#' {
                        m[r] |= 1 << (4 - col);
                    }
                }
            }
            if (*c as u32) < 128 {
                f.ascii[*c as usize] = Some(m);
            } else {
                f.extra.push((*c, m));
            }
        }
        f
    }

    /// La maschera del carattere `c` (le minuscole diventano maiuscole).
    pub fn glifo(&self, c: char) -> [u8; 7] {
        let c = match c {
            'à' => 'À',
            'è' => 'È',
            'é' => 'É',
            'ì' => 'Ì',
            'ò' => 'Ò',
            'ù' => 'Ù',
            c => c.to_ascii_uppercase(),
        };
        if (c as u32) < 128 {
            return self.ascii[c as usize].unwrap_or(IGNOTO);
        }
        self.extra.iter().find(|(e, _)| *e == c).map(|(_, m)| *m).unwrap_or(IGNOTO)
    }

    /// Disegna un carattere con l'angolo in alto a sinistra in (x, y).
    ///
    /// `inclina` sposta le righe alte a destra (corsivo finto: la voce della
    /// Memoria). `alpha` 255 = opaco.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_char(&self, fb: &mut Framebuffer, x: i32, y: i32, c: char, colore: u32, alpha: u8, inclina: bool) {
        if c == ' ' {
            return;
        }
        let m = self.glifo(c);
        for (r, bits) in m.iter().enumerate() {
            let dx = if inclina { (6 - r as i32) / 3 } else { 0 };
            for col in 0..GLIFO_W {
                if bits & (1 << (4 - col)) != 0 {
                    if alpha == 255 {
                        fb.set_pixel(x + col + dx, y + r as i32, colore);
                    } else {
                        fb.blend_pixel(x + col + dx, y + r as i32, colore, alpha);
                    }
                }
            }
        }
    }

    /// Come `draw_char`, ma accende solo metà dei pixel a scacchiera: una
    /// lettera "retinata", che sembra incerta. `fase` sposta la scacchiera
    /// (cambiandola nel tempo la lettera sfrigola).
    #[allow(clippy::too_many_arguments)]
    pub fn draw_char_retinato(&self, fb: &mut Framebuffer, x: i32, y: i32, c: char, colore: u32, alpha: u8, inclina: bool, fase: i32) {
        if c == ' ' {
            return;
        }
        let m = self.glifo(c);
        for (r, bits) in m.iter().enumerate() {
            let dx = if inclina { (6 - r as i32) / 3 } else { 0 };
            for col in 0..GLIFO_W {
                let (px, py) = (x + col + dx, y + r as i32);
                if bits & (1 << (4 - col)) != 0 && (px + py + fase) & 1 == 0 {
                    fb.blend_pixel(px, py, colore, alpha);
                }
            }
        }
    }

    /// Disegna un testo su una riga. Restituisce la larghezza in pixel.
    pub fn draw_text(&self, fb: &mut Framebuffer, x: i32, y: i32, testo: &str, colore: u32) -> i32 {
        self.draw_text_alpha(fb, x, y, testo, colore, 255)
    }

    pub fn draw_text_alpha(&self, fb: &mut Framebuffer, x: i32, y: i32, testo: &str, colore: u32, alpha: u8) -> i32 {
        let mut cx = x;
        for c in testo.chars() {
            self.draw_char(fb, cx, y, c, colore, alpha, false);
            cx += AVANZAMENTO;
        }
        cx - x
    }

    /// Disegna un testo centrato orizzontalmente attorno a `cx`.
    pub fn draw_centered(&self, fb: &mut Framebuffer, cx: i32, y: i32, testo: &str, colore: u32, alpha: u8) {
        let x = cx - misura(testo) / 2;
        self.draw_text_alpha(fb, x, y, testo, colore, alpha);
    }
}

/// Larghezza in pixel di un testo su una riga.
pub fn misura(testo: &str) -> i32 {
    let n = testo.chars().count() as i32;
    if n == 0 {
        0
    } else {
        n * AVANZAMENTO - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tutti_i_glifi_sono_5x7() {
        for (c, righe) in GLIFI {
            for r in righe {
                assert_eq!(r.chars().count(), 5, "glifo {c:?}");
                assert!(r.chars().all(|ch| ch == '#' || ch == '.'), "glifo {c:?}");
            }
        }
    }

    #[test]
    fn nessun_doppione() {
        for (i, (a, _)) in GLIFI.iter().enumerate() {
            assert!(GLIFI[i + 1..].iter().all(|(b, _)| a != b), "doppione {a:?}");
        }
    }

    #[test]
    fn minuscole_come_maiuscole() {
        let f = Font::new();
        assert_eq!(f.glifo('a'), f.glifo('A'));
        assert_eq!(f.glifo('è'), f.glifo('È'));
        assert_eq!(f.glifo('@'), IGNOTO);
    }

    #[test]
    fn disegna_la_i() {
        let f = Font::new();
        let mut fb = Framebuffer::new(5, 7);
        f.draw_char(&mut fb, 0, 0, 'I', 1, 255, false);
        // Colonna centrale tutta accesa.
        for y in 0..7 {
            assert_eq!(fb.get_pixel(2, y), Some(1));
        }
        assert_eq!(misura("CIAO"), 23);
    }
}
