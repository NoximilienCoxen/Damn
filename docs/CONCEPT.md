# EGRESSO: game design doc

*Nome provvisorio. Versione 0.3: la Sala che cresce. Approvata; il primo prototipo (tre stanze) è nel codice.*

La versione precedente (il mondo scritto) è in [`archivio/CONCEPT-v0.2.md`](archivio/CONCEPT-v0.2.md); perché è stata messa da parte è raccontato in [`diario/04-revisione.md`](diario/04-revisione.md).

## Il gancio
**Un metroidvania che ti vuole tenere dentro. Ogni chiave che trovi allarga la prigione. L'uscita c'è dalla prima stanza.**

Platformer di esplorazione in vista laterale, alla Animal Well: si avanza **capendo**, non accumulando poteri (come Outer Wilds e Tunic). Il tono è ironico e un po' assurdo: la Sala è una padrona di casa premurosa che non vuole che tu te ne vada.

## Ispirazione e fonti
- **Adventure Time, *The Hall of Egress*.** Un eroe chiuso in una sala da cui più cerchi di uscire, meno esci. Prendiamo l'idea e il tema, non personaggi, dialoghi o immagini.
- **Wu wei, secondo *Effortless Action* di E. Slingerland.** Il wu wei non è "non fare". È agire senza sforzo e senza forzare, accordati con ciò che c'è intorno; richiede attenzione e spesso anni di pratica (il cuoco Ding). Ha dentro un paradosso: non si può *provare* a non provare.
- **"S." di J.J. Abrams e Doug Dorst.** Un racconto a strati. C'è il testo ufficiale, le note di un curatore che nascondono segnali, e le note a margine di due lettori che si rispondono nel tempo con inchiostri diversi a ogni rilettura. In più, oggetti infilati tra le pagine.
- **Twinetroid / la logica metroidvania.** Un metroidvania si può ridurre a un grafo: stanze, passaggi, condizioni per attraversarli. Qui le condizioni sono **cose capite**, non oggetti raccolti.

---

## 1. I pilastri

### Pilastro 1: la Sala risponde allo sforzo
Questa è la regola centrale. **Più forzi, più la Sala si allunga.** È una legge fisica visibile e prevedibile, non una punizione.

- **Cosa conta come "forzare":** spingere contro un muro, correre contro una corrente, saltare a ripetizione, tornare di corsa sui propri passi.
- **Come si vede:** i mattoni si moltiplicano, il corridoio si stira, le porte arretrano. La lanterna del protagonista brucia più forte.
- **Come si torna indietro:** un corridoio stirato torna alla sua misura quando lo attraversi senza forzare.
- **Non è passività:** non si vince stando fermi. Il wu wei richiede di agire nel modo giusto, non di smettere di agire.

### Pilastro 2: il wu wei sta nella fisica, non in un pulsante
Nessun tasto "lascia andare". Il gioco insegna col corpo, con poche regole fisiche coerenti:

| Elemento | Se forzi | Se ti accordi | Idea dal libro |
|---|---|---|---|
| **Correnti** (aria, acqua) | ti respingono, con fatica | ti portano lontano, anche lungo strade nascoste | "scorrere con" |
| **Germogli** | se li tocchi o li guardi troppo appassiscono | lasciati in pace crescono: torni e c'è una piattaforma | il contadino che tira i germogli |
| **Nodi** (passaggi stretti, ingranaggi) | se corri ti incastri | rallentando con attenzione passi | il cuoco Ding |
| **Creature** | se le insegui scappano | se resti calmo si radunano e fanno da ponte | la virtù che attira |

Il controllo preciso serve ancora: ci sono salti, tempi, attenzione. Il gioco premia l'abilità, ma un'abilità che si accorda col mondo invece di imporsi.

### Pilastro 3: si avanza capendo (metroidbrainia)
- La mappa è un **grafo di stanze collegate**, con ritorni e scorciatoie.
- I passaggi si aprono con la **conoscenza**: "le correnti portano", "i germogli crescono se non guardati", "il corridoio si accorcia". Le chiavi vere sono poche.
- La Sala però **regala** abilità e chiavi, come in un metroidvania classico: un doppio salto, una chiave d'ottone, una lanterna più forte. Funzionano davvero, ma ognuna apre stanze nuove e **allarga la Sala**. È la trappola: il giocatore abituato ai metroidvania collezionerà, e ogni dono lo allontana dall'uscita.
- **L'uscita c'è dalla prima stanza.** Chi sa come fare può finire il gioco in un minuto.

### Pilastro 4: il racconto sta sui muri, a strati (da "S.")
Non ci sono dialoghi né cutscene. Il racconto è scritto sui muri, in quattro strati:

1. **Le targhe del Curatore.** Didascalie ufficiali e pompose della Sala, piene di piccole anomalie volute: una data sbagliata, una parola fuori posto. Messe in fila, le anomalie compongono un messaggio.
2. **Le scritte dei prigionieri.** Note lasciate da chi è passato prima, ognuna in un **inchiostro** della propria epoca: gesso, carbone, ruggine, vernice che brilla al buio. Più scendi nella Sala, più le scritte sono antiche.
3. **Due prigionieri che si rispondono.** Il Cartografo vuole uscire con metodo: misura, colleziona chiavi, disegna mappe sempre più grandi. La Nuotatrice a un certo punto smette di cercare. Si rispondono da stanze lontane e da epoche diverse. Seguire il loro dialogo è anche seguire la mappa.
4. **Il tuo inchiostro.** Il protagonista può lasciare un segno sui muri. Verso la fine scopre che le scritte più antiche, quelle a matita, sono sue: è già stato qui. Per questo la Sala lo accoglie con "BENTORNATO.".

Ci sono anche gli **oggetti trovati**, come gli inserti di "S.": una lettera, un disco cifrante, una cartolina dall'esterno. Non sono chiavi: servono a capire.

### Pilastro 5: la Sala parla
La voce della Sala resta dal prototipo, ma cambia ruolo. Non spiega i comandi: **incoraggia lo sforzo**, ed è questo che la rende ironica. "BRAVO. CONTINUA." mentre il corridoio si allunga. Esulta a ogni chiave raccolta. Quando smetti di forzare si fa inquieta.

---

## 2. Il loop
1. Entri in una stanza e la Sala ti offre un obiettivo: una porta chiusa, una chiave, un dono.
2. Ci provi nel modo più ovvio (forzare, collezionare) e funziona, ma la Sala cresce.
3. Sui muri le scritte ti raccontano cos'hanno provato gli altri. Le contraddizioni tra il Cartografo e la Nuotatrice fanno nascere una domanda.
4. Torni in una stanza già vista e ci passi in un modo nuovo: lasciandoti portare dalla corrente, aspettando un germoglio.
5. La mappa si richiude su se stessa: scorciatoie, stanze più piccole, meno strada.

## 3. La mappa (prima bozza, come grafo)

```
        [Archivio del Curatore]
               |
[Riva] -- [Atrio] -- [Galleria delle Correnti] -- [Serra]
  ^          |                 |                    |
  |      [Pozzo] ---- [Cella del Cartografo]  [Stanza della Nuotatrice]
  |          |  (scendendo: epoche più antiche)
  +--- la corrente della Riva porta fuori, da sempre
```

| Stanza | Cosa insegna | Cosa apre |
|---|---|---|
| **Riva** (inizio) | ti svegli sulla riva; il canale che porta all'arco è oltre una diga, e ci si arriva solo dall'alto, con l'aria | è l'uscita |
| **Atrio** | il corridoio che si allunga se corri | il resto della Sala |
| **Galleria delle Correnti** | le correnti respingono chi le forza e portano chi le segue | una via nascosta verso la Stanza della Nuotatrice |
| **Serra** | i germogli crescono se li lasci in pace | una scorciatoia verso l'alto |
| **Archivio del Curatore** | le targhe ufficiali mentono, le anomalie parlano | il messaggio nascosto del Curatore |
| **Pozzo** | più scendi, più le scritte sono antiche | la scoperta della matita, cioè il tuo inchiostro |
| **Cella del Cartografo** | la mappa più grande della Sala, e lui che non è mai uscito | il dubbio |
| **Stanza della Nuotatrice** | l'ultima scritta: si è lasciata portare | la comprensione |

**Il finale.** All'inizio, per entrare nella Sala, hai nuotato contro la corrente della Riva, e la Sala lo ha festeggiato come un successo. Si esce tornando alla Riva e lasciandosi portare da quella stessa corrente. Nessuno te lo dice in chiaro: lo capisci dalle correnti, dalla Nuotatrice e dalle tue vecchie scritte.

Si esce lasciandosi portare dalla corrente, **non stando fermi**. Ci si arriva solo se il giocatore ha capito come funzionano le correnti; nessun timer di inattività sblocca qualcosa.

## 4. Errori da evitare (lezione del prototipo e del libro)
1. **"Lasciar andare" non è mai un pulsante né un obiettivo dichiarato.** Sarebbe come provare a non provare.
2. **Stare fermi N secondi non deve sbloccare niente.** Premierebbe chi finge il wu wei.
3. **Le regole devono essere prevedibili.** Il giocatore deve poter dedurre cosa succederà, non andare per tentativi.
4. **La pratica conta.** Il cuoco Ding si è allenato anni. Il gioco non deve negare l'abilità, deve cambiarne la direzione.
5. **I tutorial non devono insistere.** Il contadino che tira i germogli li uccide. La Sala parla poco, e solo per tentarti.

## 5. Estetica
- Rimane lo stile del prototipo: pixel art 320×180 generata dal codice, buio, lanterne, glow, nessun file di asset.
- **Gli inchiostri** hanno un colore e una grana diversi per epoca. Alcuni si leggono solo al buio, altri solo alla luce della lanterna.
- L'acqua e l'aria sono visibili come particelle che scorrono: la corrente si deve **vedere** prima di sentirla.

## 6. Cosa resta e cosa cambia
- **Resta tutto il motore:** framebuffer, luce, glow, font, particelle, collisioni, versione web e modalità headless.
- **Resta la Sala**, con un ruolo nuovo, e il font scritto a mano, che diventa la base degli inchiostri.
- **Va in archivio:** nomi, tre voci, parole che invecchiano, realtà nascosta.

## 7. Il primo prototipo (fatto)
Tre stanze: Riva, Atrio e Galleria delle Correnti. Ci sono:
- la regola dello sforzo: la Riva e l'Atrio hanno un tratto che si ripete quando forzi e si ritira quando li attraversi con calma;
- le correnti d'acqua e d'aria, con la regola "chi si muove resta dov'è, chi si lascia andare viene portato";
- il vento a raffiche dell'Atrio, con i ripari;
- tre inchiostri sui muri: il Cartografo, la Nuotatrice, il Curatore;
- l'uscita vera: dalla Riva, salire con l'aria sulla mensola, scendere nel canale e lasciarsi portare sotto il labbro di roccia.

Una scoperta del primo test: se il canale è raggiungibile a nuoto dalla riva, si esce dopo dieci secondi, per caso, lasciando il pulsante. Per questo c'è la diga. **Ogni uscita deve chiedere almeno due cose capite**, qui l'aria e l'acqua.

Il prototipo è riuscito se chi lo prova scopre l'uscita **senza che gliela si spieghi**.

## 8. La scala del gioco
Tre stanze servono a provare le regole, non bastano a viverle. Il gioco completo ha bisogno di **molte più stanze**: la mappa del punto 3 è solo l'ossatura, e ogni sua voce diventerà un gruppo di stanze. Si cresce un pezzo alla volta, verificando ogni pezzo giocandolo:
1. completare l'ossatura (Serra, Archivio, Pozzo, le due celle), una regola nuova per volta;
2. i doni della Sala: abilità e chiavi che funzionano ma allargano la mappa;
3. il tuo inchiostro e la scoperta della matita;
4. una mappa consultabile, che registra le stanze ma non i segreti.

## 9. Domande aperte
- Il segno che lascia il giocatore: scrive parole libere, o solo un simbolo?
- Quanto si deve vedere la regola dello sforzo? Un indicatore, o solo il mondo che cambia?
- Il Curatore e la Sala sono la stessa voce?
- Un solo finale, o anche quello "classico": uscire con tutte le chiavi, e scoprire di essere in una sala più grande?
