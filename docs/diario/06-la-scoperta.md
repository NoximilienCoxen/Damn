# 06. La scoperta

Il prototipo delle tre stanze ("la Sala che cresce") è stato bocciato: regole senza contesto, nessuna atmosfera, niente di quanto avevamo detto. Si è ripartiti da zero, tenendo un solo punto fermo: **trovare Egresso esplorando a caso**. Questo capitolo racconta come è fatta la prima stanza curata, quella della scoperta.

## I file

| File | Cosa fa |
|---|---|
| `src/game/terreno.rs` | il mondo come funzioni della x: suolo, piazza e collina di Egresso, vento, rovine, alberi |
| `src/game/giocatore.rs` | camminare e saltare su un suolo continuo (niente griglia) |
| `src/game/natura.rs` | le lucciole |
| `src/game/render.rs` | cielo, parallasse, facciata, suolo, erba, glow, il buio |
| `src/game/mod.rs` | le scene (fuori, soglia, dentro), la telecamera, il suono |

## Un mondo fatto di funzioni
Niente mappe: il suolo è `Terreno::suolo(x)`, la somma di tre rumori 1D a scale diverse. Il rumore è "periodico": i punti del reticolo si ripetono ogni `GIRO` pixel, quindi il mondo gira su se stesso. Le x del protagonista e della telecamera non hanno limiti; ogni distanza nel mondo passa da `delta(a, b)`, la distanza più breve tenendo conto del giro.

Vicino all'arco il rumore viene sostituito: una piazza piana, poi da un lato il fronte di roccia e la cima della collina, che scende dolcemente (`liscio`) fino a tornare al terreno naturale. Dall'altro lato la piazza si raccorda al terreno.

Rovine e alberi stanno in **celle**: ogni cella ha un hash, e l'hash decide se lì c'è qualcosa e cosa. La probabilità di una rovina cresce con il quadrato della vicinanza a Egresso; gli alberi invece vicino a Egresso non crescono.

## Il vento che indica la strada
`Terreno::vento(x, t)` è una brezza (due seni lenti) che, avvicinandosi all'arco, sfuma in un tiro verso l'arco con un'ampiezza che respira. Lo mostrano:
- l'**erba**: un filo ogni 2 pixel, che si piega con il quadrato dell'altezza (la punta si piega più della base) e si scosta quando ci passi;
- le **foglie**: nascono dalla parte da cui viene il vento, fuori schermo, e lo seguono;
- la **sciarpa** del protagonista, che sventola dalla parte in cui soffia;
- le **lucciole**, spinte anche loro.

## La telecamera e la scoperta
La telecamera segue il protagonista; entro 190 pixel dall'arco si sposta piano verso l'arco, così la facciata entra in quadro prima di arrivarci. Le lettere del nome si accendono con la vicinanza **massima raggiunta** (`Egresso::lettere`): una volta accese restano accese.

Davanti all'arco per un quarto di secondo (cioè passandoci davanti camminando) si passa alla **soglia**: i comandi si spengono, il protagonista entra da solo e svanisce, il buio si allarga dall'arco a coprire tutto. Poi **dentro**: nero, e la voce.

## Il buio che non prende luce
Il glow somma la luce sfocata a tutta l'immagine: anche l'arco si schiarirebbe. Per questo il buio dell'arco viene disegnato **dopo** il glow, e il protagonista, se ci sta davanti, viene ridisegnato sopra.

## Il suono
Il motore non ha audio. Il gioco espone quattro numeri (`Game::suono`): vento, richiamo, buio, voce. La pagina web li legge a ogni frame e muove un piccolo sintetizzatore WebAudio: rumore bianco filtrato per il vento, tre oscillatori bassi per il richiamo, una campana che parte quando la voce comincia a parlare. La campana va direttamente all'uscita, perché il volume generale si abbassa quando si entra nel buio.

Anche il **seme** viene da fuori (`Game::semina`): nel browser da `Math.random`, sul computer dall'orologio, in headless da `--seme N`.

## Test
- `terreno`: il mondo si ripete; Egresso è lontano dalla partenza e cambia con il seme; la piazza è piana e la collina sta sopra; il vento tira verso l'arco; vicino a Egresso ci sono più rovine.
- `giocatore`: cammina sulle colline senza staccarsi; il fronte della collina è un muro.
- `game`: si parte lontano e senza lettere accese; camminando verso l'arco lo si trova e si entra (sei notti diverse); andando dall'altra parte si fa il giro e si arriva lo stesso; da dentro si ricomincia altrove; il suono cresce avvicinandosi.
