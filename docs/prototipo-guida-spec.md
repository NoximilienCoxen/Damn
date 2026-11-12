# Specifica — Prototipo di guida (Godot)

> Documento per Claude Code. Contesto completo del gioco in `docs/GDD.md` (sezioni 2-6 sono quelle rilevanti qui).
> Obiettivo di questo prototipo: rispondere a **una sola domanda**: *guidare è bello su touch?*
> Lavorando in cloud su GitHub, leggere anche `docs/setup-repo-e-build-android.md`: dove i due documenti sono in conflitto, vale quello.

---

## 0. Regole di lavoro (leggere prima)

- **Scope rigido.** Si costruisce SOLO quello che è in questo documento. Niente avversari, niente trigger, niente menu, niente progressione, niente multiplayer, niente grafica rifinita. Se ti viene un'idea utile, scrivila in `NOTE.md` e non implementarla.
- **Si procede a milestone** (sezione 9). Dopo ogni milestone: fermati, riassumi cosa funziona e come provarlo sul telefono, e aspetta il feedback prima di passare alla successiva.
- **Il feeling si regola, non si indovina.** Ogni numero che influenza la guida deve essere un parametro modificabile a runtime (sezione 6). Niente costanti nascoste nel codice.
- **Commit piccoli e frequenti**, un cambiamento logico per commit.
- **Chiedi** quando un requisito è ambiguo, invece di scegliere in silenzio.

---

## 1. Stack e struttura

- **Godot 4.x** (ultima stabile), **GDScript**, renderer **Mobile**.
- Gioco **3D**, schermo **orizzontale** (landscape fisso).
- File di testo ovunque possibile (`.tscn`, `.gd`, `.tres`): il progetto deve restare leggibile e modificabile da Claude Code.

Struttura suggerita:

```
res://
  scenes/
    main.tscn            # scena di test: pista + auto + UI
    car/car.tscn
    track/test_track.tscn
    ui/hud.tscn, ui/touch_controls.tscn, ui/debug_panel.tscn
  scripts/
    car/car_controller.gd        # macchina a stati + movimento
    car/car_tuning.gd            # Resource con tutti i parametri
    car/car_input.gd             # astrazione input (vedi sotto)
    track/track_info.gd          # centerline e query sulla pista
    game/boost_bar.gd            # barra in sospeso / incassata
    ui/...
  tuning/
    default_tuning.tres
```

### Regola architetturale (importante per il multiplayer futuro)
**Separa l'input dalla simulazione.** `car_input.gd` produce uno stato semplice per frame:

```
steer: float   # -1..1
drift_held: bool
```

`car_controller.gd` legge SOLO questo e non sa se arriva da touch, tastiera o rete. Simulazione in `_physics_process` con passo fisso. Niente logica di gioco dentro gli script UI.

---

## 2. Controlli (da GDD sezione 2)

### Sterzo — metà sinistra dello schermo
- Il primo tocco nella metà sinistra definisce il **centro** (punto di origine, non fisso).
- Spostamento orizzontale rispetto a quel punto → `steer` in -1..1.
- Parametri: corsa massima in % della larghezza schermo (partenza **18%**), curva di risposta esponenziale, zona morta minima (partenza **~2%**).
- Mostrare un indicatore discreto: cerchio di origine + pallino che segue il dito.

### Drift — pulsante in basso a destra
- `drift_held = true` finché il tocco resta sullo schermo, **anche se il dito esce dal pulsante** (importante: servirà poi per lo swipe del trigger).
- Pulsante grande, area di tocco generosa.

### Accelerazione
Automatica, sempre. Niente freno, niente boost.

### Test sul PC
- Tastiera: frecce o A/D = sterzo, barra spaziatrice = drift.
- Abilitare `emulate_touch_from_mouse` per provare i controlli touch con il mouse.
- Il gioco va giudicato **sul telefono**, il PC serve solo per sviluppare.

---

## 3. Auto — macchina a stati

Non usare `VehicleBody3D` né `RigidBody3D` per la guida. Serve un **controller arcade custom** (es. `CharacterBody3D` o corpo cinematico con velocità integrata a mano), perché il feeling va tenuto sotto controllo e per la riproducibilità.

| Stato | Descrizione | Entrata / uscita |
|---|---|---|
| `GRIP` | Guida normale | Stato base |
| `DRIFT` | Derapata, barra in sospeso che si accumula | `drift_held` e sterzo oltre una soglia minima |
| `IMPACT` | Perdita di velocità dopo contatto con muro | Collisione leggera |
| `CRASHED` | Schianto pesante, auto ferma, attesa respawn | Collisione oltre soglia di velocità/angolo |
| `RESPAWN` | Rientro con immunità | Fine attesa |

*(Aria, scia, testacoda NON sono in questo prototipo.)*

### Movimento — idea di base
- La velocità avanti tende verso `max_speed` con accelerazione `accel`.
- Lo sterzo ruota il vettore direzione con una **velocità angolare dipendente dalla velocità** (più veloce = sterza meno a parità di input).
- In `GRIP` la velocità laterale viene smorzata rapidamente (aderenza alta).
- In `DRIFT` l'aderenza laterale scende: l'auto scivola e il suo asse si discosta dal vettore velocità. Lo sterzo **non** cambia la direzione di marcia ma l'**angolo di derapata** (verso l'interno stringe, verso l'esterno allarga).
- Muri: **rallentano e deviano**, non fermano. Solo oltre una soglia → `CRASHED`.

Non cercare realismo. Cerca controllo, leggibilità e piacere.

---

## 4. Drift e uscita perfetta

- Entrata in `DRIFT`: `drift_held` premuto mentre `|steer|` supera una soglia minima. Se si preme senza sterzare, non succede niente (non c'è freno).
- Durante il drift si accumula **barra in sospeso** a velocità proporzionale alla velocità e all'angolo di derapata.
- **Uscita**: al rilascio (o quando l'angolo scende sotto una soglia).
- **Uscita perfetta**: l'angolo tra il vettore velocità dell'auto e la **tangente della pista nel punto più vicino** è sotto `perfect_exit_angle` (partenza **12°**).
  - Per questo serve `track_info.gd`: dato un punto nel mondo, restituisce punto più vicino sulla centerline, tangente, e distanza laterale dal centro. Costruire la pista a partire da un `Path3D`/`Curve3D`.
- Esito dell'uscita:
  - **Perfetta**: incassi tutta la barra in sospeso + bonus + piccolo scatto di velocità + feedback forte
  - **Normale**: incassi la barra in sospeso
  - **Strisciata leggera** durante il drift: perdi solo il bonus perfetto
  - **Schianto vero** durante il drift: perdi il `50%` della barra in sospeso

---

## 5. Barra, schianto e respawn

### Barra (versione minima)
- Due valori: `pending` (in sospeso) e `banked` (incassata), entrambi mostrati nell'HUD con colori diversi.
- La barra ha 3 slot (0-3). Nessun trigger in questo prototipo: serve solo a **sentire il rischio/ricompensa**.

### Schianto e respawn (da GDD sezione 6)
- Attesa **1,5-2 s**.
- Respawn sulla centerline, **leggermente dietro** il punto dello schianto.
- Ripartenza al **70%** della velocità, poi accelerazione normale.
- **1 s di immunità** (auto semi-trasparente).

---

## 6. Parametri e pannello di debug (requisito fondamentale)

Tutti i parametri vivono in una `Resource` (`CarTuning`) salvata in `.tres`. Un **pannello di debug** nel gioco (apribile con un tocco in un angolo) li espone con slider e permette di:

- cambiarli **mentre si guida**
- **salvare** i valori correnti su file (JSON in `user://`)
- **ripristinare** i default
- **stampare/esportare** i valori attuali in forma copiabile, così possono essere riportati nel codice

Parametri minimi:

**Sterzo:** corsa massima (% schermo), esponente curva di risposta, zona morta, riduzione sterzo con la velocità
**Motore:** velocità massima, accelerazione
**Aderenza:** aderenza laterale in `GRIP`, aderenza laterale in `DRIFT`
**Drift:** soglia di sterzo per entrare, velocità di rotazione dell'angolo di derapata, velocità di accumulo barra, bonus uscita perfetta
**Uscita perfetta:** soglia d'angolo (°), scatto di velocità (valore e durata)
**Muri:** perdita di velocità allo sfregamento, soglia di schianto
**Schianto:** durata attesa, % velocità al respawn, durata immunità, % di barra in sospeso persa
**Camera:** FOV base e massimo, distanza e altezza, "elasticità" in accelerazione e in drift

---

## 7. Camera e feedback

- Camera dietro l'auto (terza persona), orizzontale.
- **FOV che cresce** con la velocità.
- Camera **elastica**: resta leggermente indietro in accelerazione e si sposta lateralmente in drift.
- Linee di velocità / leggero blur ai bordi (versione semplice, va bene anche uno shader basico o particelle).
- **Vibrazione** (`Input.vibrate_handheld`) su: uscita perfetta, schianto, sfregamento forte.
- **Suono**: segnaposto va bene (motore con pitch legato alla velocità, derapata, "ding" per l'uscita perfetta).
- Feedback chiaro dell'uscita perfetta: scritta "PERFETTO", suono, vibrazione, breve flash.
- Oggetti a bordo pista (pali, barriere) ripetuti e vicini alla strada, per dare senso di velocità.

---

## 8. Pista di test ("palestra di guida")

Una sola pista **grey-box** (cubi, rampe, nessuna texture), circuito chiuso, costruita attorno a una centerline `Curve3D`. Deve contenere:

- Un **rettilineo lungo** (per sentire velocità e FOV)
- Una curva **larga**, una **media**, una **a gomito**
- Una **chicane** (drift brevi in sequenza)
- Un tratto **stretto** (sterzo fine)
- Muri laterali su tutto il percorso
- Pali o segnali a bordo strada per il senso di velocità

*(La rampa e il salto non sono in questo prototipo: stato "Aria" rimandato.)*

Larghezza strada come **parametro**, per poterla cambiare facilmente: servirà per ricavare le misure delle piste vere.

---

## 9. Milestone

**M0 — Repo e pipeline APK** (vedi `docs/setup-repo-e-build-android.md`)
Progetto vuoto + GitHub Actions che produce l'APK Android debug. Si passa a M1 solo dopo che l'utente conferma di averlo installato e aperto sul telefono.

**M1 — Auto che si guida**
Auto, pista di test, sterzo relativo touch, accelerazione automatica, muri che rallentano, camera con FOV dinamico, schianto e respawn. Pannello di debug con i parametri di sterzo/motore.
*Test:* l'auto è piacevole da guidare anche senza drift? Lo sterzo è preciso senza essere nervoso?

**M2 — Drift**
Stato `DRIFT`, pulsante tenuto premuto, angolo di derapata controllato dallo sterzo, rilascio, vibrazione base.
*Test:* è divertente entrare e uscire da un drift? Si capisce cosa sta succedendo?

**M3 — Uscita perfetta e barra**
`track_info.gd`, calcolo dell'uscita perfetta, barra in sospeso/incassata nell'HUD, perdita del 50% in caso di schianto in drift, feedback di uscita (scritta, suono, vibrazione, scatto).
*Test:* si capisce *quando* uscire? La differenza tra chi guida bene e chi no si vede nella barra?

**M4 — Rifinitura**
Salvataggio dei parametri, pulizia del progetto, `NOTE.md` con le idee rimandate. L'export Android esiste già dalla M0.

---

## 10. Criteri di successo del prototipo

Il prototipo è riuscito se, giocando sul telefono per 5-10 minuti:

1. Guidare è piacevole **anche senza obiettivi**, senza il pannello di debug in mano
2. Lo sterzo non richiede precisione da controller e non risulta nervoso
3. Un drift riuscito **dà soddisfazione fisica** (vibrazione, suono, scatto)
4. Dopo alcuni giri l'utente **migliora in modo percepibile** (uscite perfette più frequenti)
5. Uno schianto è irritante ma non punitivo: si riparte in meno di 2 secondi

Se uno di questi cinque punti non regge, si corregge **qui**, prima di aggiungere avversari o trigger.

---

## 11. Fuori scope (rimandato)

Avversari e IA · trigger e pericoli · scia e near miss · salti e stato Aria · testacoda · catene e moltiplicatore · menu, progressione, salvataggi · multiplayer · piste vere · grafica definitiva · audio definitivo.
