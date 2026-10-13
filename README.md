# EGRESSO

> *Di notte, esplorando a caso, trovi un dungeon che nessuno ti ha indicato.*

Un gioco di esplorazione in pixel art, con un engine scritto da zero in Rust e **zero file di asset**: font, mondo, pixel art, luce e suono sono generati dal codice.

- Concept: [`docs/CONCEPT.md`](docs/CONCEPT.md) (v0.4: si riparte da zero; le versioni precedenti sono in [`docs/archivio/`](docs/archivio/)).
- Diario di bordo (come è fatto, capitolo per capitolo): [`docs/diario/`](docs/diario/)

## Stato: la scoperta
La prima stanza curata: un esterno notturno, ogni partita diverso, da esplorare liberamente. Da qualche parte c'è Egresso. Nessuna freccia: il vento, le lucciole e le rovine tirano da quella parte. Cosa c'è dentro è ancora da inventare.

## Comandi
| Tasto | Azione |
|---|---|
| ← → / A D | cammina |
| Spazio / W / ↑ | salta (tieni premuto per saltare più in alto) |
| F3 | overlay di debug |

## Provarlo

### Nel browser (anche da telefono)
Lo stesso codice Rust gira nel browser, compilato in WebAssembly. Sul telefono compaiono i pulsanti touch: ◀ ▶ a sinistra e SALTA a destra. Meglio in orizzontale, e con l'audio acceso (il suono parte al primo tocco).

```sh
rustup target add wasm32-unknown-unknown   # una volta sola
./web/build.sh                             # crea web/dist/egresso.html
```
`web/dist/egresso.html` è un unico file autonomo: si apre con doppio clic, oppure si carica su un qualunque hosting statico (itch.io, GitHub Pages).

### Sul computer
1. Installa Rust: su Windows scarica `rustup-init.exe` da <https://rustup.rs>; su macOS e Linux usa `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`.
2. Scarica il codice e avvia:
   ```sh
   git clone https://github.com/NoximilienCoxen/Damn.git
   cd Damn
   git checkout ccr-5fdec350-chubqj
   cargo run --release      # gioca (finestra 960×540, scala intera)
   ```
3. `cargo test` lancia i test unitari di engine e gioco.

Su Linux serve una sessione grafica X11 o Wayland con `libxkbcommon` (presente su qualunque desktop).

### Modalità headless
Fa girare il gioco senza finestra con un input registrato e salva screenshot PNG. Serve per i test automatici e per controllare la grafica dove non c'è uno schermo.

```sh
cargo run --release -- --headless --frames 1800 --seme 3 \
  --script "60:left:+" \
  --every 150 --out shots --trace
```

Ogni evento dello script è `frame:tasto:azione`, dove l'azione è `+` (giù), `-` (su) o `!` (tocco). `--trace` stampa posizione e stato a ogni frame; `--seme N` sceglie la notte (con il seme 3, Egresso è a sinistra).
