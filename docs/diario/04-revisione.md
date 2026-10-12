# 04. Revisione del concept

Questo capitolo non parla di codice: racconta perché il prototipo del "mondo scritto" è stato messo da parte e da dove riparte il progetto.

## Cosa non funzionava
Provando il prototipo dal telefono, il giudizio è stato netto: l'idea reggeva sulla carta, non come gioco.

1. **Il gioco premiava il controllo.** Dare un nome a una cosa per renderla reale è un atto di controllo. E "lascia andare" era un pulsante: un'altra mossa per piegare il mondo. È l'opposto del wu wei.
2. **Le regole non si potevano prevedere.** La realtà era nascosta. Il burrone diventava camminabile per un dettaglio che il giocatore non poteva conoscere. Senza deduzione non c'è puzzle, ci sono solo tentativi.
3. **Non c'era spazio.** Due stanze in fila: niente mappa, niente ritorni, niente "adesso so come passare di là". Di metroidvania non c'era niente.

## Le fonti
Per avere un vocabolario comune abbiamo letto, in parte, tre testi:
- **Edward Slingerland, *Effortless Action*** (Oxford University Press, 2003). Lette l'introduzione e i primi due capitoli. Idee chiave: il wu wei come azione senza sforzo e non come inazione; il paradosso del "provare a non provare"; le immagini del cuoco Ding, del contadino che tira i germogli, della scarpa che calza.
- **J.J. Abrams e Doug Dorst, *S.*** Letti la prefazione e l'inizio del capitolo 1. Idee chiave: il racconto a strati (testo, note del curatore con segnali nascosti, note a margine di due lettori che si rispondono nel tempo), gli inserti trovati tra le pagine, il tema dell'identità come nave di Teseo.
- **Twinetroid**, un librogame su Metroid portato su Twine da Devin Monnens. L'articolo non era raggiungibile dal nostro ambiente; ne abbiamo letto solo i riassunti. Idee chiave: un metroidvania è un grafo; i segreti si trovano osservando.

Dei libri abbiamo usato solo idee e riassunti con parole nostre. Nella repository non c'è il loro testo.

## Il passaggio chiave
Il prototipo chiedeva al giocatore di "lasciar andare" e glielo dava come pulsante. Il libro di Slingerland spiega perché non poteva funzionare: se il non-sforzo diventa un obiettivo, lo si insegue con sforzo.

La nuova versione rovescia il problema. Il gioco **non chiede mai** di lasciar andare. Al contrario, la Sala ti **incoraggia** a forzare, collezionare, spingere, come un metroidvania qualunque, e ogni sforzo la allarga. Il lasciar andare non è un comando. Lo scopre il giocatore, attraverso regole fisiche coerenti: correnti, germogli, corridoi che si allungano.

Il nuovo concept è in [`../CONCEPT.md`](../CONCEPT.md).

## Cosa resta del lavoro fatto
Il motore è intatto, e si riusa tutto: framebuffer, luce, font, particelle, versione web, headless. Del gioco restano la Sala e il suo "BENTORNATO.", che nella nuova storia ha finalmente un senso: sei davvero già stato qui.
