# EGRESSO

> *Un metroidvania che ti vuole tenere dentro. Ogni sforzo allarga la prigione. L'uscita c'è dalla prima stanza.*

Un platformer di esplorazione, ironico e un po' assurdo, ispirato al wu wei: si avanza capendo, non accumulando poteri. Ha un engine scritto da zero in Rust e **zero file di asset**: font, stanze, pixel art e luce sono generati dal codice.

- Concept e game design: [`docs/CONCEPT.md`](docs/CONCEPT.md) (v0.3, "la Sala che cresce"; la versione precedente è in [`docs/archivio/`](docs/archivio/)).
- Diario di bordo (come è fatto, capitolo per capitolo): [`docs/diario/`](docs/diario/)

## Stato: prototipo di tre stanze
È un primo pezzo verticale. Il gioco completo avrà **molte più stanze** (vedi la mappa nel concept): qui si provano le regole.

**Le regole, tutte fisiche e prevedibili:**
- **Più forzi, più la Sala si allunga.** Spingere contro muri, venti e correnti, o saltare a ripetizione, aggiunge pezzi di stanza. Attraversarla senza forzare la riporta alla sua misura.
- **Dentro una corrente, chi si muove resta dov'è; chi si lascia andare viene portato.** In acqua, finché tieni premuto nuoti a galla; se lasci tutto affondi piano e la corrente di fondo ti porta. Nell'aria che sale vale lo stesso.

**Le stanze:**
- **La Riva**, dove ti svegli. Un canale corre verso un arco luminoso, separato dalla riva da una diga.
- **L'Atrio**, un corridoio con il vento a raffiche e qualche riparo.
- **La Galleria delle Correnti**: una vasca divisa da un labbro di roccia, una colonna d'aria, un corridoio in alto che riporta all'ingresso.

Sui muri ci sono le scritte del Cartografo (carbone: si legge solo da vicino, con la lanterna), della Nuotatrice (gesso) e le targhe ufficiali, e bugiarde, del Curatore. La Sala, la voce del dungeon, ti incoraggia a forzare.

## Comandi
| Tasto | Azione |
|---|---|
| ← → / A D | muoviti (in acqua: nuota) |
| Spazio / W / ↑ | salta (tieni premuto per saltare più in alto) |
| nessuno | lasciarsi andare |
| F3 | overlay di debug (FPS, tempi, quanto è lunga ogni stanza) |

## Provarlo

### Nel browser (anche da telefono)
Lo stesso codice Rust gira nel browser, compilato in WebAssembly. Sul telefono compaiono i pulsanti touch: ◀ ▶ a sinistra e SALTA a destra. Meglio in orizzontale.

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
cargo run --release -- --headless --frames 600 --stanza 2 \
  --script "300:right:+ 360:right:-" \
  --shots 340,500 --every 0 --out shots --trace
```

Ogni evento dello script è `frame:tasto:azione`, dove l'azione è `+` (giù), `-` (su) o `!` (tocco). `--trace` stampa posizione e stato a ogni frame; `--stanza N` fa partire dalla stanza N (0 Riva, 1 Atrio, 2 Galleria).
