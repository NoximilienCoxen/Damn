//! Il "guscio" dell'engine: apre la finestra, fa girare il game loop e
//! copia il framebuffer sullo schermo. Oppure, in modalità headless, fa
//! girare il gioco senza finestra con un input registrato e salva screenshot.
//!
//! Il gioco vero implementa il trait `Game`: l'engine non sa niente di parole
//! o di occhi chiusi.

use std::num::NonZeroU32;
use std::rc::Rc;
use std::time::Instant;

use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use super::framebuffer::Framebuffer;
use super::input::{Input, Key};
use super::time::FixedStep;

pub const LARGHEZZA: usize = 320;
pub const ALTEZZA: usize = 180;

/// Tempi dell'ultimo frame, per l'overlay di debug.
#[derive(Clone, Copy, Default)]
pub struct Stats {
    pub ms_update: f32,
    pub ms_render: f32,
    pub fps: f32,
}

pub trait Game {
    /// Un passo di simulazione, lungo sempre `time::DT`.
    fn update(&mut self, input: &Input);
    /// Disegna lo stato attuale.
    fn render(&mut self, fb: &mut Framebuffer, stats: &Stats);
    /// `true` per chiudere il programma.
    fn quit(&self) -> bool {
        false
    }
    /// Una riga di stato per `--trace` in headless (es. posizione del giocatore).
    fn traccia(&self) -> String {
        String::new()
    }
}

// ------------------------------------------------------------------ finestra

pub fn run_window<G: Game + 'static>(game: G, titolo: &str) {
    let event_loop = EventLoop::new().expect("impossibile creare l'event loop");
    let mut app = App {
        game,
        titolo: titolo.to_string(),
        window: None,
        surface: None,
        fb: Framebuffer::new(LARGHEZZA, ALTEZZA),
        input: Input::new(),
        passo: FixedStep::default(),
        ultimo: Instant::now(),
        stats: Stats::default(),
    };
    event_loop.run_app(&mut app).expect("errore nell'event loop");
}

type Surface = softbuffer::Surface<Rc<Window>, Rc<Window>>;

struct App<G: Game> {
    game: G,
    titolo: String,
    window: Option<Rc<Window>>,
    surface: Option<Surface>,
    fb: Framebuffer,
    input: Input,
    passo: FixedStep,
    ultimo: Instant,
    stats: Stats,
}

impl<G: Game> ApplicationHandler for App<G> {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attr = Window::default_attributes()
            .with_title(&self.titolo)
            .with_inner_size(LogicalSize::new(LARGHEZZA as f64 * 3.0, ALTEZZA as f64 * 3.0))
            .with_min_inner_size(LogicalSize::new(LARGHEZZA as f64, ALTEZZA as f64));
        let window = Rc::new(el.create_window(attr).expect("impossibile aprire la finestra"));
        let context = softbuffer::Context::new(window.clone()).expect("softbuffer: contesto");
        let surface = softbuffer::Surface::new(&context, window.clone()).expect("softbuffer: superficie");
        self.window = Some(window);
        self.surface = Some(surface);
        self.ultimo = Instant::now();
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::Focused(false) => self.input.reset(),
            WindowEvent::KeyboardInput {
                event: KeyEvent { physical_key: PhysicalKey::Code(code), state, repeat: false, .. },
                ..
            } => {
                if let Some(k) = mappa_tasto(code) {
                    self.input.set(k, state == ElementState::Pressed);
                }
            }
            WindowEvent::RedrawRequested => {
                self.frame();
                if self.game.quit() {
                    el.exit();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _el: &ActiveEventLoop) {
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }
}

impl<G: Game> App<G> {
    fn frame(&mut self) {
        let ora = Instant::now();
        let dt = (ora - self.ultimo).as_secs_f32();
        self.ultimo = ora;
        self.stats.fps = self.stats.fps * 0.95 + (1.0 / dt.max(1e-4)) * 0.05;

        let t0 = Instant::now();
        for _ in 0..self.passo.avanza(dt) {
            self.game.update(&self.input);
            self.input.fine_passo();
        }
        let t1 = Instant::now();
        self.game.render(&mut self.fb, &self.stats);
        let t2 = Instant::now();
        self.stats.ms_update = self.stats.ms_update * 0.9 + (t1 - t0).as_secs_f32() * 100.0;
        self.stats.ms_render = self.stats.ms_render * 0.9 + (t2 - t1).as_secs_f32() * 100.0;

        let (Some(window), Some(surface)) = (&self.window, &mut self.surface) else { return };
        let size = window.inner_size();
        let (Some(w), Some(h)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) else { return };
        surface.resize(w, h).expect("softbuffer: resize");
        let mut buffer = surface.buffer_mut().expect("softbuffer: buffer");
        scala_su(&self.fb, &mut buffer, size.width as usize, size.height as usize);
        buffer.present().expect("softbuffer: present");
    }
}

/// Copia il framebuffer 320×180 sullo schermo con scala intera (pixel
/// perfetti) e bande nere attorno.
fn scala_su(fb: &Framebuffer, dst: &mut [u32], dw: usize, dh: usize) {
    let (sw, sh) = (fb.larghezza(), fb.altezza());
    let scala = (dw / sw).min(dh / sh).max(1);
    let (ow, oh) = (sw * scala, sh * scala);
    let ox = dw.saturating_sub(ow) / 2;
    let oy = dh.saturating_sub(oh) / 2;
    dst.fill(0);
    let src = fb.pixel();
    for sy in 0..sh {
        let y0 = oy + sy * scala;
        if y0 >= dh {
            break;
        }
        // Prima riga ingrandita in orizzontale, poi copiata `scala` volte.
        let riga = &mut dst[y0 * dw..(y0 + 1) * dw];
        for sx in 0..sw {
            let x0 = ox + sx * scala;
            if x0 >= dw {
                break;
            }
            let x1 = (x0 + scala).min(dw);
            riga[x0..x1].fill(src[sy * sw + sx]);
        }
        for k in 1..scala {
            let y = y0 + k;
            if y >= dh {
                break;
            }
            dst.copy_within(y0 * dw..(y0 + 1) * dw, y * dw);
        }
    }
}

fn mappa_tasto(code: KeyCode) -> Option<Key> {
    Some(match code {
        KeyCode::ArrowLeft => Key::Left,
        KeyCode::ArrowRight => Key::Right,
        KeyCode::ArrowUp => Key::Up,
        KeyCode::ArrowDown => Key::Down,
        KeyCode::Space => Key::Space,
        KeyCode::KeyA => Key::A,
        KeyCode::KeyD => Key::D,
        KeyCode::KeyE => Key::E,
        KeyCode::KeyQ => Key::Q,
        KeyCode::KeyR => Key::R,
        KeyCode::KeyW => Key::W,
        KeyCode::KeyS => Key::S,
        KeyCode::Enter => Key::Enter,
        KeyCode::Escape => Key::Escape,
        KeyCode::F3 => Key::F3,
        _ => return None,
    })
}

// ------------------------------------------------------------------ headless

/// Un evento dello script di input: al frame `frame`, il tasto `key` va giù
/// (`Some(true)`), su (`Some(false)`) o viene "toccato" (`None`: giù e su
/// al frame dopo).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EventoScript {
    pub frame: u32,
    pub key: Key,
    pub giu: Option<bool>,
}

/// Legge uno script come `"10:right:+ 40:right:- 41:e:!"`.
/// Separatori ammessi: spazi, virgole, punti e virgola, a capo.
/// `#` inizia un commento fino a fine riga.
pub fn parse_script(testo: &str) -> Result<Vec<EventoScript>, String> {
    let mut out = Vec::new();
    for riga in testo.lines() {
        let riga = riga.split('#').next().unwrap_or("");
        for tok in riga.split(|c: char| c.is_whitespace() || c == ',' || c == ';').filter(|t| !t.is_empty()) {
            let parti: Vec<&str> = tok.split(':').collect();
            if parti.len() != 3 {
                return Err(format!("evento non valido: {tok}"));
            }
            let frame = parti[0].parse().map_err(|_| format!("frame non valido: {tok}"))?;
            let key = Key::from_name(parti[1]).ok_or(format!("tasto sconosciuto: {tok}"))?;
            let giu = match parti[2] {
                "+" => Some(true),
                "-" => Some(false),
                "!" => None,
                _ => return Err(format!("azione non valida (+ - !): {tok}")),
            };
            out.push(EventoScript { frame, key, giu });
        }
    }
    out.sort_by_key(|e| e.frame);
    Ok(out)
}

/// Fa girare il gioco per `frames` passi senza finestra. Salva un PNG in
/// `cartella` per ogni frame in `scatti` (e ogni `ogni` frame, se > 0).
#[allow(clippy::too_many_arguments)]
pub fn run_headless<G: Game>(game: &mut G, frames: u32, script: &[EventoScript], scatti: &[u32], ogni: u32, cartella: &str, traccia: bool) -> std::io::Result<()> {
    std::fs::create_dir_all(cartella)?;
    let mut fb = Framebuffer::new(LARGHEZZA, ALTEZZA);
    let mut input = Input::new();
    let mut da_rilasciare: Vec<Key> = Vec::with_capacity(8);
    let stats = Stats { fps: 60.0, ..Stats::default() };
    let mut prossimo = 0;
    for f in 0..frames {
        for k in da_rilasciare.drain(..) {
            input.set(k, false);
        }
        while prossimo < script.len() && script[prossimo].frame == f {
            let e = script[prossimo];
            match e.giu {
                Some(g) => input.set(e.key, g),
                None => {
                    input.set(e.key, true);
                    da_rilasciare.push(e.key);
                }
            }
            prossimo += 1;
        }
        game.update(&input);
        input.fine_passo();
        if traccia {
            println!("{f:5} {}", game.traccia());
        }
        if scatti.contains(&f) || (ogni > 0 && f % ogni == 0) {
            game.render(&mut fb, &stats);
            super::png::save(&fb, &format!("{cartella}/frame_{f:05}.png"))?;
        }
        if game.quit() {
            break;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_parsing() {
        let s = parse_script("10:right:+ 40:right:-\n# commento\n5:E:!").unwrap();
        assert_eq!(s.len(), 3);
        assert_eq!(s[0], EventoScript { frame: 5, key: Key::E, giu: None });
        assert!(parse_script("10:nonesiste:+").is_err());
    }

    #[test]
    fn scala_intera_centrata() {
        let mut fb = Framebuffer::new(2, 1);
        fb.set_pixel(0, 0, 7);
        fb.set_pixel(1, 0, 9);
        // Schermo 7×3: scala 3 → immagine 6×3, 0 px di margine sopra, 0 a sinistra (7-6)/2=0.
        let mut dst = vec![1u32; 7 * 3];
        scala_su(&fb, &mut dst, 7, 3);
        assert_eq!(&dst[0..7], &[7, 7, 7, 9, 9, 9, 0]);
        assert_eq!(&dst[14..21], &[7, 7, 7, 9, 9, 9, 0]);
    }
}
