//! Le stanze della Sala, descritte come dati.
//!
//! Ogni stanza è una griglia di 40×22 tile (`#` roccia, `.` aria, `~` acqua,
//! `O` l'arco verso fuori). Una stanza può avere un **modulo**: un gruppo di
//! colonne che la Sala ripete quando la forzi. Più copie del modulo, più la
//! stanza è lunga: è la regola dello sforzo (vedi `docs/CONCEPT.md`).
//!
//! Le stanze di questo prototipo sono tre, ma il formato è pensato per
//! molte di più: aggiungerne una è aggiungere una `static` e collegarla.

pub const COLONNE: i32 = 40;
pub const RIGHE: i32 = 22;
pub const TILE: i32 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fluido {
    Acqua,
    Aria,
}

/// Una corrente: dentro la sua area, chi si lascia andare viene portato.
pub struct DefCorrente {
    /// Area in tile (x0, y0, x1, y1), estremi inclusi, coordinate della mappa base.
    pub area: (i32, i32, i32, i32),
    /// Velocità della corrente (pixel/s).
    pub vel: (f32, f32),
    pub fluido: Fluido,
    /// `Some((periodo, durata))`: soffia a raffiche (secondi).
    pub raffiche: Option<(f32, f32)>,
}

/// Con cosa è scritto qualcosa sul muro: dice chi l'ha scritto e quando.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Inchiostro {
    /// Il Cartografo: carbone, si legge solo da vicino, con la lanterna.
    Carbone,
    /// La Nuotatrice: gesso, chiaro.
    Gesso,
    /// Il Curatore: targhe d'ottone, ufficiali e bugiarde.
    Ottone,
}

pub struct Scritta {
    /// Angolo in alto a sinistra, in tile della mappa base.
    pub x: i32,
    pub y: i32,
    pub righe: &'static [&'static str],
    pub inchiostro: Inchiostro,
}

pub struct Stanza {
    pub nome: &'static str,
    pub mappa: [&'static str; RIGHE as usize],
    /// Colonne ripetibili: (prima colonna, larghezza).
    pub modulo: Option<(i32, i32)>,
    /// Copie extra del modulo all'inizio della partita.
    pub moduli_iniziali: f32,
    pub lanterne: &'static [(i32, i32)],
    pub correnti: &'static [DefCorrente],
    pub scritte: &'static [Scritta],
    /// Stanze collegate oltre il bordo sinistro e destro (indice in `STANZE`).
    pub sinistra: Option<usize>,
    pub destra: Option<usize>,
}

pub const RIVA_I: usize = 0;
pub const ATRIO_I: usize = 1;
pub const GALLERIA_I: usize = 2;

/// La Riva: qui ti svegli. L'acqua del canale corre verso l'arco, cioè
/// verso fuori. Per entrare nella Sala hai nuotato contro: per questo è già
/// lunga. Dalla riva al canale non si passa a nuoto (c'è la diga): si passa
/// dalla mensola in alto, e sulla mensola ti porta solo l'aria.
pub static RIVA: Stanza = Stanza {
    nome: "LA RIVA",
    mappa: [
        "########################################",
        "########################################",
        "######...#..........#..................#",
        "######...#.............................#",
        "######.................................#",
        "######.................................#",
        "######......##############.............#",
        "######..........##.....................#",
        "######..........##.....................#",
        "######..........##......................",
        "######..........##......................",
        "######..........##......................",
        "######..........##......################",
        "O~~~##~~~~~~~~~~##~~~~~~################",
        "O~~~##~~~~~~~~~~##~~~~~~################",
        "O~~~~~~~~~~~~~~~##~~~~~~################",
        "O~~~~~~~~~~~~~~~##~~~~~~################",
        "O~~~~~~~~~~~~~~~##~~~~~~################",
        "O~~~~~~~~~~~~~~~##~~~~~~################",
        "O~~~~~~~~~~~~~~~##~~~~~~################",
        "O~~~~~~~~~~~~~~~##~~~~~~################",
        "########################################",
    ],
    modulo: Some((8, 4)),
    moduli_iniziali: 8.0,
    lanterne: &[(36, 2), (21, 9)],
    correnti: &[
        // Nel canale, in superficie l'acqua spinge piano contro il labbro di roccia...
        DefCorrente { area: (1, 13, 15, 14), vel: (-26.0, 0.0), fluido: Fluido::Acqua, raffiche: None },
        // ...e sotto porta via. Nella vasca vicino alla riva è ferma.
        DefCorrente { area: (1, 15, 15, 20), vel: (-80.0, 0.0), fluido: Fluido::Acqua, raffiche: None },
        // Una colonna d'aria sale dalla riva fino alla mensola.
        DefCorrente { area: (26, 2, 27, 11), vel: (0.0, -110.0), fluido: Fluido::Aria, raffiche: None },
    ],
    scritte: &[
        Scritta { x: 29, y: 5, righe: &["LA RIVA.", "L'ACQUA NON", "PORTA DA", "NESSUNA PARTE."], inchiostro: Inchiostro::Ottone },
        Scritta { x: 13, y: 3, righe: &["SONO ENTRATO DI QUI.", "CHE FATICA."], inchiostro: Inchiostro::Carbone },
    ],
    sinistra: None,
    destra: Some(ATRIO_I),
};

/// L'Atrio: un corridoio con il vento a raffiche. Chi ci corre contro lo allunga.
pub static ATRIO: Stanza = Stanza {
    nome: "L'ATRIO",
    mappa: [
        "########################################",
        "########################################",
        "########################################",
        "#......................................#",
        "#......................................#",
        "#......................................#",
        "#......................................#",
        "#......................................#",
        "#......................................#",
        "........................................",
        "......#.....#.....#.....#.....#....#....",
        "......#.....#.....#.....#.....#....#....",
        "########################################",
        "########################################",
        "########################################",
        "########################################",
        "########################################",
        "########################################",
        "########################################",
        "########################################",
        "########################################",
        "########################################",
    ],
    modulo: Some((18, 6)),
    moduli_iniziali: 0.0,
    lanterne: &[(4, 3), (15, 3), (27, 3), (36, 3)],
    correnti: &[DefCorrente { area: (1, 3, 38, 11), vel: (-130.0, 0.0), fluido: Fluido::Aria, raffiche: Some((3.6, 1.5)) }],
    scritte: &[
        Scritta { x: 2, y: 8, righe: &["ATRIO: 40 PASSI."], inchiostro: Inchiostro::Carbone },
        Scritta { x: 8, y: 5, righe: &["L'ATRIO.", "LUNGHEZZA COSTANTE."], inchiostro: Inchiostro::Ottone },
        Scritta { x: 25, y: 4, righe: &["RIMISURATO: 71.", "POI 118.", "CRESCE QUANDO", "CORRO?"], inchiostro: Inchiostro::Carbone },
        Scritta { x: 14, y: 7, righe: &["IL VENTO", "NON CE L'HA", "CON TE."], inchiostro: Inchiostro::Gesso },
    ],
    sinistra: Some(RIVA_I),
    destra: Some(GALLERIA_I),
};

/// La Galleria delle Correnti: una vasca divisa da un labbro di roccia e una
/// colonna d'aria che sale. Il corridoio in alto riporta all'ingresso.
pub static GALLERIA: Stanza = Stanza {
    nome: "LA GALLERIA DELLE CORRENTI",
    mappa: [
        "########################################",
        "########################################",
        "#.....................................##",
        "#.....................................##",
        "#.....................................##",
        "#......###########################....##",
        "#...............##.....................#",
        "#...............##.....................#",
        "#...............##.....................#",
        "................##.....................#",
        "................##.....................#",
        "................##.....................#",
        "#######.........##..........############",
        "#######~~~~~~~~~##~~~~~~~~~~############",
        "#######~~~~~~~~~##~~~~~~~~~~############",
        "#######~~~~~~~~~~~~~~~~~~~~~############",
        "#######~~~~~~~~~~~~~~~~~~~~~############",
        "#######~~~~~~~~~~~~~~~~~~~~~############",
        "#######~~~~~~~~~~~~~~~~~~~~~############",
        "#######~~~~~~~~~~~~~~~~~~~~~############",
        "#######~~~~~~~~~~~~~~~~~~~~~############",
        "########################################",
    ],
    modulo: None,
    moduli_iniziali: 0.0,
    lanterne: &[(5, 4), (36, 4), (15, 11), (38, 11)],
    correnti: &[
        DefCorrente { area: (7, 15, 27, 20), vel: (70.0, 0.0), fluido: Fluido::Acqua, raffiche: None },
        DefCorrente { area: (34, 2, 37, 11), vel: (0.0, -110.0), fluido: Fluido::Aria, raffiche: None },
    ],
    scritte: &[
        Scritta { x: 1, y: 6, righe: &["GALLERIA", "DELLE", "CORRENTI.", "NO BAGNI."], inchiostro: Inchiostro::Ottone },
        Scritta { x: 8, y: 7, righe: &["HO NUOTATO", "CONTRO IL", "MURO.", "TRE ORE."], inchiostro: Inchiostro::Carbone },
        Scritta { x: 19, y: 7, righe: &["HO SMESSO", "DI NUOTARE.", "L'ACQUA SA", "DOVE ANDARE."], inchiostro: Inchiostro::Gesso },
        Scritta { x: 28, y: 7, righe: &["L'ARIA MI", "RESPINGE.", "RIPROVO", "DOMANI."], inchiostro: Inchiostro::Carbone },
        Scritta { x: 8, y: 2, righe: &["QUELLA DELLA RIVA", "VA FUORI. SEMPRE."], inchiostro: Inchiostro::Gesso },
        Scritta { x: 24, y: 2, righe: &["SERRA: CHIUSA", "PER AMPLIAMENTO."], inchiostro: Inchiostro::Ottone },
    ],
    sinistra: Some(ATRIO_I),
    destra: None,
};

pub static STANZE: [&Stanza; 3] = [&RIVA, &ATRIO, &GALLERIA];

/// La scritta che, letta, fa capire come si esce.
pub const INDIZIO: (usize, usize) = (GALLERIA_I, 4);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_mappe_sono_rettangolari() {
        for st in STANZE {
            for r in st.mappa {
                assert_eq!(r.chars().count(), COLONNE as usize, "{}: {r}", st.nome);
            }
        }
    }

    #[test]
    fn le_stanze_collegate_si_guardano() {
        for (i, st) in STANZE.iter().enumerate() {
            if let Some(d) = st.destra {
                assert_eq!(STANZE[d].sinistra, Some(i), "{}", st.nome);
            }
        }
    }

    #[test]
    fn le_scritte_stanno_nello_schermo() {
        for st in STANZE {
            for s in st.scritte {
                for r in s.righe {
                    let fine = s.x * TILE + r.chars().count() as i32 * 6;
                    assert!(fine <= COLONNE * TILE, "{}: {r}", st.nome);
                }
            }
        }
    }
}
