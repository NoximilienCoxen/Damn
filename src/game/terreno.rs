//! Il mondo di fuori: una striscia di terra che gira su se stessa (chi
//! cammina sempre dritto torna dove è partito) e, da qualche parte, ogni
//! partita in un posto diverso, Egresso.
//!
//! Tutto è una funzione della x (e del seme): niente mappe da caricare,
//! niente memoria che cresce. Le x del mondo sono illimitate; il terreno
//! si ripete ogni `GIRO` pixel.

use crate::engine::math::hash01;

/// Lunghezza del mondo prima che si ripeta (px).
pub const GIRO: f32 = 3600.0;
/// Mezza larghezza della piazza davanti all'arco.
pub const PIAZZA: f32 = 110.0;
/// Quanto la collina di Egresso sta sopra la piazza.
pub const COLLINA: f32 = 50.0;
/// Da quanto lontano Egresso si fa sentire (vento, lucciole, rovine).
pub const RICHIAMO: f32 = 1300.0;
/// Larghezza delle celle in cui possono esserci rovine o alberi.
const CELLA_ROVINE: f32 = 48.0;
const CELLE_ROVINE: i32 = (GIRO / CELLA_ROVINE) as i32;
const CELLA_ALBERI: f32 = 40.0;
const CELLE_ALBERI: i32 = (GIRO / CELLA_ALBERI) as i32;

/// La parola incisa sulle pietre.
pub const NOME: &str = "EGRESSO";

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Rovina {
    /// Una colonna spezzata, alta `h`.
    Colonna { x: f32, h: f32 },
    /// Un blocco caduto.
    Blocco { x: f32, w: f32, h: f32 },
    /// Una lapide con una lettera incisa.
    Lapide { x: f32, lettera: char },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Albero {
    Pino { x: f32, h: f32 },
    Chioma { x: f32, r: f32 },
}

pub struct Terreno {
    pub seme: u32,
    /// Dove sta l'arco (x nel primo giro).
    pub arco: f32,
    /// Da che parte dell'arco sale la collina (-1 o 1).
    pub lato: f32,
    pub piazza_y: f32,
    pub partenza: f32,
}

/// Smussa 0..1 in modo che inizio e fine siano piatti.
pub fn liscio(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Rumore 1D tra -1 e 1, che si ripete ogni `periodo` unità.
fn rumore_periodico(x: f32, periodo: i32, seme: u32) -> f32 {
    let i = x.floor() as i32;
    let f = x - i as f32;
    let a = hash01(i.rem_euclid(periodo), 0, seme);
    let b = hash01((i + 1).rem_euclid(periodo), 0, seme);
    (a + (b - a) * liscio(f)) * 2.0 - 1.0
}

/// Rumore 1D tra -1 e 1, senza ripetizione (per gli sfondi).
pub fn rumore(x: f32, seme: u32) -> f32 {
    let i = x.floor() as i32;
    let f = x - i as f32;
    let a = hash01(i, 0, seme);
    let b = hash01(i + 1, 0, seme);
    (a + (b - a) * liscio(f)) * 2.0 - 1.0
}

/// Distanza con segno più breve da `a` a `b`, tenendo conto del giro.
pub fn delta(a: f32, b: f32) -> f32 {
    (b - a + GIRO * 0.5).rem_euclid(GIRO) - GIRO * 0.5
}

impl Terreno {
    pub fn new(seme: u32) -> Terreno {
        let h = |k: u32| hash01(seme as i32, k as i32, 0x9e37);
        let partenza = GIRO * 0.5;
        let lato = if h(1) < 0.5 { -1.0 } else { 1.0 };
        let verso = if h(2) < 0.5 { -1.0 } else { 1.0 };
        let distanza = 900.0 + h(3) * 700.0;
        let arco = (partenza + verso * distanza).rem_euclid(GIRO);
        let mut t = Terreno { seme, arco, lato, piazza_y: 0.0, partenza };
        t.piazza_y = t.naturale(arco).min(140.0);
        t
    }

    /// Il terreno com'è senza Egresso: colline basse, irregolari.
    pub fn naturale(&self, x: f32) -> f32 {
        let x = x.rem_euclid(GIRO);
        let s = self.seme;
        132.0
            + rumore_periodico(x / 240.0, 15, s) * 18.0
            + rumore_periodico(x / 80.0, 45, s ^ 0x51) * 7.0
            + rumore_periodico(x / 30.0, 120, s ^ 0xa3) * 2.0
    }

    /// Distanza dall'arco, positiva dalla parte della collina.
    pub fn verso_collina(&self, x: f32) -> f32 {
        delta(self.arco, x) * self.lato
    }

    /// L'altezza del suolo (y del bordo superiore) alla x data.
    pub fn suolo(&self, x: f32) -> f32 {
        let d = self.verso_collina(x);
        let py = self.piazza_y;
        let n = self.naturale(x);
        if d.abs() <= PIAZZA {
            py
        } else if d > PIAZZA {
            // La collina: un fronte di roccia, la cima, poi scende dolce.
            let cima = py - COLLINA;
            let k = liscio((d - PIAZZA - 120.0) / 200.0);
            cima + (n - cima) * k
        } else {
            let k = liscio((-d - PIAZZA) / 120.0);
            py + (n - py) * k
        }
    }

    /// 1 sull'arco, 0 da `RICHIAMO` in là.
    pub fn vicinanza(&self, x: f32) -> f32 {
        (1.0 - delta(self.arco, x).abs() / RICHIAMO).clamp(0.0, 1.0)
    }

    /// Il vento alla x data: una brezza qualunque che, vicino a Egresso,
    /// diventa un respiro che tira verso l'arco.
    pub fn vento(&self, x: f32, t: f32) -> f32 {
        let v = self.vicinanza(x);
        let brezza = ((t * 0.37).sin() * 0.6 + (t * 1.1 + x * 0.003).sin() * 0.4) * 0.35;
        let verso = delta(x, self.arco).signum();
        // Respira: tira a ondate.
        let respiro = 0.75 + 0.25 * (t * 0.8).sin();
        brezza * (1.0 - v) + verso * v.powf(1.4) * 1.3 * respiro
    }

    /// La rovina nella cella `i`, se c'è. Più vicino a Egresso, più rovine.
    pub fn rovina(&self, i: i32) -> Option<Rovina> {
        let i = i.rem_euclid(CELLE_ROVINE);
        let h = |k: i32| hash01(i, k, self.seme ^ 0x7f4a);
        let x = (i as f32 + 0.2 + 0.6 * h(1)) * CELLA_ROVINE;
        let v = self.vicinanza(x);
        // Né sulla piazza né sulla collina.
        if self.verso_collina(x).abs() < PIAZZA + 340.0 && self.verso_collina(x) > -PIAZZA - 30.0 {
            return None;
        }
        if h(2) > 0.015 + 0.5 * v * v {
            return None;
        }
        let tipo = h(3);
        Some(if tipo < 0.4 {
            Rovina::Colonna { x, h: 10.0 + h(4) * 16.0 }
        } else if tipo < 0.7 || v < 0.35 {
            Rovina::Blocco { x, w: 8.0 + h(4) * 6.0, h: 5.0 + h(5) * 3.0 }
        } else {
            let k = (h(6) * NOME.len() as f32) as usize % NOME.len();
            Rovina::Lapide { x, lettera: NOME.as_bytes()[k] as char }
        })
    }

    pub fn rovine_tra(&self, x0: f32, x1: f32) -> impl Iterator<Item = (f32, Rovina)> + '_ {
        let (i0, i1) = ((x0 / CELLA_ROVINE).floor() as i32 - 1, (x1 / CELLA_ROVINE).floor() as i32 + 1);
        (i0..=i1).filter_map(move |i| {
            // La x nel primo giro, riportata vicino a x0.
            let giro = (i as f32 * CELLA_ROVINE / GIRO).floor() * GIRO;
            self.rovina(i).map(|r| (giro, r))
        })
    }

    /// L'albero nella cella `i`, se c'è. Vicino a Egresso non crescono.
    pub fn albero(&self, i: i32) -> Option<Albero> {
        let i = i.rem_euclid(CELLE_ALBERI);
        let h = |k: i32| hash01(i, k, self.seme ^ 0x3c11);
        let x = (i as f32 + 0.15 + 0.7 * h(1)) * CELLA_ALBERI;
        if self.vicinanza(x) > 0.8 || h(2) > 0.42 {
            return None;
        }
        Some(if h(3) < 0.55 { Albero::Pino { x, h: 26.0 + h(4) * 30.0 } } else { Albero::Chioma { x, r: 9.0 + h(4) * 8.0 } })
    }

    pub fn alberi_tra(&self, x0: f32, x1: f32) -> impl Iterator<Item = (f32, Albero)> + '_ {
        let (i0, i1) = ((x0 / CELLA_ALBERI).floor() as i32 - 2, (x1 / CELLA_ALBERI).floor() as i32 + 2);
        (i0..=i1).filter_map(move |i| {
            let giro = (i as f32 * CELLA_ALBERI / GIRO).floor() * GIRO;
            self.albero(i).map(|a| (giro, a))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn il_mondo_gira_su_se_stesso() {
        let t = Terreno::new(7);
        for k in 0..50 {
            let x = k as f32 * 71.3;
            assert!((t.suolo(x) - t.suolo(x + GIRO)).abs() < 0.01);
        }
    }

    #[test]
    fn egresso_e_lontano_dalla_partenza_e_cambia_con_il_seme() {
        let mut posti = Vec::new();
        for s in 0..20 {
            let t = Terreno::new(s);
            let d = delta(t.partenza, t.arco).abs();
            assert!((900.0..=1600.0).contains(&d), "seme {s}: {d}");
            posti.push((t.arco / 100.0) as i32);
        }
        posti.sort();
        posti.dedup();
        assert!(posti.len() > 5);
    }

    #[test]
    fn la_piazza_e_piana_e_la_collina_sta_sopra() {
        let t = Terreno::new(3);
        let a = t.arco;
        assert_eq!(t.suolo(a - 50.0), t.piazza_y);
        assert_eq!(t.suolo(a + 50.0), t.piazza_y);
        assert_eq!(t.suolo(a + t.lato * (PIAZZA + 40.0)), t.piazza_y - COLLINA);
    }

    #[test]
    fn il_vento_tira_verso_l_arco_da_vicino() {
        let t = Terreno::new(5);
        for k in 0..20 {
            let tt = k as f32 * 0.7;
            assert!(t.vento(t.arco - 300.0, tt) > 0.2);
            assert!(t.vento(t.arco + 300.0, tt) < -0.2);
        }
    }

    #[test]
    fn piu_vicino_piu_rovine() {
        let t = Terreno::new(11);
        let conta = |da: f32, a: f32| t.rovine_tra(da, a).count();
        let lato = -t.lato;
        let vicino = conta(t.arco + lato * 1000.0, t.arco + lato * 460.0).max(conta(t.arco + lato * 460.0, t.arco + lato * 1000.0));
        let lontano = conta(t.arco + 1700.0, t.arco + 2300.0);
        assert!(vicino > lontano, "vicino {vicino}, lontano {lontano}");
    }
}
