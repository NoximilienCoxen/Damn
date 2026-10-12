# 01: Il framebuffer

## Il concetto: un'immagine è una fila di numeri
Lo schermo di EGRESSO è largo **320 pixel** e alto **180**. Ogni pixel è un colore, e un colore è un numero. Il **framebuffer** è l'array che contiene tutti quei numeri: 320 × 180 = 57.600 pixel.

Ogni frame il gioco: 1) cancella il framebuffer, 2) ci disegna sopra tutto (stanza, personaggio, ricordi), 3) lo manda alla finestra. Tutto il resto dell'engine, alla fine, scrive qui dentro.

### Da 2D a 1D
L'immagine è bidimensionale, ma la memoria è una fila sola. Allora mettiamo le righe una dopo l'altra:

```
immagine 4×3            nel Vec (12 elementi)

   x→ 0  1  2  3
y 0   A  B  C  D        [A B C D | E F G H | I J K L]
↓ 1   E  F  G  H          0 1 2 3   4 5 6 7   8 9 10 11
  2   I  J  K  L
```

Il pixel `G` è a `x = 2, y = 1`. Nel Vec è all'indice 6. Infatti:

> **indice = y × larghezza + x** → 1 × 4 + 2 = 6

È *la* formula di questa lezione: ogni volta che si salta una riga si avanza di `larghezza` posizioni.

### Il colore in un `u32`
Un `u32` occupa 32 bit = 4 byte. Usiamo il formato `0x00RRGGBB`:

```
0x00  FF  80  20
  │   │   │   └ blu   (0-255)
  │   │   └──── verde (0-255)
  │   └──────── rosso (0-255)
  └──────────── inutilizzato
```

La funzione `rgb(r, g, b)` in `framebuffer.rs` (già scritta) compone questi pezzi con gli operatori sui bit `<<` (sposta a sinistra) e `|` (unisci). `rgb(255, 128, 32)` vale `0x00FF8020`. In `src/bin/dump.rs` trovi l'operazione inversa, `>>`, che estrae i canali.

## Il Rust di questa lezione

### `struct` e `impl`
```rust
pub struct Framebuffer {
    larghezza: usize,
    altezza: usize,
    pixel: Vec<u32>,
}
```
Una `struct` raggruppa dei dati sotto un nome. Dentro `impl Framebuffer { ... }` scriviamo i **metodi**, le funzioni che lavorano su un `Framebuffer`.

### `self`, `&self`, `&mut self`
Il primo parametro di un metodo dice cosa il metodo può fare col framebuffer:
- `&self`: lo può **leggere** (es. `larghezza()`);
- `&mut self`: lo può **modificare** (es. `clear`, `set_pixel`).

Il `&` vuol dire "in prestito": il metodo lo usa senza diventarne proprietario. È il famoso *borrowing*: lo approfondiamo nella lezione 02. Per ora basta la regola "per modificare serve `&mut`".

### `Vec<u32>` e gli indici
```rust
let mut v = vec![0u32; 5];   // [0, 0, 0, 0, 0]
v[2] = 7;                    // [0, 0, 7, 0, 0]
let n = v.len();             // 5
for p in v.iter_mut() {      // scorre gli elementi *modificandoli*
    *p = 1;                  // `*p` = "il valore a cui punta p"
}
```
Gli indici di un Vec sono **`usize`**, un intero senza segno grande quanto un puntatore. Se l'indice è fuori dal Vec, il programma va in **panic**: si ferma con un errore. Meglio un errore subito che un bug nascosto, ma noi non vogliamo che succeda quando uno sprite esce dallo schermo.

### Numeri con e senza segno
| tipo | valori | usato per |
|---|---|---|
| `i32` | da −2.147.483.648 a 2.147.483.647 | coordinate (possono essere negative) |
| `usize` | da 0 in su | indici e dimensioni |

Rust **non converte da solo** tra tipi numerici: si scrive `x as usize`. Attenzione però: `(-1i32) as usize` non dà errore, dà un numero enorme. Quindi **prima controlli che `x >= 0`, poi converti**.

### `todo!()`
È una macro che, se eseguita, fa panic con un messaggio. Serve a dire "qui manca il codice". Il programma compila lo stesso, così puoi lavorare un esercizio alla volta.

## Esercizi
Apri `src/engine/framebuffer.rs`. Per vedere solo i test di questa lezione:
```sh
cargo test es01
```
All'inizio falliscono tutti con `not yet implemented`: è normale.

### 01.1: `clear`
Colora tutti i pixel con `colore`. Si fa in 1-3 righe.

### 01.2: `set_pixel`
1. Se `x` o `y` sono negativi, esci con `return;`.
2. Converti in `usize` e controlla che `x < larghezza` e `y < altezza`. Attenzione: `x == larghezza` è **già fuori**, perché gli indici partono da 0.
3. Calcola l'indice con la formula e assegna il colore.

### 01.3: `fill_rect`
Prima versione: due `for` annidati (righe da `y` a `y + h`, colonne da `x` a `x + w`) che chiamano `self.set_pixel(...)`. Il controllo dei bordi te lo fa già `set_pixel`.

In Rust `a..b` è l'intervallo da `a` incluso a `b` escluso, e se `b <= a` è vuoto. Quindi il caso `w <= 0` si gestisce da solo.

### Quando i test sono verdi
```sh
cargo run --bin dump
```
Crea `egresso.ppm`: aprilo (GIMP, IrfanView, l'Anteprima del Mac, o un convertitore online da PPM a PNG). Devi vedere una stanza buia con una porta, un omino, un pavimento e qualche stella. È il primo frame di EGRESSO, disegnato dal **tuo** codice.

Poi fai commit e push:
```sh
git add -A
git commit -m "Lezione 01: framebuffer"
git push
```

## Bonus (facoltativi)
- **B1:** `fill_rect` con `set_pixel` ricontrolla i bordi per *ogni* pixel. Riscrivila calcolando **una volta** il rettangolo già tagliato ai bordi (suggerimento: `x.max(0)` e `(x + w).min(larghezza as i32)`), poi riempi ogni riga con una slice: `self.pixel[inizio..fine].fill(colore)`.
- **B2:** aggiungi un metodo `get_pixel(&self, x: i32, y: i32) -> Option<u32>` che restituisce `None` fuori dallo schermo. `Option` è il modo di Rust per dire "forse c'è un valore": niente `null`.
- **B3:** cambia la scena in `dump.rs`: disegna la *tua* stanza della Hall.

## Domande per riflettere
1. Perché `set_pixel` prende `i32` e non direttamente `usize`?
2. Se lo schermo fosse 1920×1080, quanti byte occuperebbe il framebuffer? E perché per EGRESSO 320×180 è un'ottima scelta?
3. Nella formula `y * larghezza + x`, cosa succederebbe se usassi `x * altezza + y`? (Il test `es01_2_set_pixel_usa_la_riga_giusta` esiste per questo.)

---
## Revisione
*(La scrivo io dopo il tuo push.)*
