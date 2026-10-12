//! Il contratto fra engine e gioco (il trait `Game`) e la modalità headless,
//! che fa girare il gioco senza finestra con un input registrato e salva
//! screenshot. I "gusci" veri stanno in `window.rs` (desktop) e `web.rs`
//! (browser): l'engine non sa niente di parole o di occhi chiusi.

use super::framebuffer::Framebuffer;
use super::input::{Input, Key};

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
    /// Quali comandi mostrare all'interfaccia (es. pulsanti touch): una
    /// maschera di bit decisa dal gioco. Di default tutti.
    fn comandi(&self) -> u32 {
        u32::MAX
    }
    /// Una riga di stato per `--trace` in headless (es. posizione del giocatore).
    fn traccia(&self) -> String {
        String::new()
    }
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
pub fn run_headless<G: Game>(
    game: &mut G,
    frames: u32,
    script: &[EventoScript],
    scatti: &[u32],
    ogni: u32,
    cartella: &str,
    traccia: bool,
) -> std::io::Result<()> {
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
}
