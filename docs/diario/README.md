# Diario di bordo

Questo diario era nato come corso a lezioni, con esercizi da risolvere. Per ora il codice lo scrivo io, così il progetto avanza; il diario resta come **materiale di studio**: ogni capitolo racconta cosa è stato costruito, perché, e quali concetti ci sono dietro. Le lezioni 00 e 01 restano nella forma originale, con gli esercizi già risolti nel codice, e si possono ancora rifare come esercizio.

## Capitoli

| # | Capitolo | Concetti |
|---|---|---|
| 00 | [Setup e primo programma](00-setup.md) | Rust, cargo, struttura del progetto |
| 01 | [Il framebuffer](01-framebuffer.md) | pixel, colori, `struct`/`impl`, indici (esercizi risolti in `src/engine/framebuffer.rs`) |
| 02 | [Il motore minimo](02-motore.md) | finestra, game loop a passo fisso, input, font scritto a mano, glow, PNG, headless |
| 03 | [Il prototipo](03-prototipo.md) | mondo a due strati, macchina a stati delle percezioni, decadimento, collisioni, rivelazione |
| 04 | [Revisione del concept](04-revisione.md) | perché il "mondo scritto" è stato messo da parte, le fonti, la nuova direzione |
| 05 | [La Sala che cresce](05-sala-che-cresce.md) | stanze che si allungano, correnti, sforzo, telecamera, scritte (prototipo poi scartato) |
| 06 | [La scoperta](06-la-scoperta.md) | mondo fatto di funzioni che gira su se stesso, vento, parallasse, il buio dopo il glow, suono sintetizzato |

## Prossimi capitoli (quando ci arriviamo)
- Luce vera: ombre proiettate dalla roccia.
- Una mappa consultabile, che registra le stanze ma non i segreti.
- Audio sintetizzato.
- Dentro Egresso: la prima stanza.
