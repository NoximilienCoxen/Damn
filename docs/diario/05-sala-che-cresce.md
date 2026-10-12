# 05. Il prototipo della Sala che cresce

Questo capitolo racconta come sono fatte, nel codice, le regole del concept v0.3. Il gameplay del "mondo scritto" è stato tolto: ne restano il motore, la Sala e la struttura del gioco.

## I file

| File | Cosa fa |
|---|---|
| `src/game/stanze.rs` | le stanze come dati: mappa ASCII, modulo ripetibile, correnti, scritte, lanterne, collegamenti |
| `src/game/mondo.rs` | la griglia di una stanza già allungata, la mappatura tra colonne base e colonne del mondo, le correnti |
| `src/game/giocatore.rs` | il corpo: platformer, nuoto, correnti, misura dello sforzo |
| `src/game/sala.rs` | la voce del dungeon, a copioni |
| `src/game/render.rs` | la stanza dipinta una volta, telecamera, lanterna, acqua, scie, scritte |
| `src/game/mod.rs` | le regole che legano tutto: sforzo, cambi di stanza, letture, finale |

## Una stanza che si allunga
Ogni stanza ha una mappa base di 40×22 tile. Alcune hanno un **modulo**: un gruppo di colonne (nella Riva 4, nell'Atrio 6) che il mondo ripete `moduli` volte in più. Con `moduli = 3` l'Atrio è largo 40 + 3×6 = 58 tile.

Il mondo costruisce la sua griglia colonna per colonna chiedendo "da quale colonna base vengo?" (`Mondo::x_base`): prima del modulo è la stessa colonna, dentro il modulo si ripete il modulo, dopo si scala di `moduli × larghezza`. All'opposto, tutto ciò che è definito sulla mappa base (correnti, scritte, lanterne) chiede "dove sono finito?" (`x_mondo`, e `x1_mondo` per l'estremo destro di un'area, che si allunga col modulo).

**Quando la stanza cambia lunghezza** (`Egresso::ridimensiona`):
- se il protagonista è oltre il tratto ripetuto, si sposta con il mondo, e la telecamera con lui: vedi muoversi solo la parte di là dal tratto;
- se è dentro il tratto, resta dov'è: le copie sono identiche, basta che resti dentro;
- la stanza viene ridipinta (è l'unica allocazione) e un po' di polvere e una piccola scossa fanno vedere che qualcosa è cambiato.

La lunghezza vera è un numero con la virgola (`Egresso::moduli`): **forzare** aggiunge 1,3 moduli al secondo; **camminare senza forzare** ne toglie mezzo per ogni modulo percorso. Il mondo usa la parte intera.

## Cosa vuol dire "forzare"
`Giocatore::update` restituisce un `Moto` con il campo `sforzo`. Si forza quando:
- si spinge contro un muro (anche da fermi: lo controlla `muovi_x`);
- si va contro una corrente, d'acqua o d'aria;
- si salta "di foga": un contatore cresce a ogni salto e cala col tempo, e sopra una soglia anche saltare è forzare.

## Le correnti
Una corrente è un'area con una velocità, d'acqua o d'aria, e può soffiare a raffiche (periodo e durata). La regola del corpo è una sola, per entrambe:

> **Chi si muove resta dov'è; chi si lascia andare viene portato.**

- **In acqua**, con un tasto premuto si nuota: la testa resta fuori (`superficie - 4`) e la corrente frena appena (60%). Senza tasti, il corpo scende piano fino a 26 pixel sotto la superficie e prende la velocità della corrente. Così chi nuota urta un labbro di roccia che scende di due tile sotto il pelo dell'acqua, e chi si lascia andare ci passa sotto.
- **Nell'aria**, con i tasti la fisica è quella di sempre (in una corrente che sale si cade piano); senza, la corrente prende il corpo.
- **Il riparo**: una raffica orizzontale non arriva in un tile che ha roccia poco sopravento (`Mondo::riparato`). Nell'Atrio i blocchi bassi sono ripari.

Le correnti si **vedono** prima di sentirle: scie che scorrono, calcolate a ogni frame da un hash (nessuna particella da gestire). Le raffiche si annunciano con 0,8 secondi di scie leggere.

## Disegnare una stanza larga
La stanza intera viene dipinta una volta (`render::dipingi`): mattoni, roccia, muschio, luce delle lanterne e dell'arco, tutto calcolato pixel per pixel. A ogni frame si copia la finestra della telecamera. Sopra, cose che cambiano: la luce della lanterna del protagonista (più grande e più calda quando forza), l'acqua con il pelo che ondeggia, le scie, le scritte.

Le scritte si leggono in proporzione alla luce: il **gesso** con qualunque luce, il **carbone** solo con la tua lanterna da vicino, l'**ottone** delle targhe è una lastra che brilla appena c'è luce. Stare vicini a una scritta per 0,6 secondi la segna come letta. Una sola conta per il finale: quella della Nuotatrice nel corridoio alto della Galleria.

## Cosa ha insegnato il primo test
La prima versione della Riva aveva il canale raggiungibile a nuoto dalla riva. Provandola nel browser: nuoti a sinistra, lasci il pulsante, e dieci secondi dopo sei fuori, senza aver capito niente. È esattamente il rischio del concept: un'uscita "per caso" non insegna nulla.

La correzione è nella mappa, non nelle regole. Una diga separa la vasca dal canale; al canale si arriva dalla mensola in alto, e sulla mensola porta solo la colonna d'aria della riva, se ti lasci andare. **Ogni uscita deve chiedere almeno due cose capite.**

## Test
- `stanze`: mappe rettangolari, collegamenti simmetrici, scritte dentro lo schermo.
- `mondo`: il modulo allunga la stanza e le copie sono identiche; andata e ritorno della mappatura; superficie; riparo dal vento.
- `giocatore`: chi nuota urta il labbro, chi si lascia andare passa sotto; la colonna d'aria porta in alto; contro la raffica si forza.
- `game`: si parte sulla Riva già lunga; camminando si arriva all'Atrio; forzare allunga l'Atrio, camminare tra le raffiche no; nella vasca l'acqua è ferma; la colonna d'aria porta sulla mensola; dal canale lasciandosi portare si esce, nuotando no; leggere la Nuotatrice fa capire.

Con `--stanza N` la modalità headless parte da una stanza qualsiasi.
