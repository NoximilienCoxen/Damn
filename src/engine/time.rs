//! Il passo fisso del game loop.
//!
//! Lo schermo può andare a 60, 144 o 30 fps, ma la fisica deve comportarsi
//! sempre allo stesso modo. Allora accumuliamo il tempo reale trascorso e lo
//! "spendiamo" a fette fisse di 1/60 di secondo: zero, uno o più update per
//! frame a seconda di quanto tempo è passato.

pub const DT: f32 = 1.0 / 60.0;

/// Oltre questo numero di passi per frame smettiamo di inseguire il tempo
/// perso (es. finestra trascinata): meglio rallentare che bloccarsi.
const MAX_PASSI: u32 = 5;

#[derive(Default)]
pub struct FixedStep {
    accumulatore: f32,
}

impl FixedStep {
    /// Aggiunge il tempo del frame e restituisce quanti update eseguire.
    pub fn avanza(&mut self, secondi: f32) -> u32 {
        self.accumulatore += secondi.min(0.25);
        let mut passi = 0;
        while self.accumulatore >= DT && passi < MAX_PASSI {
            self.accumulatore -= DT;
            passi += 1;
        }
        if passi == MAX_PASSI {
            self.accumulatore = 0.0;
        }
        passi
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spende_il_tempo_a_fette_fisse() {
        let mut f = FixedStep::default();
        assert_eq!(f.avanza(DT * 0.5), 0);
        assert_eq!(f.avanza(DT * 0.6), 1); // 1.1 passi accumulati → 1
        assert_eq!(f.avanza(DT * 2.0), 2);
        assert_eq!(f.avanza(10.0), MAX_PASSI);
    }
}
