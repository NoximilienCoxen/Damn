//! Il guscio desktop: apre la finestra, fa girare il game loop e copia il
//! framebuffer sullo schermo. Usa solo `winit` (finestra ed eventi) e
//! `softbuffer` (un array di pixel da riempire). Non esiste nella build web.

use std::num::NonZeroU32;
use std::rc::Rc;
use std::time::Instant;

use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use super::app::{Game, Stats, ALTEZZA, LARGHEZZA};
use super::framebuffer::Framebuffer;
use super::input::{Input, Key};
use super::time::FixedStep;

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
            WindowEvent::KeyboardInput { event: KeyEvent { physical_key: PhysicalKey::Code(code), state, repeat: false, .. }, .. } => {
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

#[cfg(test)]
mod tests {
    use super::*;

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
