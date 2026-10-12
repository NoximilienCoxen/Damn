//! La Sala: la voce del dungeon.
//!
//! La Sala ti vuole bene e non vuole che te ne vada. Non spiega le regole:
//! **incoraggia lo sforzo** (è questo che la rende ironica) e si fa
//! inquieta quando smetti di forzare. Parla a **copioni**: poche frasi di
//! fila legate a un momento del gioco, ognuno detto una volta sola.

pub struct Copione {
    pub frasi: &'static [&'static str],
    /// Un commento può scadere (non arriva fuori contesto); l'accoglienza no.
    pub scade: bool,
}

/// I momenti in cui la Sala parla.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Battuta {
    Benvenuto,
    Cresce,
    Muro,
    Atrio,
    Rallenti,
    Galleria,
    Affonda,
    Vento,
    Ritorno,
    Arco,
}

const N: usize = Battuta::Arco as usize + 1;

impl Battuta {
    pub const TUTTE: [Battuta; N] = [
        Battuta::Benvenuto,
        Battuta::Cresce,
        Battuta::Muro,
        Battuta::Atrio,
        Battuta::Rallenti,
        Battuta::Galleria,
        Battuta::Affonda,
        Battuta::Vento,
        Battuta::Ritorno,
        Battuta::Arco,
    ];

    pub fn copione(self) -> &'static Copione {
        match self {
            Battuta::Benvenuto => {
                &Copione { frasi: &["BENTORNATO.", "HAI NUOTATO TANTO, PER ARRIVARE QUI.", "LA SALA È DI LÀ."], scade: false }
            }
            Battuta::Cresce => &Copione { frasi: &["COSÌ. SPINGI.", "C'È SEMPRE PIÙ SALA DA VEDERE."], scade: true },
            Battuta::Muro => &Copione { frasi: &["PIÙ FORTE."], scade: true },
            Battuta::Atrio => &Copione { frasi: &["LA PORTA È IN FONDO. SBRIGATI."], scade: true },
            Battuta::Rallenti => &Copione { frasi: &["...PERCHÉ RALLENTI?"], scade: true },
            Battuta::Galleria => &Copione { frasi: &["CHE BELLA GALLERIA, EH?"], scade: true },
            Battuta::Affonda => &Copione { frasi: &["NUOTA! NON STARE LÌ!"], scade: true },
            Battuta::Vento => &Copione { frasi: &["NON COSÌ. SALTA, SALTA!"], scade: true },
            Battuta::Ritorno => &Copione { frasi: &["DOVE VAI? LA SALA È DI QUA."], scade: true },
            Battuta::Arco => &Copione { frasi: &["NO.", "ASPETTA."], scade: false },
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

    /// Avanza il tempo.
    pub fn update(&mut self, dt: f32) {
        let qualcuno_aspetta = self.coda[0].is_some();
        if let Some(mut c) = self.attuale {
            let copione = c.battuta.copione();
            let testo = copione.frasi[c.frase];
            c.t += dt;
            let ultima = c.frase + 1 == copione.frasi.len();
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
                self.attuale = Some(Corrente { battuta: b, frase: 0, t: 0.0 });
            }
        }
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

    fn avanza(s: &mut Sala, secondi: f32) {
        for _ in 0..(secondi * 60.0) as usize {
            s.update(1.0 / 60.0);
        }
    }

    #[test]
    fn un_copione_dice_le_frasi_in_ordine() {
        let mut s = Sala::new();
        s.di(Battuta::Benvenuto);
        s.update(0.0);
        let mut viste = vec![s.visibile().unwrap().0];
        for _ in 0..1200 {
            s.update(1.0 / 60.0);
            if let Some((t, _, _)) = s.visibile() {
                if *viste.last().unwrap() != t {
                    viste.push(t);
                }
            }
        }
        assert_eq!(viste, Battuta::Benvenuto.copione().frasi);
        assert!(!s.occupata());
    }

    #[test]
    fn ogni_copione_una_volta_sola() {
        let mut s = Sala::new();
        s.di(Battuta::Muro);
        s.di(Battuta::Muro);
        avanza(&mut s, 10.0);
        s.di(Battuta::Muro);
        s.update(0.0);
        assert!(s.attuale.is_none());
    }

    #[test]
    fn un_commento_fuori_tempo_si_perde_ma_puo_tornare() {
        let mut s = Sala::new();
        s.di(Battuta::Benvenuto); // lungo e non scade
        s.update(0.0);
        s.di(Battuta::Muro); // commento: aspetta troppo
        avanza(&mut s, 12.0);
        assert!(!s.detta(Battuta::Muro));
        s.di(Battuta::Muro);
        s.update(0.0);
        assert_eq!(s.attuale.map(|c| c.battuta), Some(Battuta::Muro));
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
