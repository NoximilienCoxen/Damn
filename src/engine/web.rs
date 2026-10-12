//! Il guscio web: il gioco compilato in WebAssembly, pilotato da JavaScript.
//!
//! Niente librerie (né wasm-bindgen né altro): esportiamo poche funzioni
//! "C" che la pagina chiama direttamente.
//!
//! - `egresso_init()`               crea il gioco;
//! - `egresso_key(codice, giu)`     un tasto (o pulsante touch) giù o su;
//! - `egresso_frame(ms)`            avanza il tempo e disegna;
//! - `egresso_pixels()`             puntatore ai pixel RGBA (320×180×4 byte);
//! - `egresso_comandi()`            quali pulsanti mostrare (maschera di bit).
//!
//! Il JS crea un `ImageData` direttamente sulla memoria del modulo wasm,
//! quindi i pixel non vengono mai copiati due volte.
//!
//! Il gioco concreto viene agganciato con la macro `esporta_web!` (in
//! `lib.rs`), così l'engine continua a non sapere chi sia.

use super::app::{Game, Stats, ALTEZZA, LARGHEZZA};
use super::framebuffer::{canali, Framebuffer};
use super::input::{Input, Key};
use super::time::FixedStep;

/// Tutto lo stato del guscio web.
pub struct Web<G: Game> {
    pub game: G,
    fb: Framebuffer,
    rgba: Vec<u8>,
    input: Input,
    passo: FixedStep,
    stats: Stats,
}

impl<G: Game> Web<G> {
    pub fn new(game: G) -> Web<G> {
        Web {
            game,
            fb: Framebuffer::new(LARGHEZZA, ALTEZZA),
            rgba: vec![255; LARGHEZZA * ALTEZZA * 4],
            input: Input::new(),
            passo: FixedStep::default(),
            stats: Stats { fps: 60.0, ..Stats::default() },
        }
    }

    pub fn key(&mut self, codice: u32, giu: bool) {
        if let Some(k) = tasto(codice) {
            self.input.set(k, giu);
        }
    }

    pub fn frame(&mut self, ms: f32) {
        let dt = (ms / 1000.0).max(0.0);
        if dt > 0.0 {
            self.stats.fps = self.stats.fps * 0.95 + (1.0 / dt) * 0.05;
        }
        for _ in 0..self.passo.avanza(dt) {
            self.game.update(&self.input);
            self.input.fine_passo();
        }
        self.game.render(&mut self.fb, &self.stats);
        // 0x00RRGGBB → byte R, G, B, A (A resta sempre 255).
        for (px, out) in self.fb.pixel().iter().zip(self.rgba.chunks_exact_mut(4)) {
            let (r, g, b) = canali(*px);
            out[0] = r;
            out[1] = g;
            out[2] = b;
        }
    }

    pub fn comandi(&self) -> u32 {
        self.game.comandi()
    }

    pub fn pixels(&self) -> *const u8 {
        self.rgba.as_ptr()
    }
}

/// I codici dei tasti usati dalla pagina web (devono coincidere con il JS).
pub fn tasto(codice: u32) -> Option<Key> {
    Some(match codice {
        0 => Key::Left,
        1 => Key::Right,
        2 => Key::Up,
        3 => Key::Down,
        4 => Key::Space,
        5 => Key::E,
        6 => Key::Q,
        7 => Key::R,
        8 => Key::Enter,
        9 => Key::F3,
        _ => return None,
    })
}

/// Esporta le funzioni del guscio web per il gioco costruito da `$nuovo`.
#[macro_export]
macro_rules! esporta_web {
    ($tipo:ty, $nuovo:expr) => {
        thread_local! {
            static WEB: ::std::cell::RefCell<Option<$crate::engine::web::Web<$tipo>>> =
                const { ::std::cell::RefCell::new(None) };
        }

        #[no_mangle]
        pub extern "C" fn egresso_init() {
            WEB.with(|w| *w.borrow_mut() = Some($crate::engine::web::Web::new($nuovo)));
        }

        #[no_mangle]
        pub extern "C" fn egresso_key(codice: u32, giu: u32) {
            WEB.with(|w| {
                if let Some(w) = w.borrow_mut().as_mut() {
                    w.key(codice, giu != 0)
                }
            });
        }

        #[no_mangle]
        pub extern "C" fn egresso_frame(ms: f32) {
            WEB.with(|w| {
                if let Some(w) = w.borrow_mut().as_mut() {
                    w.frame(ms)
                }
            });
        }

        #[no_mangle]
        pub extern "C" fn egresso_comandi() -> u32 {
            WEB.with(|w| w.borrow().as_ref().map_or(u32::MAX, |w| w.comandi()))
        }

        #[no_mangle]
        pub extern "C" fn egresso_pixels() -> *const u8 {
            WEB.with(|w| w.borrow().as_ref().map_or(::std::ptr::null(), |w| w.pixels()))
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Finto(u32);
    impl Game for Finto {
        fn update(&mut self, input: &Input) {
            if input.down(Key::Right) {
                self.0 += 1;
            }
        }
        fn render(&mut self, fb: &mut Framebuffer, _: &Stats) {
            fb.clear(0x00ff8000);
        }
    }

    #[test]
    fn frame_converte_in_rgba_e_inoltra_i_tasti() {
        let mut w = Web::new(Finto(0));
        w.key(1, true);
        w.frame(1000.0 / 30.0 + 1.0); // poco più di due passi da 1/60
        assert_eq!(w.game.0, 2);
        assert_eq!(&w.rgba[0..4], &[255, 128, 0, 255]);
    }
}
