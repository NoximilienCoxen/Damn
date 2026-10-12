//! La Sala: la voce del dungeon, e la guida del giocatore.
//!
//! Le tre voci interiori (Controllo, Paura, Memoria) danno nomi alle cose.
//! La Sala no: non nomina mai niente, al massimo ripete i nomi che dai tu.
//! Parla a **copioni**: poche frasi di fila legate a un momento del gioco.
//! Alcuni copioni, finendo, **sbloccano un comando**: è così che il
//! giocatore impara cosa può fare, un'azione alla volta.

/// I comandi che la Sala può sbloccare (maschera di bit).
pub mod comando {
    pub const MUOVI: u8 = 1;
    pub const SALTA: u8 = 2;
    /// E: dare un nome.
    pub const DI: u8 = 4;
    /// Q: ascoltare un'altra voce.
    pub const VOCE: u8 = 8;
    /// R: lasciare andare.
    pub const LASCIA: u8 = 16;
    pub const TUTTI: u8 = MUOVI | SALTA | DI | VOCE | LASCIA;
}

pub struct Copione {
    pub frasi: &'static [&'static str],
    /// Comandi sbloccati quando l'ultima frase è stata scritta.
    pub sblocca: u8,
    /// Un commento può scadere (non arriva fuori contesto); un copione che
    /// insegna qualcosa no.
    pub scade: bool,
}

/// I momenti in cui la Sala parla.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Battuta {
    Benvenuto,
    Vuoto,
    PrimaCaduta,
    PrimoNome,
    Abisso,
    TreVoci,
    Invecchia,
    PrimoLasciare,
    Porta,
    Burro,
    RitornoAllAtrio,
    Fermo,
}

const N: usize = Battuta::Fermo as usize + 1;

use comando::*;

impl Battuta {
    pub const TUTTE: [Battuta; N] = [
        Battuta::Benvenuto,
        Battuta::Vuoto,
        Battuta::PrimaCaduta,
        Battuta::PrimoNome,
        Battuta::Abisso,
        Battuta::TreVoci,
        Battuta::Invecchia,
        Battuta::PrimoLasciare,
        Battuta::Porta,
        Battuta::Burro,
        Battuta::RitornoAllAtrio,
        Battuta::Fermo,
    ];

    pub fn copione(self) -> &'static Copione {
        match self {
            Battuta::Benvenuto => {
                &Copione { frasi: &["BENTORNATO.", "NON TI RICORDI DI ME? NON IMPORTA.", "CAMMINA."], sblocca: MUOVI, scade: false }
            }
            Battuta::Vuoto => &Copione {
                frasi: &["DAVANTI A TE NON C'È NIENTE. NON ANCORA.", "DAGLI UN NOME, E SARÀ QUELLO."],
                sblocca: DI,
                scade: false,
            },
            Battuta::PrimaCaduta => {
                &Copione { frasi: &["CADERE È UN MODO DI CONOSCERMI.", "DAGLI UN NOME, PRIMA."], sblocca: DI, scade: false }
            }
            Battuta::PrimoNome => &Copione { frasi: &["LO CHIAMI PONTE. VA BENE.", "ORA ESISTE. PER TE."], sblocca: 0, scade: true },
            Battuta::Abisso => &Copione { frasi: &["QUI LE PAROLE INVECCHIANO.", "E C'È DA SALTARE."], sblocca: SALTA, scade: false },
            Battuta::TreVoci => &Copione {
                frasi: &["DENTRO DI TE, OGNUNO LO CHIAMA A MODO SUO.", "ASCOLTALI TUTTI. POI SCEGLI."],
                sblocca: VOCE,
                scade: false,
            },
            Battuta::Invecchia => &Copione {
                frasi: &["TUTTO CIÒ CHE DICI, INVECCHIA.", "PUOI ANCHE NON DIRE NIENTE.", "LASCIA ANDARE, E GUARDA."],
                sblocca: LASCIA,
                scade: false,
            },
            Battuta::PrimoLasciare => &Copione { frasi: &["ECCO. ADESSO MI VEDI."], sblocca: 0, scade: true },
            Battuta::Porta => &Copione { frasi: &["OGNI PORTA CHE HAI NOMINATO", "TI HA RIPORTATO QUI."], sblocca: 0, scade: true },
            Battuta::Burro => &Copione { frasi: &["NON GUARDARE ME."], sblocca: 0, scade: true },
            Battuta::RitornoAllAtrio => &Copione { frasi: &["OGNI PORTA CHE NOMINI È MIA."], sblocca: 0, scade: true },
            Battuta::Fermo => &Copione { frasi: &["COSÌ. BRAVO."], sblocca: 0, scade: true },
        }
    }
}

/// Velocità di scrittura (caratteri al secondo) e tempi di ogni frase.
const CPS: f32 = 26.0;
const PAUSA: f32 = 1.3;
const DISSOLVENZA: f32 = 0.4;
/// Un commento che aspetta in coda più di così ha perso il suo momento.
const SCADENZA: f32 = 4.5;
/// Se qualcuno aspetta, una frase di commento resta solo questo dopo essere stata scritta.
const PAUSA_BREVE: f32 = 0.8;

#[derive(Clone, Copy, Debug)]
pub struct Corrente {
    pub battuta: Battuta,
    pub frase: usize,
    pub t: f32,
    sbloccato: bool,
}

pub struct Sala {
    dette: [bool; N],
    /// Copioni in attesa, con da quanto aspettano. Al massimo 3: la Sala non insiste.
    coda: [Option<(Battuta, f32)>; 3],
    pub attuale: Option<Corrente>,
}

impl Default for Sala {
    fn default() -> Self {
        Self::new()
    }
}

fn scrittura(testo: &str) -> f32 {
    testo.chars().count() as f32 / CPS
}

fn durata(testo: &str) -> f32 {
    scrittura(testo) + PAUSA + DISSOLVENZA
}

impl Sala {
    pub fn new() -> Sala {
        Sala { dette: [false; N], coda: [None; 3], attuale: None }
    }

    pub fn detta(&self, b: Battuta) -> bool {
        self.dette[b as usize]
    }

    /// `true` se la Sala sta parlando o ha qualcosa da dire.
    pub fn occupata(&self) -> bool {
        self.attuale.is_some() || self.coda[0].is_some()
    }

    /// Chiede alla Sala di dire il copione `b`, se non l'ha già detto.
    pub fn di(&mut self, b: Battuta) {
        if self.dette[b as usize] {
            return;
        }
        if let Some(posto) = self.coda.iter_mut().find(|p| p.is_none()) {
            *posto = Some((b, 0.0));
            self.dette[b as usize] = true;
        }
    }

    /// Avanza il tempo. Restituisce i comandi appena sbloccati (0 se nessuno).
    pub fn update(&mut self, dt: f32) -> u8 {
        let mut sbloccati = 0;
        let qualcuno_aspetta = self.coda[0].is_some();
        if let Some(mut c) = self.attuale {
            let copione = c.battuta.copione();
            let testo = copione.frasi[c.frase];
            c.t += dt;
            let ultima = c.frase + 1 == copione.frasi.len();
            if ultima && !c.sbloccato && c.t >= scrittura(testo) {
                c.sbloccato = true;
                sbloccati |= copione.sblocca;
            }
            // Un commento lascia spazio a chi aspetta.
            if copione.scade && qualcuno_aspetta && c.t > scrittura(testo) + PAUSA_BREVE {
                c.t = c.t.max(durata(testo) - DISSOLVENZA);
            }
            self.attuale = if c.t <= durata(testo) {
                Some(c)
            } else if !ultima {
                Some(Corrente { frase: c.frase + 1, t: 0.0, ..c })
            } else {
                None
            };
        }
        for (_, attesa) in self.coda.iter_mut().flatten() {
            *attesa += dt;
        }
        while self.attuale.is_none() {
            let Some((b, attesa)) = self.coda[0].take() else { break };
            self.coda.rotate_left(1);
            if b.copione().scade && attesa > SCADENZA {
                // Fuori tempo: non lo dice, e potrà dirlo la prossima volta che serve.
                self.dette[b as usize] = false;
            } else {
                self.attuale = Some(Corrente { battuta: b, frase: 0, t: 0.0, sbloccato: false });
            }
        }
        sbloccati
    }

    /// La frase che si vede: testo, caratteri già scritti, opacità (0..1).
    pub fn visibile(&self) -> Option<(&'static str, usize, f32)> {
        let c = self.attuale?;
        let testo = c.battuta.copione().frasi[c.frase];
        let n = ((c.t * CPS) as usize).min(testo.chars().count());
        let alpha = ((durata(testo) - c.t) / DISSOLVENZA).clamp(0.0, 1.0);
        Some((testo, n, alpha))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn avanza(s: &mut Sala, secondi: f32) -> u8 {
        let mut sb = 0;
        for _ in 0..(secondi * 60.0) as usize {
            sb |= s.update(1.0 / 60.0);
        }
        sb
    }

    #[test]
    fn un_copione_dice_le_frasi_in_ordine_e_poi_sblocca() {
        let mut s = Sala::new();
        s.di(Battuta::Benvenuto);
        s.update(0.0);
        assert_eq!(s.visibile().unwrap().0, "BENTORNATO.");
        let mut viste = vec!["BENTORNATO."];
        let mut sb = 0;
        for _ in 0..1200 {
            sb |= s.update(1.0 / 60.0);
            if let Some((t, _, _)) = s.visibile() {
                if *viste.last().unwrap() != t {
                    viste.push(t);
                }
            }
        }
        assert_eq!(viste, Battuta::Benvenuto.copione().frasi);
        assert_eq!(sb, comando::MUOVI);
        assert!(!s.occupata());
    }

    #[test]
    fn ogni_copione_una_volta_sola() {
        let mut s = Sala::new();
        s.di(Battuta::Burro);
        s.di(Battuta::Burro);
        avanza(&mut s, 10.0);
        s.di(Battuta::Burro);
        s.update(0.0);
        assert!(s.attuale.is_none());
    }

    #[test]
    fn un_commento_fuori_tempo_si_perde_ma_puo_tornare() {
        let mut s = Sala::new();
        s.di(Battuta::Benvenuto); // insegna: lungo e non scade
        s.update(0.0);
        s.di(Battuta::PrimoNome); // commento: aspetta troppo
        avanza(&mut s, 12.0);
        assert!(!s.detta(Battuta::PrimoNome));
        s.di(Battuta::PrimoNome);
        s.update(0.0);
        assert_eq!(s.attuale.map(|c| c.battuta), Some(Battuta::PrimoNome));
    }

    #[test]
    fn un_copione_che_insegna_non_scade() {
        let mut s = Sala::new();
        s.di(Battuta::Benvenuto);
        s.update(0.0);
        s.di(Battuta::Vuoto);
        let sb = avanza(&mut s, 20.0);
        assert_eq!(sb, comando::MUOVI | comando::DI);
    }

    #[test]
    fn le_frasi_entrano_nello_schermo() {
        for b in Battuta::TUTTE {
            for f in b.copione().frasi {
                // 7 px per carattere (la Sala ha lettere più distanziate).
                assert!(f.chars().count() as i32 * 7 <= 316, "{f} troppo lunga");
            }
        }
    }
}
