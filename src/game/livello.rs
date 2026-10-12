//! Le stanze del prototipo, descritte come dati.
//!
//! Due strati:
//! - la **realtà** (ASCII qui sotto): com'è fatto davvero il dungeon. Si vede
//!   e si tocca solo dentro una rivelazione;
//! - le **percezioni**: zone in cui i sensi producono attributi e le voci
//!   propongono nomi. Solo ciò che è nominato regge.

use super::lessico::{self, Lemma, Voce};
use super::sala::Battuta;

pub const COLONNE: i32 = 40;
pub const RIGHE: i32 = 22;
pub const TILE: i32 = 8;

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
    /// Cosa dice la Sala la prima volta che lo percepisci.
    pub copione: Option<Battuta>,
}

const fn p(voce: Voce, lemma: &'static Lemma, x: i32, y: i32, verticale: bool, celle: i32) -> DefProposta {
    DefProposta { voce, lemma, x, y, verticale, celle }
}

use Voce::*;

/// Una stanza: la sua realtà e le sue percezioni.
pub struct Stanza {
    pub nome: &'static str,
    /// La realtà, una riga per fila di tile.
    /// `#` roccia, `=` gradino di pietra, `O` varco verso l'esterno, `.` vuoto.
    pub realta: [&'static str; RIGHE as usize],
    /// Lanterne della realtà (in tile): illuminano ciò che viene rivelato.
    pub lanterne: &'static [(i32, i32)],
    pub percetti: &'static [DefPercetto],
    /// Dove nasce il protagonista (in pixel, angolo in alto a sinistra).
    pub partenza: (f32, f32),
    /// Camminando oltre il bordo destro si passa alla stanza successiva.
    pub uscita_destra: bool,
}

const PARTENZA: (f32, f32) = (32.0, (17 * TILE - 12) as f32);

/// Stanza 0, l'Atrio: insegna una sola cosa. Il vuoto non regge finché non
/// lo nomini. (È anche il posto in cui ti riporta ogni PORTA nominata.)
pub static ATRIO: Stanza = Stanza {
    nome: "L'ATRIO",
    realta: [
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
        "#.......................................",
        "#.......................................",
        "########################################",
        "########################################",
        "########################################",
        "########################################",
        "########################################",
    ],
    lanterne: &[(6, 8), (18, 9), (31, 8)],
    percetti: &[
        DefPercetto {
            area: (0, 4, 0, 16),
            attributi: &["FREDDO", "RUVIDO"],
            proposte: &[p(Controllo, &lessico::MURO, 0, 4, true, 13)],
            iniziale: Some(0),
            copione: None,
        },
        DefPercetto {
            area: (1, 17, 12, 17),
            attributi: &["DURO"],
            proposte: &[p(Controllo, &lessico::PAVIMENTO, 1, 17, false, 12)],
            iniziale: Some(0),
            copione: None,
        },
        // Il primo vuoto: una voce sola, una parola sola.
        DefPercetto {
            area: (13, 14, 22, 21),
            attributi: &["VUOTO"],
            proposte: &[p(Controllo, &lessico::PONTE_SALDO, 13, 17, false, 10)],
            iniziale: None,
            copione: Some(Battuta::Vuoto),
        },
        DefPercetto {
            area: (23, 17, 39, 17),
            attributi: &["DURO"],
            proposte: &[p(Controllo, &lessico::PAVIMENTO, 23, 17, false, 17)],
            iniziale: Some(0),
            copione: None,
        },
    ],
    partenza: PARTENZA,
    uscita_destra: true,
};

/// Stanza 1, l'Abisso: tre voci, una porta, due finali.
pub static ABISSO: Stanza = Stanza {
    nome: "L'ABISSO",
    realta: [
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
    ],
    lanterne: &[(19, 9), (33, 9), (7, 8)],
    percetti: &[
        // Il muro alle spalle: il protagonista l'ha già nominato.
        DefPercetto {
            area: (0, 4, 0, 16),
            attributi: &["FREDDO", "RUVIDO"],
            proposte: &[p(Controllo, &lessico::MURO, 0, 4, true, 13)],
            iniziale: Some(0),
            copione: None,
        },
        DefPercetto {
            area: (1, 17, 14, 17),
            attributi: &["DURO"],
            proposte: &[p(Controllo, &lessico::PAVIMENTO, 1, 17, false, 14)],
            iniziale: Some(0),
            copione: None,
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
            copione: Some(Battuta::TreVoci),
        },
        DefPercetto {
            area: (25, 17, 38, 17),
            attributi: &["DURO"],
            proposte: &[p(Controllo, &lessico::PAVIMENTO, 25, 17, false, 14)],
            iniziale: Some(0),
            copione: None,
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
            copione: Some(Battuta::Porta),
        },
    ],
    partenza: PARTENZA,
    uscita_destra: false,
};

pub static STANZE: [&Stanza; 2] = [&ATRIO, &ABISSO];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_realta_e_rettangolare() {
        for st in STANZE {
            for r in st.realta {
                assert_eq!(r.chars().count(), COLONNE as usize, "{}: {r}", st.nome);
            }
        }
    }

    #[test]
    fn le_proposte_stanno_nella_stanza() {
        for st in STANZE {
            for d in st.percetti {
                for pr in d.proposte {
                    for i in 0..pr.celle {
                        let (x, y) = pr.cella(i);
                        assert!((0..COLONNE).contains(&x) && (0..RIGHE).contains(&y), "{}", st.nome);
                    }
                }
            }
        }
    }
}
