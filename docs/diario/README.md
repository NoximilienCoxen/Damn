# Diario di sviluppo

Qui costruiamo l'engine di EGRESSO un pezzo alla volta. Ogni lezione spiega un concetto di game engine e il Rust che serve per realizzarlo, poi ti lascia degli esercizi nel codice.

## Come funziona una lezione
1. Leggi il capitolo.
2. Apri i file indicati e cerca i `todo!("Esercizio ...")`.
3. Scrivi la tua soluzione e lancia `cargo test`: i test degli esercizi passano solo quando sono risolti.
4. Fai commit e push: rivedo il codice e aggiungo una sezione **Revisione** in fondo al capitolo.
5. Se ti blocchi, chiedi pure un suggerimento: ne arrivano di sempre più espliciti prima della soluzione.

## Lezioni

| # | Lezione | Rust | Engine | Stato |
|---|---|---|---|---|
| 00 | [Setup e primo programma](00-setup.md) | cargo, `let`/`mut` | cos'è un engine | da fare |
| 01 | [Il framebuffer](01-framebuffer.md) | `struct`, `Vec`, indici | pixel e colori | da fare |
| 02 | La finestra | ownership e borrow | winit/softbuffer, scaling 320×180 | |
| 03 | Il game loop | `Instant`, enum | fixed timestep | |
| 04 | Input | `match`, `HashSet` | premuto / appena premuto | |
| 05 | Un font scritto a mano | stringhe, iteratori, `const` | glifi 5×7, disegnare testo | |
| 06 | Il mondo come parole | moduli, array 2D | livelli in ASCII → parole posizionate | |
| 07 | Platformer | `f32`, trait per `Vec2` | collisioni AABB asse per asse | |
| 08 | Memoria e performance | `with_capacity`, indici al posto dei puntatori | pool, struct-of-arrays, zero allocazioni nel loop | |
| 09 | Lettere fisiche | `impl` avanzato, slice mutabili | corpi rigidi: impulsi, rotazione, impilamento | |
| 10 | Luce | colori in `f32`, cicli veloci | luci emissive per lettera, ombre a raggi | |
| 11 | I sensi e le voci | enum con dati, trait | attributi → nomi, le tre voci, affermare | |
| 12 | Decadimento | RNG scritto a mano, dizionari | lettere che cadono, mutazioni fra parole | |
| 13 | Lasciar andare | closure, lifetime | dissolvenza a particelle, la realtà sotto | |
| 14 | Audio sintetizzato | thread, `Arc` | oscillatori, inviluppi | |
| 15 | La stanza senza nome | macchina a stati | il finale senza input | |
