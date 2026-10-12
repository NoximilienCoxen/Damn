//! Lezione 01: il framebuffer.
//!
//! Un framebuffer è un'immagine in memoria: una lunga fila di pixel,
//! riga dopo riga. Tutto ciò che il gioco disegna (muri, personaggio,
//! ricordi che si sciolgono) finisce qui dentro, e alla fine di ogni
//! frame questo array viene copiato sullo schermo.
//!
//! La spiegazione completa è in `docs/diario/01-framebuffer.md`.

/// Crea un colore a partire da rosso, verde e blu (0-255 ciascuno).
///
/// Il colore è un `u32` nel formato `0x00RRGGBB`: il byte più alto è
/// inutilizzato, poi rosso, verde e blu. È il formato che useremo anche
/// per la finestra (lezione 02).
///
/// `const fn` significa che può essere calcolata a tempo di compilazione,
/// così possiamo usarla per definire costanti come `NERO` qui sotto.
pub const fn rgb(r: u8, g: u8, b: u8) -> u32 {
    // `as u32` converte il byte in un numero a 32 bit, poi `<<` lo sposta
    // a sinistra (16 bit per il rosso, 8 per il verde) e `|` li unisce.
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

/// Qualche colore di prova. La vera palette arriverà con gli sprite.
pub const NERO: u32 = rgb(0, 0, 0);
pub const BIANCO: u32 = rgb(255, 255, 255);

/// Un'immagine fatta di `larghezza * altezza` pixel.
///
/// `pub struct` dichiara un nuovo tipo, come una "scheda" con dei campi.
/// I campi senza `pub` sono privati: dall'esterno si usano i metodi.
pub struct Framebuffer {
    larghezza: usize,
    altezza: usize,
    /// I pixel, riga per riga: prima tutta la riga 0, poi la riga 1, ecc.
    /// `Vec<u32>` è un array che vive nella memoria dinamica (heap) e può
    /// avere qualsiasi lunghezza.
    pixel: Vec<u32>,
}

/// `impl` contiene i metodi di `Framebuffer`.
impl Framebuffer {
    /// Crea un framebuffer tutto nero.
    ///
    /// (Esempio già scritto: guardalo bene, gli esercizi gli somigliano.)
    pub fn new(larghezza: usize, altezza: usize) -> Framebuffer {
        Framebuffer {
            larghezza,
            altezza,
            // `vec![valore; quantità]` crea un Vec ripetendo `valore`.
            pixel: vec![NERO; larghezza * altezza],
        }
    }

    pub fn larghezza(&self) -> usize {
        self.larghezza
    }

    pub fn altezza(&self) -> usize {
        self.altezza
    }

    /// Tutti i pixel in sola lettura. `&[u32]` è una *slice*: una "vista"
    /// su una parte (qui tutto) di un array, senza copiarlo.
    pub fn pixel(&self) -> &[u32] {
        &self.pixel
    }

    /// Tutti i pixel, modificabili. Serve agli effetti che lavorano
    /// sull'intera immagine (glow, rivelazione).
    pub fn pixel_mut(&mut self) -> &mut [u32] {
        &mut self.pixel
    }

    /// Esercizio 01.1 (soluzione): colora tutti i pixel.
    ///
    /// `slice::fill` scrive lo stesso valore in ogni elemento: è un ciclo
    /// `for` già scritto (e ottimizzato) dalla libreria standard.
    pub fn clear(&mut self, colore: u32) {
        self.pixel.fill(colore);
    }

    /// Indice nel Vec del pixel (x, y), oppure `None` se è fuori schermo.
    ///
    /// Il controllo `x < 0` va fatto *prima* della conversione: `-1 as usize`
    /// non è un errore, è un numero enorme.
    #[inline]
    fn indice(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 {
            return None;
        }
        let (x, y) = (x as usize, y as usize);
        if x >= self.larghezza || y >= self.altezza {
            return None;
        }
        Some(y * self.larghezza + x)
    }

    /// Esercizio 01.2 (soluzione): colora un pixel, ignorando quelli fuori schermo.
    #[inline]
    pub fn set_pixel(&mut self, x: i32, y: i32, colore: u32) {
        if let Some(i) = self.indice(x, y) {
            self.pixel[i] = colore;
        }
    }

    /// Il colore del pixel (x, y), o `None` fuori schermo.
    #[inline]
    pub fn get_pixel(&self, x: i32, y: i32) -> Option<u32> {
        self.indice(x, y).map(|i| self.pixel[i])
    }

    /// Mescola `colore` sopra il pixel esistente: `alpha` 0 = invisibile,
    /// 255 = opaco.
    #[inline]
    pub fn blend_pixel(&mut self, x: i32, y: i32, colore: u32, alpha: u8) {
        if let Some(i) = self.indice(x, y) {
            self.pixel[i] = mix(self.pixel[i], colore, alpha);
        }
    }

    /// Somma `colore` al pixel esistente (luce che si accumula), saturando a 255.
    #[inline]
    pub fn add_pixel(&mut self, x: i32, y: i32, colore: u32) {
        if let Some(i) = self.indice(x, y) {
            self.pixel[i] = add(self.pixel[i], colore);
        }
    }

    /// Esercizio 01.3 (soluzione, versione "bonus B1"): riempi un rettangolo.
    ///
    /// Invece di controllare i bordi per ogni pixel, tagliamo il rettangolo
    /// ai bordi una volta sola e poi riempiamo ogni riga con una slice.
    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, colore: u32) {
        let x0 = x.max(0);
        let y0 = y.max(0);
        let x1 = (x + w).min(self.larghezza as i32);
        let y1 = (y + h).min(self.altezza as i32);
        if x0 >= x1 || y0 >= y1 {
            return;
        }
        for riga in y0 as usize..y1 as usize {
            let inizio = riga * self.larghezza;
            self.pixel[inizio + x0 as usize..inizio + x1 as usize].fill(colore);
        }
    }
}

/// Canali (r, g, b) di un colore.
#[inline]
pub const fn canali(c: u32) -> (u8, u8, u8) {
    ((c >> 16) as u8, (c >> 8) as u8, c as u8)
}

/// Mescola due colori: `t` 0 = tutto `a`, 255 = tutto `b`.
#[inline]
pub fn mix(a: u32, b: u32, t: u8) -> u32 {
    let (ar, ag, ab) = canali(a);
    let (br, bg, bb) = canali(b);
    let t = t as u32;
    let m = |x: u8, y: u8| ((x as u32 * (255 - t) + y as u32 * t) / 255) as u8;
    rgb(m(ar, br), m(ag, bg), m(ab, bb))
}

/// Somma due colori canale per canale, senza superare 255.
#[inline]
pub fn add(a: u32, b: u32) -> u32 {
    let (ar, ag, ab) = canali(a);
    let (br, bg, bb) = canali(b);
    rgb(ar.saturating_add(br), ag.saturating_add(bg), ab.saturating_add(bb))
}

/// Moltiplica la luminosità di un colore per `k` (0.0 = nero).
#[inline]
pub fn scala(c: u32, k: f32) -> u32 {
    let (r, g, b) = canali(c);
    let f = |x: u8| (x as f32 * k).clamp(0.0, 255.0) as u8;
    rgb(f(r), f(g), f(b))
}

// I test vivono nello stesso file, in un modulo che viene compilato solo
// con `cargo test`.
#[cfg(test)]
mod tests {
    use super::*;

    /// Legge il pixel (x, y) direttamente dal Vec, per controllare.
    fn leggi(fb: &Framebuffer, x: usize, y: usize) -> u32 {
        fb.pixel()[y * fb.larghezza() + x]
    }

    const ROSSO: u32 = rgb(255, 0, 0);
    const VERDE: u32 = rgb(0, 255, 0);

    #[test]
    fn rgb_compone_i_canali() {
        assert_eq!(rgb(0x12, 0x34, 0x56), 0x00123456);
    }

    #[test]
    fn new_crea_un_framebuffer_nero() {
        let fb = Framebuffer::new(4, 3);
        assert_eq!(fb.pixel().len(), 12);
        assert!(fb.pixel().iter().all(|&p| p == NERO));
    }

    #[test]
    fn es01_1_clear_colora_tutto() {
        let mut fb = Framebuffer::new(5, 4);
        fb.clear(ROSSO);
        assert!(fb.pixel().iter().all(|&p| p == ROSSO));
    }

    #[test]
    fn es01_2_set_pixel_colora_un_solo_pixel() {
        let mut fb = Framebuffer::new(4, 3);
        fb.set_pixel(2, 1, ROSSO);
        assert_eq!(leggi(&fb, 2, 1), ROSSO);
        // Tutti gli altri devono essere rimasti neri.
        let colorati = fb.pixel().iter().filter(|&&p| p == ROSSO).count();
        assert_eq!(colorati, 1);
    }

    #[test]
    fn es01_2_set_pixel_usa_la_riga_giusta() {
        // Se scambi x e y nella formula, questo test se ne accorge.
        let mut fb = Framebuffer::new(4, 3);
        fb.set_pixel(3, 0, ROSSO);
        fb.set_pixel(0, 2, VERDE);
        assert_eq!(fb.pixel()[3], ROSSO);
        assert_eq!(fb.pixel()[8], VERDE);
    }

    #[test]
    fn es01_2_set_pixel_fuori_schermo_non_fa_niente() {
        let mut fb = Framebuffer::new(4, 3);
        fb.set_pixel(-1, 0, ROSSO);
        fb.set_pixel(0, -1, ROSSO);
        fb.set_pixel(4, 0, ROSSO); // x == larghezza è già fuori!
        fb.set_pixel(0, 3, ROSSO);
        fb.set_pixel(100, 100, ROSSO);
        assert!(fb.pixel().iter().all(|&p| p == NERO));
    }

    #[test]
    fn es01_3_fill_rect_riempie_il_rettangolo() {
        let mut fb = Framebuffer::new(6, 5);
        fb.fill_rect(1, 2, 3, 2, ROSSO);
        for y in 0..5 {
            for x in 0..6 {
                let dentro = (1..4).contains(&x) && (2..4).contains(&y);
                let atteso = if dentro { ROSSO } else { NERO };
                assert_eq!(leggi(&fb, x, y), atteso, "pixel ({x}, {y})");
            }
        }
    }

    #[test]
    fn es01_3_fill_rect_taglia_i_bordi() {
        let mut fb = Framebuffer::new(4, 4);
        // Sborda a sinistra, in alto, a destra e in basso.
        fb.fill_rect(-2, -2, 10, 10, VERDE);
        assert!(fb.pixel().iter().all(|&p| p == VERDE));
    }

    #[test]
    fn mix_e_add() {
        assert_eq!(mix(NERO, BIANCO, 0), NERO);
        assert_eq!(mix(NERO, BIANCO, 255), BIANCO);
        assert_eq!(add(rgb(200, 10, 0), rgb(100, 10, 5)), rgb(255, 20, 5));
        assert_eq!(scala(rgb(100, 200, 50), 0.5), rgb(50, 100, 25));
    }

    #[test]
    fn es01_3_fill_rect_vuoto_non_disegna() {
        let mut fb = Framebuffer::new(4, 4);
        fb.fill_rect(1, 1, 0, 3, ROSSO);
        fb.fill_rect(1, 1, 3, -1, ROSSO);
        fb.fill_rect(10, 10, 2, 2, ROSSO);
        assert!(fb.pixel().iter().all(|&p| p == NERO));
    }
}
