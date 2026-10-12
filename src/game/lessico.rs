//! Il lessico: le voci interiori e le parole che possono proporre.
//!
//! Ogni parola (lemma) è una catena di stati: la parola "sana" e le sue
//! versioni marce. Lo stato attivo dipende dalla certezza (da 1 a 0): quando
//! scende sotto la soglia di uno stato si passa al successivo, le lettere
//! cambiano o cadono e con loro cambia la natura dell'oggetto.
//! È tutto scritto a mano, di proposito: così le mutazioni restano
//! controllabili e diventano materiale per i puzzle.

use crate::engine::framebuffer::rgb;

/// Chi propone un nome.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Voce {
    /// Etichette precise e rassicuranti.
    Controllo,
    /// Pericoli ovunque.
    Paura,
    /// Ricordi sbagliati, spesso assurdi.
    Memoria,
}

impl Voce {
    pub fn colore(self) -> u32 {
        match self {
            Voce::Controllo => rgb(255, 226, 170),
            Voce::Paura => rgb(255, 72, 84),
            Voce::Memoria => rgb(110, 225, 235),
        }
    }

    pub fn nome(self) -> &'static str {
        match self {
            Voce::Controllo => "CONTROLLO",
            Voce::Paura => "PAURA",
            Voce::Memoria => "MEMORIA",
        }
    }

    /// Quanto trema la parola proposta (in pixel).
    pub fn tremito(self) -> f32 {
        match self {
            Voce::Controllo => 0.4,
            Voce::Paura => 1.6,
            Voce::Memoria => 0.8,
        }
    }
}

/// Cosa *è* un oggetto nominato, cioè come si comporta.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Natura {
    /// Regge: ci si cammina sopra, blocca.
    Solida,
    /// Regge, ma si scivola.
    Scivolosa,
    /// Un nome che non regge niente (BURRONE, TORTA, POSTE).
    Vuota,
    /// Una porta: attraversarla riporta all'Atrio.
    Porta,
    /// Toccarla fa male.
    Trappola,
}

impl Natura {
    pub fn regge(self) -> bool {
        matches!(self, Natura::Solida | Natura::Scivolosa)
    }
}

pub struct Stato {
    /// Testo dello stato. Tutti gli stati di un lemma hanno la stessa
    /// lunghezza: lo spazio è una lettera caduta.
    pub testo: &'static str,
    pub natura: Natura,
    /// Lo stato è attivo finché la certezza è >= soglia.
    pub soglia: f32,
}

pub struct Lemma {
    pub stati: &'static [Stato],
    /// Certezza persa al secondo, sempre.
    pub logorio: f32,
    /// Certezza persa in più al secondo mentre ci stai sopra (l'uso consuma).
    pub logorio_uso: f32,
}

impl Lemma {
    /// Indice dello stato attivo per una certa certezza.
    pub fn stato_per(&self, certezza: f32) -> usize {
        self.stati.iter().position(|s| certezza >= s.soglia).unwrap_or(self.stati.len() - 1)
    }

    pub fn lunghezza(&self) -> usize {
        self.stati[0].testo.chars().count()
    }
}

const fn st(testo: &'static str, natura: Natura, soglia: f32) -> Stato {
    Stato { testo, natura, soglia }
}

use Natura::*;

pub static PAVIMENTO: Lemma =
    Lemma { stati: &[st("PAVIMENTO", Solida, 0.5), st("PAVIM NTO", Solida, 0.0)], logorio: 0.0, logorio_uso: 0.0 };

pub static MURO: Lemma =
    Lemma { stati: &[st("MURO", Solida, 0.5), st("MURA", Solida, 0.25), st("MORA", Vuota, 0.0)], logorio: 0.0, logorio_uso: 0.0 };

pub static PONTE: Lemma =
    Lemma { stati: &[st("PONTE", Solida, 0.66), st("PON E", Solida, 0.33), st("POSTE", Vuota, 0.0)], logorio: 0.012, logorio_uso: 0.22 };

/// Il ponte dell'Atrio: il primo nome che dici. Non marcisce: prima si
/// impara che nominare funziona, poi che consuma.
pub static PONTE_SALDO: Lemma = Lemma { stati: &[st("PONTE", Solida, 0.0)], logorio: 0.0, logorio_uso: 0.0 };

pub static BURRONE: Lemma = Lemma { stati: &[st("BURRONE", Vuota, 0.4), st("BURRO  ", Scivolosa, 0.0)], logorio: 0.05, logorio_uso: 0.0 };

pub static FRIGO: Lemma =
    Lemma { stati: &[st("FRIGO", Solida, 0.6), st("FR GO", Solida, 0.3), st("FICO ", Vuota, 0.0)], logorio: 0.01, logorio_uso: 0.16 };

pub static PORTA: Lemma = Lemma { stati: &[st("PORTA", Porta, 0.5), st("TORTA", Vuota, 0.0)], logorio: 0.03, logorio_uso: 0.0 };

pub static TRAPPOLA: Lemma =
    Lemma { stati: &[st("TRAPPOLA", Trappola, 0.4), st("TRAPPOLA", Trappola, 0.0)], logorio: 0.0, logorio_uso: 0.0 };

pub static ARMADIO: Lemma = Lemma { stati: &[st("ARMADIO", Solida, 0.5), st("ARMADI ", Solida, 0.0)], logorio: 0.01, logorio_uso: 0.05 };

#[cfg(test)]
mod tests {
    use super::*;

    const TUTTI: &[&Lemma] = &[&PAVIMENTO, &MURO, &PONTE, &PONTE_SALDO, &BURRONE, &FRIGO, &PORTA, &TRAPPOLA, &ARMADIO];

    #[test]
    fn stati_della_stessa_lunghezza_e_soglie_decrescenti() {
        for l in TUTTI {
            let n = l.lunghezza();
            for s in l.stati {
                assert_eq!(s.testo.chars().count(), n, "{}", s.testo);
            }
            for w in l.stati.windows(2) {
                assert!(w[0].soglia > w[1].soglia, "{}", w[0].testo);
            }
            assert_eq!(l.stati.last().unwrap().soglia, 0.0, "l'ultimo stato copre tutto");
        }
    }

    #[test]
    fn il_ponte_marcisce_in_poste() {
        assert_eq!(PONTE.stati[PONTE.stato_per(1.0)].testo, "PONTE");
        assert_eq!(PONTE.stati[PONTE.stato_per(0.5)].testo, "PON E");
        let fine = &PONTE.stati[PONTE.stato_per(0.1)];
        assert_eq!(fine.testo, "POSTE");
        assert!(!fine.natura.regge());
    }

    #[test]
    fn la_paura_diventa_burro() {
        let fine = &BURRONE.stati[BURRONE.stato_per(0.0)];
        assert_eq!(fine.natura, Natura::Scivolosa);
    }
}
