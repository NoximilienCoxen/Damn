//! Un encoder PNG minimo, senza dipendenze.
//!
//! Un PNG è: una firma di 8 byte, poi dei "chunk" (lunghezza, tipo, dati,
//! CRC32). Bastano tre chunk: IHDR (dimensioni e formato), IDAT (i pixel
//! compressi con zlib) e IEND. Per non scrivere un compressore usiamo i
//! blocchi "stored" di deflate: dati non compressi ma validi. I file sono
//! più grandi, ma per screenshot di 320×180 va benissimo.

/// Codifica i pixel `0x00RRGGBB` in un file PNG RGB a 8 bit.
pub fn encode(larghezza: usize, altezza: usize, pixel: &[u32]) -> Vec<u8> {
    assert_eq!(pixel.len(), larghezza * altezza);

    // I dati grezzi: ogni riga inizia con un byte di "filtro" (0 = nessuno).
    let mut grezzi = Vec::with_capacity(altezza * (1 + larghezza * 3));
    for riga in pixel.chunks_exact(larghezza) {
        grezzi.push(0);
        for &p in riga {
            grezzi.extend_from_slice(&[(p >> 16) as u8, (p >> 8) as u8, p as u8]);
        }
    }

    let mut out = Vec::with_capacity(grezzi.len() + 1024);
    out.extend_from_slice(&[0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n']);

    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&(larghezza as u32).to_be_bytes());
    ihdr.extend_from_slice(&(altezza as u32).to_be_bytes());
    ihdr.extend_from_slice(&[8, 2, 0, 0, 0]); // 8 bit, RGB, deflate, filtro 0, no interlacciamento
    chunk(&mut out, b"IHDR", &ihdr);
    chunk(&mut out, b"IDAT", &zlib_stored(&grezzi));
    chunk(&mut out, b"IEND", &[]);
    out
}

fn chunk(out: &mut Vec<u8>, tipo: &[u8; 4], dati: &[u8]) {
    out.extend_from_slice(&(dati.len() as u32).to_be_bytes());
    let inizio = out.len();
    out.extend_from_slice(tipo);
    out.extend_from_slice(dati);
    let crc = crc32(&out[inizio..]);
    out.extend_from_slice(&crc.to_be_bytes());
}

/// Flusso zlib fatto solo di blocchi deflate non compressi (max 65535 byte l'uno).
fn zlib_stored(dati: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(dati.len() + dati.len() / 65535 * 5 + 16);
    out.extend_from_slice(&[0x78, 0x01]);
    let mut blocchi = dati.chunks(65535).peekable();
    if blocchi.peek().is_none() {
        out.extend_from_slice(&[1, 0, 0, 0xff, 0xff]);
    }
    while let Some(b) = blocchi.next() {
        let ultimo = blocchi.peek().is_none();
        let len = b.len() as u16;
        out.push(ultimo as u8);
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&(!len).to_le_bytes());
        out.extend_from_slice(b);
    }
    out.extend_from_slice(&adler32(dati).to_be_bytes());
    out
}

fn crc32(dati: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for &b in dati {
        crc ^= b as u32;
        for _ in 0..8 {
            let maschera = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xedb8_8320 & maschera);
        }
    }
    !crc
}

fn adler32(dati: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for blocco in dati.chunks(5552) {
        for &x in blocco {
            a += x as u32;
            b += a;
        }
        a %= 65521;
        b %= 65521;
    }
    (b << 16) | a
}

/// Comodità: salva un framebuffer su disco.
pub fn save(fb: &crate::engine::framebuffer::Framebuffer, percorso: &str) -> std::io::Result<()> {
    std::fs::write(percorso, encode(fb.larghezza(), fb.altezza(), fb.pixel()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksum_noti() {
        assert_eq!(crc32(b"IEND"), 0xae42_6082);
        assert_eq!(adler32(b"Wikipedia"), 0x11e6_0398);
    }

    #[test]
    fn struttura_del_file() {
        let png = encode(2, 1, &[0xff0000, 0x00ff00]);
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
        assert_eq!(&png[12..16], b"IHDR");
        assert_eq!(&png[png.len() - 8..png.len() - 4], b"IEND");
    }
}
