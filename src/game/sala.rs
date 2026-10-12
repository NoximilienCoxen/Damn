//! La Sala: la voce del dungeon.
//!
//! Le tre voci interiori (Controllo, Paura, Memoria) danno nomi alle cose.
//! La Sala no: non nomina mai niente, al massimo ripete i nomi che dai tu.
//! Parla di rado, solo nei momenti che contano, una frase alla volta, e
//! ogni frase una volta sola per partita. È calma, un po' ironica, e sa già
//! come va a finire.

/// Le battute della Sala, ognuna legata a un momento del gioco.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Battuta {
    Benvenuto,
    PrimaCaduta,
    PrimoNome,
    Abisso,
    NomeMarcito,
    PrimoLasciare,
    Burro,
    RitornoAllAtrio,
    Fermo,
}

const N: usize = Battuta::Fermo as usize + 1;

impl Battuta {
    pub fn testo(self) -> &'static str {
        match self {
            Battuta::Benvenuto => "BENTORNATO.",
            Battuta::PrimaCaduta => "CADERE È UN MODO DI CONOSCERMI.",
            Battuta::PrimoNome => "LO CHIAMI PONTE. VA BENE.",
            Battuta::Abisso => "QUI LE PAROLE INVECCHIANO.",
            Battuta::NomeMarcito => "TUTTO CIÒ CHE DICI, INVECCHIA.",
            Battuta::PrimoLasciare => "ECCO. ADESSO MI VEDI.",
            Battuta::Burro => "NON GUARDARE ME.",
            Battuta::RitornoAllAtrio => "OGNI PORTA CHE NOMINI È MIA.",
            Battuta::Fermo => "COSÌ. BRAVO.",
        }
    }
}

/// Velocità di scrittura (caratteri al secondo) e tempo di permanenza.
const CPS: f32 = 20.0;
const PAUSA: f32 = 2.6;
const DISSOLVENZA: f32 = 0.8;
/// Una battuta che aspetta in coda più di così ha perso il suo momento.
const SCADENZA: f32 = 4.5;
/// Se qualcuno aspetta, la battuta attuale resta solo questo dopo essere stata scritta.
const PAUSA_BREVE: f32 = 1.0;

pub struct Sala {
    dette: [bool; N],
    /// Battute in attesa, con da quanto aspettano (al massimo 3: le altre si
    /// perdono, la Sala non insiste).
    coda: [Option<(Battuta, f32)>; 3],
    /// La battuta che sta dicendo e da quanti secondi.
    pub attuale: Option<(Battuta, f32)>,
}

impl Default for Sala {
    fn default() -> Self {
        Self::new()
    }
}

impl Sala {
    pub fn new() -> Sala {
        Sala { dette: [false; N], coda: [None; 3], attuale: None }
    }

    pub fn detta(&self, b: Battuta) -> bool {
        self.dette[b as usize]
    }

    /// Chiede alla Sala di dire `b`, se non l'ha già detta.
    pub fn di(&mut self, b: Battuta) {
        if self.dette[b as usize] {
            return;
        }
        if let Some(posto) = self.coda.iter_mut().find(|p| p.is_none()) {
            *posto = Some((b, 0.0));
            self.dette[b as usize] = true;
        }
    }

    pub fn update(&mut self, dt: f32) {
        let attesa = self.coda[0].is_some();
        if let Some((b, t)) = self.attuale {
            let mut t = t + dt;
            // Qualcuno aspetta: la pausa si accorcia e si passa alla dissolvenza.
            let scritta = b.testo().chars().count() as f32 / CPS;
            if attesa && t > scritta + PAUSA_BREVE {
                t = t.max(Self::durata(b) - DISSOLVENZA);
            }
            self.attuale = if t > Self::durata(b) { None } else { Some((b, t)) };
        }
        for (_, attesa) in self.coda.iter_mut().flatten() {
            *attesa += dt;
        }
        while self.attuale.is_none() {
            let Some((b, attesa)) = self.coda[0].take() else { break };
            self.coda.rotate_left(1);
            if attesa > SCADENZA {
                // Fuori tempo: non la dice, e potrà dirla la prossima volta che serve.
                self.dette[b as usize] = false;
            } else {
                self.attuale = Some((b, 0.0));
            }
        }
    }

    fn durata(b: Battuta) -> f32 {
        b.testo().chars().count() as f32 / CPS + PAUSA + DISSOLVENZA
    }

    /// Quanti caratteri si vedono e con che opacità (0..1).
    pub fn visibile(&self) -> Option<(&'static str, usize, f32)> {
        let (b, t) = self.attuale?;
        let testo = b.testo();
        let n = ((t * CPS) as usize).min(testo.chars().count());
        let fine = Self::durata(b);
        let alpha = ((fine - t) / DISSOLVENZA).clamp(0.0, 1.0);
        Some((testo, n, alpha))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ogni_battuta_una_volta_sola() {
        let mut s = Sala::new();
        s.di(Battuta::PrimoNome);
        s.di(Battuta::PrimoNome);
        s.update(0.0);
        assert_eq!(s.attuale.map(|a| a.0), Some(Battuta::PrimoNome));
        for _ in 0..600 {
            s.update(1.0 / 60.0);
        }
        assert!(s.attuale.is_none(), "detta una volta e basta");
        s.di(Battuta::PrimoNome);
        s.update(0.0);
        assert!(s.attuale.is_none());
    }

    #[test]
    fn si_scrive_lettera_per_lettera_poi_svanisce() {
        let mut s = Sala::new();
        s.di(Battuta::Benvenuto);
        s.update(0.0);
        s.update(0.1);
        let (_, n, a) = s.visibile().unwrap();
        assert_eq!(n, 2);
        assert_eq!(a, 1.0);
        // Circa 4 secondi: scrittura, pausa, dissolvenza.
        for _ in 0..300 {
            s.update(1.0 / 60.0);
        }
        assert!(s.visibile().is_none());
    }

    #[test]
    fn le_battute_aspettano_il_turno() {
        let mut s = Sala::new();
        s.di(Battuta::PrimaCaduta);
        s.di(Battuta::PrimoNome);
        s.update(0.0);
        assert_eq!(s.attuale.unwrap().0, Battuta::PrimaCaduta);
        for _ in 0..400 {
            s.update(1.0 / 60.0);
            if s.attuale.map(|a| a.0) == Some(Battuta::PrimoNome) {
                return;
            }
        }
        panic!("la seconda battuta non è mai arrivata");
    }

    #[test]
    fn una_battuta_fuori_tempo_si_perde_ma_puo_tornare() {
        let mut s = Sala::new();
        s.di(Battuta::PrimaCaduta);
        s.update(0.0);
        s.di(Battuta::NomeMarcito);
        s.di(Battuta::PrimoNome); // aspetta le altre due: più di 6 s
        for _ in 0..600 {
            s.update(1.0 / 60.0);
        }
        assert!(s.attuale.is_none(), "PrimoNome era scaduta");
        assert!(!s.detta(Battuta::PrimoNome));
        s.di(Battuta::PrimoNome);
        s.update(0.0);
        assert_eq!(s.attuale.map(|a| a.0), Some(Battuta::PrimoNome));
    }

    #[test]
    fn le_battute_entrano_nello_schermo() {
        use Battuta::*;
        for b in [Benvenuto, PrimaCaduta, PrimoNome, Abisso, NomeMarcito, PrimoLasciare, Burro, RitornoAllAtrio, Fermo] {
            // 7 px per carattere (la Sala ha lettere più distanziate).
            assert!(b.testo().chars().count() as i32 * 7 <= 316, "{:?} troppo lunga", b);
        }
    }
}
