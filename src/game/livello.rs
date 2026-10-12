//! La stanza del prototipo, descritta come dati.
//!
//! Due strati:
//! - la **realtà** (ASCII qui sotto): com'è fatto davvero il dungeon. Si vede
//!   e si tocca solo dentro una rivelazione;
//! - le **percezioni**: zone in cui i sensi producono attributi e le voci
//!   propongono nomi. Solo ciò che è nominato regge.

use super::lessico::{self, Lemma, Voce};

pub const COLONNE: i32 = 40;
pub const RIGHE: i32 = 22;
pub const TILE: i32 = 8;

/// La realtà, una riga per fila di tile.
/// `#` roccia, `=` gradino di pietra, `O` varco verso l'esterno, `.` vuoto.
pub const REALTA: [&str; RIGHE as usize] = [
    "########################################",
    "#......................................#",
    "#......................................#",
    "#......................................#",
    "#......................................#",
    "#......................................#",
    "#......................................#",
    "#......................................#",
    "#......................................#",
    "#......................................#",
    "#......................................#",
    "#......................................#",
    "#......................................#",
    "#......................................#",
    "#......................................#",
    "#......................................O",
    "#......................................O",
    "###############..........###############",
    "###############==========###############",
    "########################################",
    "########################################",
    "########################################",
];

/// Lanterne della realtà (in tile): illuminano ciò che viene rivelato.
pub const LANTERNE: &[(i32, i32)] = &[(19, 9), (33, 9), (7, 8)];

pub struct DefProposta {
    pub voce: Voce,
    pub lemma: &'static Lemma,
    /// Prima cella (in tile) e direzione.
    pub x: i32,
    pub y: i32,
    pub verticale: bool,
    /// Quante celle occupa: se è più lunga della parola, la parola si ripete.
    pub celle: i32,
}

impl DefProposta {
    pub fn cella(&self, i: i32) -> (i32, i32) {
        if self.verticale {
            (self.x, self.y + i)
        } else {
            (self.x + i, self.y)
        }
    }
}

pub struct DefPercetto {
    /// Area in tile (x0, y0, x1, y1), estremi inclusi.
    pub area: (i32, i32, i32, i32),
    pub attributi: &'static [&'static str],
    pub proposte: &'static [DefProposta],
    /// Già nominato all'inizio (indice della proposta).
    pub iniziale: Option<usize>,
}

const fn p(voce: Voce, lemma: &'static Lemma, x: i32, y: i32, verticale: bool, celle: i32) -> DefProposta {
    DefProposta { voce, lemma, x, y, verticale, celle }
}

use Voce::*;

pub static PERCETTI: &[DefPercetto] = &[
    // Il muro alle spalle: il protagonista l'ha già nominato.
    DefPercetto {
        area: (0, 4, 0, 16),
        attributi: &["FREDDO", "RUVIDO"],
        proposte: &[p(Controllo, &lessico::MURO, 0, 4, true, 13)],
        iniziale: Some(0),
    },
    DefPercetto {
        area: (1, 17, 14, 17),
        attributi: &["DURO"],
        proposte: &[p(Controllo, &lessico::PAVIMENTO, 1, 17, false, 14)],
        iniziale: Some(0),
    },
    // L'abisso: il cuore del puzzle. In realtà è un gradino.
    DefPercetto {
        area: (15, 14, 24, 21),
        attributi: &["VUOTO", "VENTO", "ECO"],
        proposte: &[
            p(Controllo, &lessico::PONTE, 15, 17, false, 10),
            p(Paura, &lessico::BURRONE, 21, 15, true, 7),
            p(Memoria, &lessico::FRIGO, 19, 16, true, 5),
        ],
        iniziale: None,
    },
    DefPercetto {
        area: (25, 17, 38, 17),
        attributi: &["DURO"],
        proposte: &[p(Controllo, &lessico::PAVIMENTO, 25, 17, false, 14)],
        iniziale: Some(0),
    },
    // L'uscita. Nominarla come PORTA riporta all'Atrio.
    DefPercetto {
        area: (36, 9, 38, 16),
        attributi: &["LEGNO", "SPIFFERO"],
        proposte: &[
            p(Controllo, &lessico::PORTA, 38, 12, true, 5),
            p(Paura, &lessico::TRAPPOLA, 37, 9, true, 8),
            p(Memoria, &lessico::ARMADIO, 36, 10, true, 7),
        ],
        iniziale: None,
    },
];

/// I biglietti che il protagonista si è lasciato: (colonna, righe di testo).
pub const BIGLIETTI: &[(i32, &[&str])] = &[
    (5, &["NOTA PER ME:", "QUELLO CHE NOMINO, REGGE."]),
    (13, &["NOTA PER ME:", "NON FIDARTI DELLA PAPERELLA."]),
    (31, &["NOTA PER ME:", "OGNI PORTA CHE HO NOMINATO", "MI HA RIPORTATO QUI."]),
];

/// Dove nasce il protagonista (in pixel, angolo in alto a sinistra).
pub const PARTENZA: (f32, f32) = (32.0, (17 * TILE - 12) as f32);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_realta_e_rettangolare() {
        for r in REALTA {
            assert_eq!(r.chars().count(), COLONNE as usize, "{r}");
        }
    }

    #[test]
    fn le_proposte_stanno_nella_stanza() {
        for d in PERCETTI {
            for pr in d.proposte {
                for i in 0..pr.celle {
                    let (x, y) = pr.cella(i);
                    assert!((0..COLONNE).contains(&x) && (0..RIGHE).contains(&y));
                }
            }
        }
    }
}
