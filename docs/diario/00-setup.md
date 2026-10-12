# 00: Setup e primo programma

## Cos'è un game engine?
Un game engine è il codice che **non** è il gioco, ma senza il quale il gioco non può esistere. In pratica:

1. **apre una finestra** e ci copia dentro un'immagine, molte volte al secondo;
2. **legge l'input** (tastiera, controller);
3. **fa avanzare il tempo** del mondo a passi regolari (il *game loop*);
4. **disegna** (sprite, tile, luci);
5. **produce suoni**.

Unity e Godot fanno tutto questo e molto altro. Noi lo scriviamo da zero, in piccolo: il nostro engine farà solo ciò che serve a EGRESSO. È esattamente la strada di Billy Basso con Animal Well, che ha scritto il suo engine da solo in C++.

## Installare Rust
Su Linux e macOS:
```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
Su Windows: scarica `rustup-init.exe` da <https://rustup.rs> (ti chiederà di installare anche i "Build Tools" di Visual Studio: accetta).

Poi verifica:
```sh
rustc --version
cargo --version
```

`rustup` installa e aggiorna Rust, `rustc` è il compilatore, `cargo` è il "tuttofare": compila, lancia, testa e scarica le librerie.

**Editor consigliato:** VS Code con l'estensione **rust-analyzer**. Ti mostra errori e tipi mentre scrivi, ed è il miglior insegnante di Rust che esista.

## Prendere il codice
```sh
git clone https://github.com/NoximilienCoxen/Damn.git
cd Damn
git checkout ccr-5fdec350-chubqj
cargo run
```
Dovresti vedere:
```
EGRESSO: tentativo di uscita n. 1
L'uscita non c'è. Prova a chiudere gli occhi.
```

## Com'è fatto il progetto
```
Cargo.toml                    # nome del progetto e dipendenze
src/
  main.rs                     # il programma principale (cargo run)
  lib.rs                      # la "libreria": l'engine e (poi) il gioco
  engine/
    mod.rs                    # elenca i moduli dell'engine
    framebuffer.rs            # lezione 01
  bin/
    dump.rs                   # programma extra: salva un'immagine (cargo run --bin dump)
docs/
  CONCEPT.md                  # il game design
  diario/                     # queste lezioni
```

Perché separare `lib.rs` e `main.rs`? Così l'engine si può usare da più programmi: il gioco vero, `dump`, e domani magari un editor di livelli. E i test lo provano senza dover aprire una finestra.

## Il Rust di questa lezione
Apri `src/main.rs`; ci trovi già:

- `fn main() { ... }`, la funzione da cui parte tutto;
- `let x = 5;`, una variabile **immutabile**: in Rust è il default, e aiuta a capire cosa cambia e cosa no;
- `let mut x = 5;`, una variabile **mutabile**;
- `println!("ciao {}", x);`, che stampa. Il `!` indica una *macro*: per ora pensala come una funzione speciale.

## Esercizio 00
Modifica `src/main.rs`:
1. Prova a togliere `mut` da `tentativi_di_uscita` e lancia `cargo run`. **Leggi l'errore del compilatore per intero.** Gli errori di Rust sono lunghi perché ti spiegano cosa fare: abituarsi a leggerli è metà del lavoro.
2. Rimetti `mut`, poi usa un ciclo `for i in 1..=3 { ... }` per stampare tre tentativi di uscita, ognuno con un messaggio diverso (`if i == 3 { ... } else { ... }`).
3. Facoltativo: crea una funzione `fn messaggio(tentativo: u32) -> &'static str` che restituisce il testo, e chiamala dal ciclo. (`&'static str` è "un testo scritto nel programma": ci torneremo.)

Niente test per questo esercizio: se `cargo run` stampa quello che volevi, è risolto. Poi passa alla [lezione 01](01-framebuffer.md).
