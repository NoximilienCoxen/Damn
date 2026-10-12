//! Input: quali tasti sono giù, e quali sono stati appena premuti o rilasciati.
//!
//! Il game loop aggiorna il mondo a passo fisso (60 volte al secondo), ma gli
//! eventi della tastiera arrivano quando vogliono. Per non perdere una
//! pressione rapidissima la "memorizziamo" (latch) finché un passo di update
//! non l'ha vista: poi `fine_passo` la cancella.

/// I tasti che l'engine conosce. Il gioco decide cosa significano.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Left,
    Right,
    Up,
    Down,
    Space,
    A,
    D,
    E,
    Q,
    R,
    W,
    S,
    Enter,
    Escape,
    F3,
}

const N: usize = Key::F3 as usize + 1;

impl Key {
    /// Nome testuale (usato dagli script di input headless).
    pub fn from_name(s: &str) -> Option<Key> {
        Some(match s.to_ascii_lowercase().as_str() {
            "left" => Key::Left,
            "right" => Key::Right,
            "up" => Key::Up,
            "down" => Key::Down,
            "space" => Key::Space,
            "a" => Key::A,
            "d" => Key::D,
            "e" => Key::E,
            "q" => Key::Q,
            "r" => Key::R,
            "w" => Key::W,
            "s" => Key::S,
            "enter" => Key::Enter,
            "escape" => Key::Escape,
            "f3" => Key::F3,
            _ => return None,
        })
    }
}

#[derive(Default)]
pub struct Input {
    giu: [bool; N],
    premuti: [bool; N],
    rilasciati: [bool; N],
    /// Un tasto qualsiasi premuto in questo passo (serve al finale "senza input").
    qualcosa: bool,
}

impl Input {
    pub fn new() -> Input {
        Input::default()
    }

    /// Chiamato dalla finestra (o dallo script headless) a ogni evento.
    pub fn set(&mut self, k: Key, giu: bool) {
        let i = k as usize;
        if giu && !self.giu[i] {
            self.premuti[i] = true;
            self.qualcosa = true;
        }
        if !giu && self.giu[i] {
            self.rilasciati[i] = true;
        }
        self.giu[i] = giu;
    }

    pub fn down(&self, k: Key) -> bool {
        self.giu[k as usize]
    }

    pub fn pressed(&self, k: Key) -> bool {
        self.premuti[k as usize]
    }

    pub fn released(&self, k: Key) -> bool {
        self.rilasciati[k as usize]
    }

    /// Un qualunque tasto premuto in questo passo.
    pub fn any_pressed(&self) -> bool {
        self.qualcosa
    }

    /// Da chiamare dopo ogni passo di update.
    pub fn fine_passo(&mut self) {
        self.premuti = [false; N];
        self.rilasciati = [false; N];
        self.qualcosa = false;
    }

    /// Rilascia tutto (es. quando la finestra perde il focus).
    pub fn reset(&mut self) {
        *self = Input::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn premuto_dura_un_solo_passo() {
        let mut i = Input::new();
        i.set(Key::E, true);
        assert!(i.pressed(Key::E) && i.down(Key::E));
        i.fine_passo();
        assert!(!i.pressed(Key::E) && i.down(Key::E));
    }

    #[test]
    fn pressione_rapidissima_non_si_perde() {
        // Premuto e rilasciato prima che l'update giri: lo vediamo comunque.
        let mut i = Input::new();
        i.set(Key::Space, true);
        i.set(Key::Space, false);
        assert!(i.pressed(Key::Space));
        assert!(!i.down(Key::Space));
    }
}
