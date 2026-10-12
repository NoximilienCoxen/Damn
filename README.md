# EGRESSO

> *Il gioco disegna ciò che il personaggio ricorda, non ciò che esiste.
> E a occhi chiusi i ricordi diventano realtà.*

Un'avventura di esplorazione e puzzle, ironica e un po' assurda, ambientata in un dungeon che non ha uscita, almeno finché tieni gli occhi aperti. Ha un engine scritto da zero in Rust e **zero file di asset**: sprite, livelli e suoni sono generati dal codice.

- Concept e game design: [`docs/CONCEPT.md`](docs/CONCEPT.md)
- Diario di sviluppo, cioè le lezioni per costruire l'engine passo passo: [`docs/diario/`](docs/diario/)

## Avvio rapido

```sh
cargo run              # il programma principale
cargo test             # i test (e gli esercizi del diario)
cargo run --bin dump   # disegna una scena di prova in egresso.ppm
```

Serve Rust: vedi [`docs/diario/00-setup.md`](docs/diario/00-setup.md).
