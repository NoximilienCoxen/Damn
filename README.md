# EGRESSO

> *Il mondo è ciò che dici che sia. Per vederlo davvero devi smettere di nominarlo.*

Un platformer di esplorazione e puzzle, ironico e un po' assurdo. Il protagonista è bendato in un dungeon senza uscita, e il mondo che attraversa è fatto delle parole con cui la sua mente prova a descriverlo. Ha un engine scritto da zero in Rust e **zero file di asset**: sprite, livelli e suoni sono generati dal codice.

- Concept e game design: [`docs/CONCEPT.md`](docs/CONCEPT.md)
- Diario di sviluppo, cioè le lezioni per costruire l'engine passo passo: [`docs/diario/`](docs/diario/)

## Avvio rapido

```sh
cargo run              # il programma principale
cargo test             # i test (e gli esercizi del diario)
cargo run --bin dump   # disegna una scena di prova in egresso.ppm
```

Serve Rust: vedi [`docs/diario/00-setup.md`](docs/diario/00-setup.md).
