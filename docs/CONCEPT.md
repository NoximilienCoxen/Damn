# EGRESSO: game design doc

*Nome provvisorio. Versione 0.2: il mondo scritto.*

## Il gancio
**Il mondo è ciò che dici che sia. Per vederlo davvero devi smettere di nominarlo.**

**La regola, in una riga:** solo ciò che nomini regge. Il resto è vuoto, finché non lo lasci andare e scopri cosa c'era davvero.

Platformer di esplorazione e puzzle in vista laterale. Il protagonista è bendato in un dungeon senza uscita, e il mondo che attraversa è fatto delle **parole** con cui la sua mente prova a descriverlo.

## Ispirazione
Lo spunto viene dall'episodio di Adventure Time *The Hall of Egress* (stagione 7, di Tom Herpich, 2016). Un eroe resta chiuso in un dungeon, e l'unica porta si attraversa solo a occhi chiusi; appena li riapre, si ritrova al punto di partenza. Una lettura diffusa dell'episodio è legata al **Wu Wei**, il "non-agire" taoista: l'eroe esce solo quando smette di voler controllare tutto, e ne ricava una visione del mondo più chiara.

Dall'episodio prendiamo l'idea e il tema, **non** personaggi, dialoghi o immagini.

## Il nome
"The Hall of Egress" è il titolo dell'episodio (Cartoon Network/WBD) ed è già usato da un fan game su itch.io. Per evitare confusione e restare trovabili, il gioco ha un nome suo. Prima di un'uscita commerciale va fatta una verifica formale dei marchi (EUIPO/USPTO).

---

## 1. Chi decide cosa è cosa

### I sensi propongono
Le parole non le scrive un narratore: **nascono dai sensi**.

| Senso | Come si attiva | Cosa produce |
|---|---|---|
| Tatto | toccare, urtare, camminarci sopra | `FREDDO` `RUVIDO` `ALTO` `BAGNATO` |
| Udito | suoni dell'ambiente | `GOCCIA` `ECO` `PASSI` |
| Olfatto | stare fermi vicino a qualcosa | `MUFFA` `CERA` `PANE?` |

Gli attributi galleggiano vicino a ciò che li ha prodotti.

### Le voci indovinano
Dagli attributi la mente prova a indovinare un **nome**. Lo propongono tre voci interiori, ognuna con il suo carattere tipografico e il suo colore, che si contendono lo stesso punto dello spazio:

| Voce | Carattere | Esempio (per `FREDDO` + `RUVIDO` + `ALTO`) |
|---|---|---|
| **Il Controllo** | maiuscolo squadrato, bianco caldo | `MURO` |
| **La Paura** | tremante, rosso | `MOSTRO` |
| **La Memoria** | corsivo sbiadito, ciano | `FRIGORIFERO DELLA NONNA` → troppo lungo, si accorcia in `FRIGO` |

### Regola anti-prolissità
- Una voce dice **al massimo una parola**, mai frasi.
- Il "dialogo interiore" è la **gara** fra parole sovrapposte che tremano nello stesso punto: l'umorismo si legge a colpo d'occhio.
- Le frasi esistono solo nei rari **biglietti** che il protagonista si lascia ("NON fidarti della paperella"), che verso la fine si diradano.

### I due verbi
- **Affermare** una parola: diventa la realtà. Diventa solida, utilizzabile e illuminata.
- **Lasciarla andare**: la parola si scioglie (vedi sotto).

### La Sala: la voce del dungeon
Oltre alle tre voci interiori c'è una quarta voce, che non viene da dentro: **la Sala**, il dungeon stesso. Parla al centro dello schermo, lettera per lettera, con un respiro lento e un alone di luce.
- **Non nomina mai niente.** Al massimo ripete i nomi che dai tu ("LO CHIAMI PONTE. VA BENE."). È il contrario delle voci interiori.
- **Parla di rado**, solo nei momenti chiave, e ogni frase una volta sola: all'arrivo ("BENTORNATO."), alla prima caduta, al primo nome, quando un nome marcisce ("TUTTO CIÒ CHE DICI, INVECCHIA."), quando lasci andare ("ECCO. ADESSO MI VEDI."), quando una porta nominata ti riporta indietro ("OGNI PORTA CHE NOMINI È MIA.").
- **Premia l'inazione:** se resti fermo senza toccare niente per qualche secondo, dice "COSÌ. BRAVO.". È il seme del finale senza input.
- Tono: calmo, ironico, di chi sa già come va a finire. Non spiega mai le regole.
- Se un momento passa mentre sta ancora parlando, la frase scade e potrà dirla la prossima volta che capita: la Sala non parla mai fuori contesto.

## 2. Nominare è controllo, e il controllo marcisce
- Ciò che è affermato diventa un **oggetto fatto di lettere**: un `MURO` si può scalare, un `PONTE` attraversare, un `MOSTRO` ti insegue davvero.
- Le parole affermate **decadono** col tempo e con l'uso: le lettere si staccano, cadono con una fisica vera e si ammucchiano.
- Una parola che perde o cambia lettere **cambia significato** e quindi cambia oggetto: `PONTE` diventa `POSTE`, `PORTA` diventa `TORTA`, `MURO` diventa `MURA` e poi `MORA`.
- Tenere tutto sotto controllo costa: più cose nomini, più cose marciscono.

## 3. Lasciar andare è vedere
- Quando lasci andare una parola, le sue lettere si sciolgono in **particelle di luce** e sotto, per qualche istante, **appare il dungeon vero**: pixel art illuminata, la cosa più bella del gioco.
- La realtà spesso è diversa da come la temevi: il `BURRONE` della Paura era un gradino, il `MOSTRO` un'armatura vuota.
- Cosa vale di più, una parola utile ma che marcisce o uno sguardo vero ma fugace? Da questa scelta nascono i puzzle.

## 3b. Insegnare la regola senza spiegarla
Il primo test vero (da telefono) ha mostrato che la regola non si capiva: "cammino nell'aria?". Ora la si impara giocando:
- **L'Atrio**, la prima stanza, ha un solo vuoto e una sola proposta (`PONTE`). Se ci cammini sopra senza dirla cadi, e il Controllo dice "NON L'AVEVO DETTO.". Premi E e il ponte diventa solido.
- **Il vuoto si vede:** dove sotto i piedi non c'è niente di nominato, il pulviscolo cade nel buio.
- **Le proposte non reggono, e si vede:** le lettere tratteggiate si sbriciolano sotto i piedi.
- **Cosa regge si vede:** ciò che è nominato ha un bordo luminoso sopra, il segno che ci si può stare.
- **I suggerimenti compaiono sopra il personaggio**, solo per ciò che ha davanti ("E DILLO", "Q ALTRA VOCE", "R LASCIA"), e spariscono dopo due usi. Nell'Atrio si insegna solo E; R arriva nell'Abisso.

## 4. L'arco visivo è il messaggio
Il messaggio non viene mai spiegato: lo raccontano i verbi e l'immagine.

| Fase | Cosa vedi | Significato |
|---|---|---|
| **L'Atrio** (inizio) | l'unica stanza vista a occhi aperti: splendida, completamente renderizzata. La porta ti riporta sempre lì. | una prigione bella: il loop dell'episodio |
| **Il dungeon** (centro) | un mondo quasi solo di parole luminose; la realtà si vede a frammenti, quando molli il controllo | vedi solo ciò che controlli |
| **La stanza senza nome** (finale) | smetti di nominare e di premere tasti; le parole si dissolvono e il dungeon intero si rivela | la "prospettiva più chiara" |

### Il finale senza input
Nell'ultima stanza ogni tasto premuto fa nascere una parola, e ogni parola ti riporta all'inizio della stanza. Il personaggio avanza solo se il giocatore **non fa niente**: cammina da solo, le parole si spengono e il mondo vero si compone attorno a lui, come un fiume.

## 5. Tono
Ironico e assurdo all'inizio (le voci litigano, la Memoria nomina tutto come oggetti di casa della nonna), poi via via più quieto. Verso la fine le voci parlano meno, perché il giocatore ha imparato a non chiedere loro niente.

## 6. Direzione visiva
- **Vista laterale**, risoluzione interna 320×180.
- **Il mondo di parole è fatto di luce:** ogni lettera è una piccola sorgente luminosa (calda se certa, fredda se dubbia) e proietta ombre sulle altre. Un mondo tipografico, fisico e illuminato, non un terminale.
- **Il font è scritto nel codice:** glifi 5×7 come stringhe, nessun file.
- **La realtà** si vede solo a frammenti: pixel art procedurale (mattoni, muschio, rumore) con illuminazione dinamica. Servono a mano solo due set piece, l'Atrio e la rivelazione finale. Il layer della realtà è l'unico punto dove, se servisse, usare un tileset da itch.io o Humble Bundle.
- **Audio** sintetizzato a runtime.

## 7. Prior art
- *Typoman* (2015): il protagonista è fatto di lettere e compone parole per cambiare il mondo.
- *Baba Is You* (2019): le parole sono regole del livello.
- *Scribblenauts*: scrivi un nome e l'oggetto appare.

**Il nostro scarto:** qui la parola è la **percezione** del personaggio, non uno strumento. Le parole marciscono, e l'obiettivo è imparare a **smettere** di nominare per vedere.

## 8. Rischi aperti
- **Localizzazione:** i puzzle di mutazione dipendono dalle parole italiane. Va previsto un vocabolario per lingua e puzzle progettati sulle mutazioni, non sulle singole parole.
- **Leggibilità:** tante parole sovrapposte possono diventare rumore; serve una gerarchia chiara di luminosità e dimensione.
- **Durata:** 1-2 ore, circa 30 stanze, una sola area. Meglio corto e denso.
