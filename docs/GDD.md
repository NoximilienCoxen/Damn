# GDD — Racer mobile (titolo provvisorio)

> Documento di design vivo. Si aggiorna un blocco alla volta.
> Ultimo aggiornamento: 12/11/2026 — completati i blocchi **Guida (core gameplay)** e **Trigger**.

---

## 0. Visione

Racer arcade per smartphone, spiritualmente ispirato a Split/Second: la gara è un game show televisivo, la **pista è l'arma**. Chi guida bene riempie una barra e la spende per far crollare l'ambiente addosso agli avversari.

**Nota legale:** nome, logo, stile visivo e asset vanno fatti da zero. Split/Second è un'ispirazione, mai un riferimento visibile.

### Pilastri (bozza)
1. **La pista è l'arma**: l'attacco passa dall'ambiente, non da armi sull'auto.
2. **Si guida davvero**: niente guida automatica, niente corsie. L'abilità di guida si traduce direttamente in barra.
3. **È uno show**: spettacolo, pubblico, commentatore, eventi. Il caos è parte del gioco.
4. **Tutto si può schivare**: ogni pericolo è annunciato e chi ha riflessi viene premiato.

---

## 1. Decisioni di base

| Tema | Decisione |
|---|---|
| Piattaforma | Mobile (iOS / Android) |
| Orientamento | **Orizzontale** |
| Motore | **Godot** (scene e script testuali, adatti allo sviluppo con Claude Code) |
| Multiplayer | Obiettivo sì. Si lancia prima con IA, ma l'architettura nasce **pensata per la rete** |
| Modalità (ipotesi) | Stagione roguelike in singolo + gara classica multiplayer, con lo stesso core |
| Monetizzazione | **Da decidere** (premium o free-to-play) |

---

## 2. Controlli

### Pollice sinistro: sterzo relativo
- Tocchi in un punto qualsiasi della metà sinistra: quel punto diventa il centro.
- Trascini in orizzontale: più trascini, più sterzi (analogico).
- Accelerazione **automatica**.

### Pollice destro: drift + trigger
- **Pulsante drift** tenuto premuto: si entra in derapata, mollando si esce. Il drift dura finché il tocco resta sullo schermo, anche se il dito si sposta.
- **Pulsante trigger** (accanto al drift):
  - **Fuori dal drift**: tap sul pulsante trigger.
  - **In drift**: tenendo premuto, **swipe verso il basso**. Il trigger parte e il drift continua.
- **Blu vs rosso** (stesso comando): tap / swipe breve = **blu**; pressione lunga / swipe tenuto (circa 0,5 s) = **rosso**, con un anello che si riempie attorno al pulsante. *(Da confermare con il test sul telefono.)*
- Il pulsante si illumina solo se esiste un bersaglio valido (vedi blocco Trigger).

### Esclusi in modo esplicito
- ❌ Tilt / giroscopio
- ❌ Guida automatica o corsie (stile Asphalt)
- ❌ **Freno**: non serve, se ti schianti respawni già in accelerazione
- ❌ **Boost / nitro**: abilità pura, la barra serve solo per i trigger
- ❌ Mira manuale: non si tocca l'elemento in pista, si preme il pulsante e il gioco sceglie il bersaglio

---

## 3. Modello di guida

Arcade, non simulativo. L'auto è una **macchina a stati**:

| Stato | Comportamento | Entrata / uscita |
|---|---|---|
| Grip | Guida normale, aderenza piena | Stato base |
| Drift | Traiettoria larga, barra in sospeso che si accumula | Pulsante tenuto / mollato |
| Scia | Meno resistenza, barra in sospeso che si accumula | Dentro il cono dietro un avversario |
| Aria | Niente sterzo, solo una piccola correzione | Salto / atterraggio |
| Impatto | Perdita di velocità, recupero rapido | Muri, auto, detriti |
| Testacoda | Perdita di velocità e controllo per un attimo, resti in gara | Onda d'urto di un trigger |
| Eliminato | Respawn | Schianto pesante o trigger |

### Sterzo — parametri
- **Corsa massima**: sterzo pieno a circa il 15-20% della larghezza dello schermo.
- **Curva di risposta esponenziale**: movimenti piccoli = correzioni fini.
- **Sensibilità legata alla velocità**: più vai veloce, meno sterza lo stesso trascinamento (contro lo zig-zag tipico del touch).
- **Zona morta minima**, solo per assorbire il tremolio del pollice.

### Muri
Strisciare contro un muro **rallenta ma non ferma**. Un errore non deve mai costare l'intera gara.

---

## 4. Drift (cuore del gameplay)

- **Entrata**: tieni premuto il drift mentre sterzi. La direzione la decide lo sterzo in quel momento.
- **Durante**: lo sterzo regola **l'angolo della derapata** (verso l'interno stringe, verso l'esterno allarga), non la direzione.
- **Uscita perfetta**: non dipende da un timer. È perfetta se molli con l'auto **allineata alla direzione della strada** (angolo tra il vettore velocità e la tangente della pista sotto una soglia).
  - Ogni curva ha il suo momento giusto, che si impara guidando.
  - Il feedback deve essere chiaro: scritta, suono, vibrazione.
- **Ricompensa**: l'uscita perfetta dà un bonus di barra e un piccolo scatto di velocità.

---

## 5. Barra: guadagno e rischio

### Struttura
La barra ha **3 slot**. Il blu costa 1 slot, il rosso li consuma tutti e 3.

### Barra in sospeso
La barra accumulata durante drift e scia è **provvisoria** (colore diverso) finché non la incassi.

| Esito | Effetto |
|---|---|
| Uscita perfetta | Incassi tutto + bonus + scatto |
| Uscita normale | Incassi tutto |
| Strisciata leggera durante il drift | Perdi solo il bonus perfetto |
| Schianto vero durante il drift | Perdi circa il **50%** della barra in sospeso *(da regolare)* |
| Schianto fuori dal drift | La barra già incassata non si tocca mai |

### Fonti di barra
| Fonte | Abilità |
|---|---|
| Drift lungo + uscita perfetta | Tempismo |
| Scia (incassata uscendo per il sorpasso, + scatto) | Posizionamento, scelta del momento |
| Near miss su detriti, pericoli, auto | Riflessi |
| Schivare il trigger di un avversario | Riflessi (bonus grosso) |
| Eliminazioni con i trigger | Lettura della gara (rimborso di circa mezzo slot per avversario) |
| Salti / atterraggi puliti | Traiettoria |

### Catene
Azioni concatenate (drift → scia → near miss → drift) entro una finestra di 2-3 secondi alzano un **moltiplicatore**. Nella cornice dello show è "il pubblico che si scalda", con folla e commentatore che reagiscono.

**Obiettivo di bilanciamento:** un esperto riempie la barra **almeno il doppio** più veloce di un principiante.

---

## 6. Schianti e respawn

- **Durata**: 1,5-2 secondi.
- **Posizione**: leggermente **dietro** il punto dello schianto, sulla linea della pista (schiantarsi apposta non deve mai convenire).
- **Ripartenza** a circa il 70% della velocità, poi accelerazione normale.
- **1 secondo di immunità** (auto semi-trasparente): niente catene di eliminazioni.

---

## 7. Trigger

### 7.1 Principio
Il giocatore non mira: decide **quando** premere. Il gioco sceglie il bersaglio con regole fisse e leggibili. L'abilità d'attacco è il tempismo e la lettura dell'avversario.

### 7.2 Blu
- **Costo**: 1 slot. Si può usare sempre, anche a barra piena (il rosso non è mai obbligatorio).
- **Selezione**: parte il blu **più vicino all'avversario che hai subito davanti**. Se vicino a quell'avversario non c'è nessun blu pronto, si passa al **successivo avversario davanti** a te che abbia un blu vicino.
- **Conseguenza di design**: chi è in testa non ha avversari davanti, quindi **non può usare i blu**. È un recupero naturale per chi insegue, senza aiuti nascosti.
- **Nessun cooldown**: con 3 slot si possono lanciare 3 blu di fila, il limite è la barra.
- **Uso**: ogni blu si può attivare **una volta per giro** (l'elemento resta distrutto, con detriti, e torna al giro successivo).
- **Preavviso** per chi viene colpito: circa **0,8 s** *(da regolare)*.

### 7.3 Rosso
- **Costo**: tutti e 3 gli slot.
- **Selezione**: parte il rosso **più vicino davanti a te**, non legato a un avversario.
- **Posizione**: punti fissi della pista, rari e mai sovrapposti ai blu.
- **Uso**: ogni rosso **una volta per gara**, perché cambia la pista in modo permanente.
- **Effetto**: il cambio di percorso **elimina chi si trova nella zona di crollo** e devia tutti gli altri sul nuovo tracciato. Chi lo attiva conosce già la nuova pista, piccolo vantaggio in più.
- **Preavviso**: circa **1,5 s** *(da regolare)*. Più devastante, ma più leggibile perché colpisce una zona ampia.
- **Dinamica strategica**: chi è in testa accumula barra senza poterla spendere sui blu e punta al rosso per seminare gli altri. Chi insegue sceglie se spendere subito sui blu o tenere la barra per il rosso.

### 7.4 Due famiglie di trigger blu
| | Esplosione ad area | Oggetto in movimento |
|---|---|---|
| Esempio | Benzinaio, palazzo che crolla | Barile spinto in pista, auto che sbanda, palla da demolizione |
| Come colpisce | Per distanza dal centro (zone) | Per contatto fisico con l'oggetto |
| Esiti | Eliminazione, testacoda o niente | Eliminazione o niente |
| Abilità di chi subisce | Scegliere quanto rischiare passando | Leggere la traiettoria e schivarla |
| Abilità di chi attacca | Aspettare che l'avversario sia nella zona | Anticipare dove sarà l'avversario |

### 7.5 Esplosioni ad area: tre zone
| Zona | Dove | Effetto |
|---|---|---|
| **Nucleo** | Dentro o a ridosso dell'elemento (es. il benzinaio stesso) | Eliminazione sicura |
| **Onda d'urto** | Fascia attorno al nucleo, sulla strada vicina | Testacoda: perdi velocità e controllo, resti in gara |
| **Sicura** | Oltre l'onda | Niente (al massimo near miss con bonus di barra) |

- Le zone valgono **per tutti, anche per chi ha attivato il trigger**.
- Il **nucleo è immediato**; l'**onda si propaga in circa 0,5 s**, quindi è schivabile anche dopo l'esplosione.
- **Level design**: il percorso più corto passa dal nucleo, quello normale resta dentro l'onda. La scorciatoia è anche il punto più pericoloso.
- Le zone sono **disegnate a mano per ogni elemento** (non cerchi generici) e visibili, in due colori, quando il trigger è pronto.

### 7.6 Oggetti in movimento
- **Collisione binaria**: chi viene colpito esplode ed è eliminato, chi schiva di striscio non subisce nulla (e prende il near miss).
- **Solo il colpito** esplode: nessun raggio che coinvolge l'auto accanto.
- Il barile **si ferma alla prima auto** colpita, quindi chi sta davanti fa da scudo a chi sta dietro.
- Un barile **mancato** continua a rotolare e può colpire chi segue, oppure resta in pista come detrito.
- Il **preavviso è la traiettoria**: il tempo che l'oggetto impiega ad arrivare è la finestra per schivare. L'attaccante deve anticipare dove sarà l'avversario.
- Si mostra la **corsia di lancio** quando il trigger è pronto.

### 7.7 Autodanno e premi
- Il proprio trigger **può colpire chi lo attiva** (vale la regola delle zone).
- **Eliminazione** di un avversario: rimborso di circa mezzo slot di barra.
- **Schivare** un trigger altrui: bonus di barra grosso.

### 7.8 Vincolo tecnico (multiplayer)
Esplosioni e oggetti in movimento **non usano fisica libera**: sono **traiettorie scriptate** (percorso predefinito con piccole variazioni controllate) con hitbox semplici. Lo stato è identico per tutti i giocatori e si sincronizza con eventi tipo "il trigger X è partito al tempo T".

---

## 8. Pericoli e distruzione (eventi dello show)

### Due famiglie
1. **Eventi dello show** (ambientali, nessuno li attiva): detriti, barili, elicottero, sezioni che crollano a ogni giro.
2. **Trigger dei giocatori** (vedi sezione 7).

### Regole
- **Tutto si annuncia prima**: avviso visivo 1-1,5 secondi prima (ombra, zona che lampeggia, scintille) + segnale audio.
- **Colori diversi** per eventi dello show e trigger avversari.
- **Tutto si può schivare**, e schivare dà barra.
- **Il tempo di preavviso** è la leva principale di bilanciamento della forza dei trigger.
- **Detriti persistenti**: i resti dei crolli possono restare in pista per uno o più giri come nuovi ostacoli.

---

## 9. Senso di velocità e feedback

- FOV che si allarga con la velocità
- Telecamera elastica (resta indietro in accelerazione, si sposta di lato in drift)
- Linee di velocità e leggero motion blur ai bordi
- **Vibrazione** su impatti e uscite perfette
- Oggetti a bordo pista vicini e ripetuti

---

## 10. Livelli di abilità

| | Principiante | Intermedio | Esperto |
|---|---|---|---|
| Sterzo | Evita i muri | Prende le corde | Linea ottimale |
| Drift | Brevi, per curvare | Uscite perfette occasionali | Lunghi, in catena, quasi sempre perfetti |
| Scia | Capita per caso | La cerca | La usa per sorpassare al momento giusto |
| Difesa | Viene colpito spesso | Schiva i pericoli dello show | Schiva i trigger avversari |
| Attacco | Preme quando il pulsante si accende | Aspetta che l'avversario sia nella zona | Anticipa i movimenti, attiva in drift con lo swipe |
| Barra | Si riempie lentamente | Un trigger a giro | Rosso disponibile spesso |

---

## 11. Parametri da regolare (solo con test sul telefono)

- Corsa dello sterzo e curva di risposta
- Sensibilità in funzione della velocità
- Soglia d'angolo per l'uscita perfetta
- Velocità di accumulo della barra in drift e in scia
- % di perdita della barra in sospeso negli schianti
- Finestra e moltiplicatore delle catene
- Durata del respawn, velocità di ripartenza, immunità
- Tempo di preavviso dei pericoli e dei trigger (blu ~0,8 s, rosso ~1,5 s)
- Tempo della pressione lunga per il rosso (~0,5 s) e soglia dello swipe in drift
- Dimensione di nucleo e onda d'urto, tempo di propagazione dell'onda
- Rimborso di barra per eliminazione e bonus per schivata

---

## 12. Domande aperte

- Monetizzazione: premium o free-to-play?
- Le auto hanno statistiche diverse che incidono sulla guida, o sono solo estetiche?
- Il comando blu/rosso (tap vs pressione lunga) regge nel caos della gara? Da verificare nel prototipo.
- Serve un aiuto in più per chi è molto staccato (oltre al recupero naturale dato dalla regola "il blu colpisce chi hai davanti")?

---

## 13. Prossimi blocchi

1. ✅ Guida (core gameplay)
2. ✅ Sistema dei trigger
3. ⬜ Piste: struttura, percorsi alternativi, cambi di mappa
4. ⬜ Auto e classi
5. ⬜ IA dei concorrenti (come guida e come usa i trigger)
6. ⬜ Modalità: stagione roguelike, multiplayer
7. ⬜ Progressione e monetizzazione
8. ⬜ Architettura tecnica (pensata per la rete)

### Primo prototipo previsto (quando sarà il momento)
Un'auto, una pista grey-box, niente avversari, niente trigger. Serve solo a capire se **guidare è bello**.
