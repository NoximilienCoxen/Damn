//! Lezione 01: il framebuffer.
//!
//! Un framebuffer è un'immagine in memoria: una lunga fila di pixel,
//! riga dopo riga. Tutto ciò che il gioco disegna (muri, personaggio,
//! ricordi che si sciolgono) finisce qui dentro, e alla fine di ogni
//! frame questo array viene copiato sullo schermo.
//!
//! Leggi `docs/diario/01-framebuffer.md` prima di risolvere gli esercizi.

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

    /// ESERCIZIO 01.1: colora *tutti* i pixel con `colore`.
    ///
    /// `&mut self` significa che il metodo può modificare il framebuffer.
    /// Suggerimento: un ciclo `for` sugli elementi di `self.pixel`, oppure
    /// cerca nella documentazione di `slice` un metodo che fa già tutto.
    pub fn clear(&mut self, colore: u32) {
        let _ = colore; // togli questa riga quando usi `colore`
        todo!("Esercizio 01.1: clear")
    }

    /// ESERCIZIO 01.2: colora il pixel alla colonna `x` e riga `y`.
    ///
    /// Le coordinate sono `i32` (con segno) di proposito: più avanti
    /// disegneremo sprite che escono in parte dallo schermo, con x o y
    /// negativi. Se il pixel è fuori dall'immagine, la funzione non deve
    /// fare niente (e soprattutto non deve andare in panic!).
    ///
    /// La formula chiave è nel diario: `indice = y * larghezza + x`.
    /// Attenzione ai tipi: `self.pixel[...]` vuole un `usize`.
    pub fn set_pixel(&mut self, x: i32, y: i32, colore: u32) {
        let _ = (x, y, colore);
        todo!("Esercizio 01.2: set_pixel")
    }

    /// ESERCIZIO 01.3: riempi un rettangolo.
    ///
    /// L'angolo in alto a sinistra è `(x, y)`, poi `w` pixel di larghezza
    /// e `h` di altezza. Le parti fuori dallo schermo vanno ignorate, e se
    /// `w` o `h` sono `<= 0` non si disegna niente.
    ///
    /// Prima versione: due `for` annidati che chiamano `set_pixel`.
    /// Funziona ed è giusta! Il bonus nel diario spiega come renderla veloce.
    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, colore: u32) {
        let _ = (x, y, w, h, colore);
        todo!("Esercizio 01.3: fill_rect")
    }
}

// I test vivono nello stesso file, in un modulo che viene compilato solo
// con `cargo test`. Sono il tuo "verificatore": quando passano tutti,
// l'esercizio è risolto. Non modificarli (puoi leggerli, anzi: fallo).
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
    fn es01_3_fill_rect_vuoto_non_disegna() {
        let mut fb = Framebuffer::new(4, 4);
        fb.fill_rect(1, 1, 0, 3, ROSSO);
        fb.fill_rect(1, 1, 3, -1, ROSSO);
        fb.fill_rect(10, 10, 2, 2, ROSSO);
        assert!(fb.pixel().iter().all(|&p| p == NERO));
    }
}
