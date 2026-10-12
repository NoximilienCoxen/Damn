//! Lo stato del mondo: la realtà, le percezioni e le lettere che reggono.
//!
//! La "mappa" è una griglia di tile che contiene, per ogni cella, la lettera
//! nominata che la occupa (se c'è). Viene ricostruita solo quando qualcosa
//! cambia (una parola affermata, una lettera caduta), non a ogni frame.

use super::lessico::{Natura, Voce};
use super::livello::{DefPercetto, Stanza, COLONNE, RIGHE, TILE};
use crate::engine::math::{Rect, Vec2};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reale {
    Vuoto,
    Roccia,
    Gradino,
    Varco,
}

impl Reale {
    pub fn regge(self) -> bool {
        matches!(self, Reale::Roccia | Reale::Gradino)
    }
}

/// Una lettera nominata che occupa una cella.
#[derive(Clone, Copy, Debug)]
pub struct Cella {
    pub lettera: char,
    pub natura: Natura,
    pub voce: Voce,
    pub percetto: u16,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Fase {
    /// Non ancora percepito.
    Ignoto,
    /// I sensi lavorano: compaiono gli attributi (secondi trascorsi).
    Sentito(f32),
    /// Le voci propongono.
    Proposto,
    /// Nominato: è una cosa fatta di lettere.
    Affermato,
    /// Lasciato andare (secondi prima di poter essere ri-percepito).
    Lasciato(f32),
}

pub struct Percetto {
    pub def: &'static DefPercetto,
    pub fase: Fase,
    /// La proposta in primo piano (ciclata con Q).
    pub focus: usize,
    /// La proposta affermata.
    pub scelta: usize,
    pub certezza: f32,
    /// Indice dello stato del lemma (0 = parola sana).
    pub stato: usize,
    /// Il protagonista ci sta sopra in questo passo.
    pub calpestato: bool,
}

impl Percetto {
    /// L'area in pixel.
    pub fn rect(&self) -> Rect {
        let (x0, y0, x1, y1) = self.def.area;
        Rect::new((x0 * TILE) as f32, (y0 * TILE) as f32, ((x1 - x0 + 1) * TILE) as f32, ((y1 - y0 + 1) * TILE) as f32)
    }

    /// La lettera della proposta `i` nella cella `k`, secondo lo stato attuale.
    pub fn lettera(&self, i: usize, k: i32, stato: usize) -> char {
        let pr = &self.def.proposte[i];
        let testo = pr.lemma.stati[stato].testo;
        let n = pr.lemma.lunghezza() as i32;
        testo.chars().nth((k % n) as usize).unwrap_or(' ')
    }
}

/// Una finestra sulla realtà, aperta lasciando andare una parola.
#[derive(Clone, Copy, Debug)]
pub struct Rivelazione {
    pub centro: Vec2,
    pub t: f32,
}

pub const RIVELAZIONE_DURATA: f32 = 5.0;
pub const RIVELAZIONE_RAGGIO: f32 = 60.0;

impl Rivelazione {
    pub fn raggio(&self) -> f32 {
        let apertura = (self.t / 0.35).min(1.0);
        let apertura = 1.0 - (1.0 - apertura) * (1.0 - apertura);
        let chiusura = ((RIVELAZIONE_DURATA - self.t) / 1.2).clamp(0.0, 1.0);
        RIVELAZIONE_RAGGIO * apertura * chiusura
    }
}

/// Cose successe durante un passo, che il gioco trasforma in effetti.
#[derive(Clone, Copy, Debug)]
pub enum Evento {
    /// Una lettera si è staccata da una parola che marcisce.
    LetteraCaduta { pos: Vec2, lettera: char, voce: Voce },
    /// Una lettera si è sciolta in luce (parola lasciata andare).
    LetteraSciolta { pos: Vec2, lettera: char, voce: Voce },
    /// Una parola è cambiata (es. PONTE → POSTE).
    Mutazione { percetto: u16 },
}

pub struct Mondo {
    pub stanza: &'static Stanza,
    pub reale: Vec<Reale>,
    pub mappa: Vec<Option<Cella>>,
    pub percetti: Vec<Percetto>,
    pub rivelazioni: Vec<Rivelazione>,
    pub eventi: Vec<Evento>,
    sporca: bool,
}

const DIST_SENSI: f32 = 28.0;
const TEMPO_SENSI: f32 = 0.9;

impl Mondo {
    pub fn new(stanza: &'static Stanza) -> Mondo {
        let mut reale = Vec::with_capacity((COLONNE * RIGHE) as usize);
        for riga in stanza.realta {
            for c in riga.chars() {
                reale.push(match c {
                    '#' => Reale::Roccia,
                    '=' => Reale::Gradino,
                    'O' => Reale::Varco,
                    _ => Reale::Vuoto,
                });
            }
        }
        let percetti = stanza
            .percetti
            .iter()
            .map(|def| Percetto {
                def,
                fase: if def.iniziale.is_some() { Fase::Affermato } else { Fase::Ignoto },
                focus: 0,
                scelta: def.iniziale.unwrap_or(0),
                certezza: 1.0,
                stato: 0,
                calpestato: false,
            })
            .collect();
        let mut m = Mondo {
            stanza,
            reale,
            mappa: vec![None; (COLONNE * RIGHE) as usize],
            percetti,
            rivelazioni: Vec::with_capacity(8),
            eventi: Vec::with_capacity(64),
            sporca: true,
        };
        m.ricostruisci();
        m
    }

    fn indice(tx: i32, ty: i32) -> Option<usize> {
        if (0..COLONNE).contains(&tx) && (0..RIGHE).contains(&ty) {
            Some((ty * COLONNE + tx) as usize)
        } else {
            None
        }
    }

    pub fn reale(&self, tx: i32, ty: i32) -> Reale {
        match Self::indice(tx, ty) {
            Some(i) => self.reale[i],
            // Sotto la stanza c'è il vuoto (si cade); ai lati e sopra, roccia.
            None if ty >= RIGHE => Reale::Vuoto,
            None => Reale::Roccia,
        }
    }

    pub fn cella(&self, tx: i32, ty: i32) -> Option<Cella> {
        Self::indice(tx, ty).and_then(|i| self.mappa[i])
    }

    /// Quanto è rivelato il punto `p`: 0 = per niente, 1 = in pieno.
    pub fn rivelato(&self, p: Vec2) -> f32 {
        let mut v: f32 = 0.0;
        for r in &self.rivelazioni {
            let raggio = r.raggio();
            if raggio <= 0.0 {
                continue;
            }
            let d = (p - r.centro).len();
            v = v.max(((raggio - d) / 10.0).clamp(0.0, 1.0));
        }
        v
    }

    /// Cosa trova il corpo nella cella (tx, ty), se qualcosa regge.
    /// Regge una lettera nominata, oppure la realtà se è rivelata.
    pub fn solido(&self, tx: i32, ty: i32) -> Option<Natura> {
        if let Some(c) = self.cella(tx, ty) {
            if c.natura.regge() {
                return Some(c.natura);
            }
        }
        if self.reale(tx, ty).regge() {
            let centro = Vec2::new((tx * TILE + TILE / 2) as f32, (ty * TILE + TILE / 2) as f32);
            if self.rivelato(centro) > 0.5 {
                return Some(Natura::Solida);
            }
        }
        None
    }

    /// `true` se nella colonna `tx`, dalla riga `da` in giù, non c'è niente
    /// che regga: lì si precipita.
    pub fn colonna_vuota(&self, tx: i32, da: i32) -> bool {
        (da.max(0)..RIGHE).all(|ty| self.solido(tx, ty).is_none())
    }

    /// Il percetto con proposte aperte che occupa la cella (tx, ty), se c'è
    /// (le parole proposte si vedono ma non reggono).
    pub fn proposta_in(&self, tx: i32, ty: i32) -> Option<(usize, Voce)> {
        for (i, p) in self.percetti.iter().enumerate() {
            if p.fase != Fase::Proposto {
                continue;
            }
            for (j, pr) in p.def.proposte.iter().enumerate() {
                for k in 0..pr.celle {
                    if pr.cella(k) == (tx, ty) && p.lettera(j, k, 0) != ' ' {
                        return Some((i, pr.voce));
                    }
                }
            }
        }
        None
    }

    /// Ricostruisce la mappa delle lettere nominate.
    pub fn ricostruisci(&mut self) {
        if !self.sporca {
            return;
        }
        self.mappa.fill(None);
        for (pi, p) in self.percetti.iter().enumerate() {
            if p.fase != Fase::Affermato {
                continue;
            }
            let pr = &p.def.proposte[p.scelta];
            let natura = pr.lemma.stati[p.stato].natura;
            for k in 0..pr.celle {
                let lettera = p.lettera(p.scelta, k, p.stato);
                if lettera == ' ' {
                    continue;
                }
                let (tx, ty) = pr.cella(k);
                if let Some(i) = Self::indice(tx, ty) {
                    self.mappa[i] = Some(Cella { lettera, natura, voce: pr.voce, percetto: pi as u16 });
                }
            }
        }
        self.sporca = false;
    }

    /// Un passo di simulazione: sensi, decadimento, rivelazioni.
    ///
    /// Gli eventi si accumulano in `self.eventi`: li consuma (e svuota) il gioco.
    pub fn update(&mut self, dt: f32, corpo: Rect) {
        let centro = corpo.center();

        for (pi, p) in self.percetti.iter_mut().enumerate() {
            let vicino = distanza_rect(centro, &p.rect()) < DIST_SENSI;
            match p.fase {
                Fase::Ignoto if vicino => p.fase = Fase::Sentito(0.0),
                Fase::Sentito(t) => {
                    if !vicino {
                        p.fase = Fase::Ignoto;
                    } else if t + dt >= TEMPO_SENSI {
                        p.fase = Fase::Proposto;
                        p.focus = 0;
                    } else {
                        p.fase = Fase::Sentito(t + dt);
                    }
                }
                Fase::Lasciato(t) => {
                    p.fase = if t - dt <= 0.0 { Fase::Ignoto } else { Fase::Lasciato(t - dt) };
                }
                Fase::Affermato => {
                    let pr = &p.def.proposte[p.scelta];
                    let lemma = pr.lemma;
                    let uso = if p.calpestato { lemma.logorio_uso } else { 0.0 };
                    p.certezza = (p.certezza - (lemma.logorio + uso) * dt).max(0.0);
                    let nuovo = lemma.stato_per(p.certezza);
                    if nuovo != p.stato {
                        // Le lettere che cambiano o spariscono cadono.
                        for k in 0..pr.celle {
                            let prima = p.lettera(p.scelta, k, p.stato);
                            let dopo = p.lettera(p.scelta, k, nuovo);
                            if prima != dopo && prima != ' ' {
                                let (tx, ty) = pr.cella(k);
                                let pos = Vec2::new((tx * TILE) as f32 + 1.0, (ty * TILE) as f32 + 1.0);
                                self.eventi.push(Evento::LetteraCaduta { pos, lettera: prima, voce: pr.voce });
                            }
                        }
                        p.stato = nuovo;
                        self.sporca = true;
                        self.eventi.push(Evento::Mutazione { percetto: pi as u16 });
                    }
                }
                _ => {}
            }
            p.calpestato = false;
        }

        for r in &mut self.rivelazioni {
            r.t += dt;
        }
        self.rivelazioni.retain(|r| r.t < RIVELAZIONE_DURATA);
        self.ricostruisci();
    }

    /// Segna che il protagonista sta sopra il percetto `i` (consuma la parola).
    pub fn calpesta(&mut self, i: u16) {
        if let Some(p) = self.percetti.get_mut(i as usize) {
            p.calpestato = true;
        }
    }

    /// Cambia la voce in primo piano.
    pub fn prossima_voce(&mut self, i: usize) {
        let p = &mut self.percetti[i];
        if p.fase == Fase::Proposto {
            p.focus = (p.focus + 1) % p.def.proposte.len();
        }
    }

    /// Afferma la proposta in primo piano. Restituisce `false` se il nome
    /// non può essere affermato perché occuperebbe il corpo del protagonista.
    pub fn afferma(&mut self, i: usize, corpo: Rect) -> bool {
        let p = &self.percetti[i];
        if p.fase != Fase::Proposto {
            return false;
        }
        let pr = &p.def.proposte[p.focus];
        if pr.lemma.stati[0].natura.regge() {
            for k in 0..pr.celle {
                let (tx, ty) = pr.cella(k);
                let r = Rect::new((tx * TILE) as f32, (ty * TILE) as f32, TILE as f32, TILE as f32);
                if r.overlaps(&corpo) && p.lettera(p.focus, k, 0) != ' ' {
                    return false;
                }
            }
        }
        let p = &mut self.percetti[i];
        p.fase = Fase::Affermato;
        p.scelta = p.focus;
        p.certezza = 1.0;
        p.stato = 0;
        self.sporca = true;
        self.ricostruisci();
        true
    }

    /// Lascia andare: la parola (o le proposte) si sciolgono e la realtà
    /// sotto si rivela per qualche secondo.
    pub fn lascia(&mut self, i: usize) {
        let p = &self.percetti[i];
        let indici: &[usize] = match p.fase {
            Fase::Affermato => std::slice::from_ref(&p.scelta),
            Fase::Proposto => &[0, 1, 2],
            _ => return,
        };
        for &pi in indici.iter().filter(|&&pi| pi < p.def.proposte.len()) {
            let pr = &p.def.proposte[pi];
            let stato = if p.fase == Fase::Affermato { p.stato } else { 0 };
            for k in 0..pr.celle {
                let lettera = p.lettera(pi, k, stato);
                if lettera != ' ' {
                    let (tx, ty) = pr.cella(k);
                    let pos = Vec2::new((tx * TILE) as f32 + 1.0, (ty * TILE) as f32 + 1.0);
                    self.eventi.push(Evento::LetteraSciolta { pos, lettera, voce: pr.voce });
                }
            }
        }
        let centro = p.rect().center();
        let p = &mut self.percetti[i];
        p.fase = Fase::Lasciato(RIVELAZIONE_DURATA + 1.0);
        self.rivelazioni.push(Rivelazione { centro, t: 0.0 });
        self.sporca = true;
        self.ricostruisci();
    }
}

/// Distanza da un punto al rettangolo (0 se è dentro).
pub fn distanza_rect(p: Vec2, r: &Rect) -> f32 {
    let dx = (r.x - p.x).max(0.0).max(p.x - r.right());
    let dy = (r.y - p.y).max(0.0).max(p.y - r.bottom());
    (dx * dx + dy * dy).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::game::livello::{ABISSO as STANZA_ABISSO, ATRIO};

    const ABISSO: usize = 2;
    const VUOTO_ATRIO: usize = 2;

    fn corpo_a(x: f32, y: f32) -> Rect {
        Rect::new(x, y, 6.0, 12.0)
    }

    #[test]
    fn il_pavimento_iniziale_regge_l_abisso_no() {
        let m = Mondo::new(&STANZA_ABISSO);
        assert!(m.solido(3, 17).is_some(), "PAVIMENTO a sinistra");
        assert!(m.solido(18, 17).is_none(), "l'abisso non nominato non regge");
        assert!(m.solido(18, 18).is_none(), "il gradino reale non si tocca senza rivelazione");
    }

    #[test]
    fn avvicinarsi_fa_nascere_le_proposte() {
        let mut m = Mondo::new(&STANZA_ABISSO);
        let corpo = corpo_a(112.0, 124.0); // sul bordo dell'abisso
        for _ in 0..70 {
            m.update(1.0 / 60.0, corpo);
        }
        assert_eq!(m.percetti[ABISSO].fase, Fase::Proposto);
    }

    #[test]
    fn affermare_il_ponte_lo_rende_solido_e_l_uso_lo_consuma() {
        let mut m = Mondo::new(&STANZA_ABISSO);
        m.percetti[ABISSO].fase = Fase::Proposto;
        assert!(m.afferma(ABISSO, corpo_a(32.0, 124.0)));
        assert!(m.solido(18, 17).is_some());
        // Starci sopra a lungo: il PONTE diventa POSTE e non regge più.
        let mut cadute = 0;
        for _ in 0..600 {
            m.calpesta(ABISSO as u16);
            m.update(1.0 / 60.0, corpo_a(144.0, 124.0));
            cadute += m.eventi.drain(..).filter(|e| matches!(e, Evento::LetteraCaduta { .. })).count();
        }
        assert!(cadute > 0, "qualche lettera deve cadere");
        assert!(m.solido(18, 17).is_none(), "POSTE non regge");
    }

    #[test]
    fn lasciar_andare_rivela_il_gradino() {
        let mut m = Mondo::new(&STANZA_ABISSO);
        m.percetti[ABISSO].fase = Fase::Proposto;
        m.lascia(ABISSO);
        for _ in 0..30 {
            m.update(1.0 / 60.0, corpo_a(32.0, 124.0));
        }
        assert!(m.solido(19, 18).is_some(), "il gradino reale ora regge");
        for _ in 0..400 {
            m.update(1.0 / 60.0, corpo_a(32.0, 124.0));
        }
        assert!(m.solido(19, 18).is_none(), "la rivelazione si richiude");
    }

    #[test]
    fn nell_atrio_il_vuoto_regge_solo_dopo_averlo_detto() {
        let mut m = Mondo::new(&ATRIO);
        assert!(m.solido(16, 17).is_none());
        assert!(m.colonna_vuota(16, 17), "sotto il vuoto non c'è niente");
        m.percetti[VUOTO_ATRIO].fase = Fase::Proposto;
        assert!(m.proposta_in(16, 17).is_some(), "la proposta si vede...");
        assert!(m.solido(16, 17).is_none(), "...ma non regge");
        assert!(m.afferma(VUOTO_ATRIO, corpo_a(32.0, 124.0)));
        assert!(m.solido(16, 17).is_some());
        assert!(!m.colonna_vuota(16, 17));
    }

    #[test]
    fn non_si_nomina_un_muro_addosso() {
        let mut m = Mondo::new(&STANZA_ABISSO);
        m.percetti[ABISSO].fase = Fase::Proposto;
        // Il corpo è dove andrebbe il PONTE.
        assert!(!m.afferma(ABISSO, corpo_a(140.0, 132.0)));
    }
}
