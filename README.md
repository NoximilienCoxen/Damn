# EGRESSO

> *Il mondo è ciò che dici che sia. Per vederlo davvero devi smettere di nominarlo.*

Un platformer di esplorazione e puzzle, ironico e un po' assurdo. Il protagonista è bendato in un dungeon senza uscita, e il mondo che attraversa è fatto delle parole con cui la sua mente prova a descriverlo. Ha un engine scritto da zero in Rust e **zero file di asset**: font, livelli, pixel art e luce sono generati dal codice.

- Concept e game design: [`docs/CONCEPT.md`](docs/CONCEPT.md)
- Diario di bordo (come è fatto l'engine, capitolo per capitolo): [`docs/diario/`](docs/diario/)

## Stato: prototipo di due stanze
**La regola:** solo ciò che nomini regge. Il resto è vuoto, finché non lo lasci andare e scopri cosa c'era davvero.

- **L'Atrio** insegna la regola: un vuoto, una sola parola da dire.
- **L'Abisso** è il puzzle vero:
  - avvicinati alle cose: i sensi producono attributi (`VUOTO`, `VENTO`, `ECO`) e tre voci propongono un nome;
  - **affermare** un nome lo rende reale. Il `PONTE` regge, ma camminarci sopra lo consuma finché diventa `POSTE`;
  - **lasciar andare** scioglie le parole e per qualche secondo rivela il dungeon vero;
  - due finali per la stessa porta: nominarla ti riporta all'Atrio, lasciarla andare ti fa uscire.

## Comandi

| Tasto | Azione |
|---|---|
| ← → / A D | muoviti |
| Spazio / W / ↑ | salta (tieni premuto per saltare più in alto) |
| E | afferma la parola in primo piano |
| Q | passa alla voce successiva |
| R | lascia andare |
| F3 | overlay di debug (FPS, tempi) |

## Provarlo

### Nel browser (anche da telefono)
Lo stesso codice Rust gira nel browser, compilato in WebAssembly. Sul telefono compaiono i pulsanti touch: ◀ ▶ a sinistra, Q E R e SALTA a destra. Meglio in orizzontale.

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
cargo run --release -- --headless --frames 400 \
  --script "10:right:+ 75:right:- 145:r:! 150:right:+" \
  --shots 100,200 --every 0 --out shots --trace
```

Ogni evento dello script è `frame:tasto:azione`, dove l'azione è `+` (giù), `-` (su) o `!` (tocco). `--trace` stampa posizione e stato a ogni frame.
