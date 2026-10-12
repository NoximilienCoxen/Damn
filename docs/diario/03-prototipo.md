# 03: Il prototipo

Il gioco (`src/game/`) usa l'engine per mettere in scena due stanze: l'Atrio, dove si impara a nominare, e l'Abisso, con tre voci, un abisso e una porta.

```
game/
  mod.rs         Egresso: tiene tutto insieme, update e regole
  lessico.rs     voci, nature degli oggetti, parole e i loro stati marci
  livello.rs     la stanza come dati: realtà in ASCII e percezioni
  mondo.rs       stato del mondo: percezioni, lettere, rivelazioni
  giocatore.rs   movimento e collisioni
  particelle.rs  luce e lettere che cadono
  render.rs      come si disegna tutto (e la pixel art procedurale)
```

## Due strati di mondo
**La realtà** è una griglia ASCII 40×22:
```
###############..........###############
###############==========###############
```
`#` è roccia, `=` il gradino, `O` il varco verso l'esterno. La realtà **non si vede e non si tocca**, tranne dentro una rivelazione.

**Le percezioni** sono zone della stanza, ognuna con i suoi attributi (`VUOTO`, `VENTO`, `ECO`) e fino a tre proposte di nome. Ogni proposta ha una voce, una parola e una posizione: per esempio il `PONTE` del Controllo occupa 10 celle orizzontali sopra l'abisso.

Solo le **lettere nominate** reggono. La funzione chiave è `Mondo::solido(tx, ty)`: una cella è solida se contiene una lettera di una parola affermata che "regge", *oppure* se la realtà è solida **e** in quel punto è aperta una rivelazione.

## La macchina a stati delle percezioni
```
Ignoto ──(ti avvicini)──▶ Sentito ──(0,9 s)──▶ Proposto ──E──▶ Affermato
   ▲                         │                    │               │
   └──────(ti allontani)─────┘                    R               R
   └──────────────(dopo la rivelazione)── Lasciato ◀──────────────┘
```
In Rust ogni fase è una variante di un `enum`, e alcune portano con sé dei dati: `Sentito(f32)` contiene da quanto tempo i sensi stanno lavorando. Il `match` del compilatore obbliga a gestire ogni caso.

## Le parole marciscono (`lessico.rs`)
Ogni parola è una catena di stati scritta a mano:
```rust
pub static PONTE: Lemma = Lemma {
    stati: &[st("PONTE", Solida, 0.66), st("PON E", Solida, 0.33), st("POSTE", Vuota, 0.0)],
    logorio: 0.012,       // certezza persa al secondo, sempre
    logorio_uso: 0.22,    // in più, mentre ci stai sopra
};
```
La **certezza** parte da 1 e scende. Quando passa sotto una soglia cambia lo stato: le lettere diverse si staccano e cadono (diventano particelle con gravità che si posano a terra), e la **natura** dell'oggetto cambia. `POSTE` non regge. `BURRONE`, lasciato marcire, diventa `BURRO`: solido ma scivoloso.

Le mutazioni sono scritte a mano di proposito: devono essere divertenti e utilizzabili nei puzzle, non casuali. I test controllano che tutti gli stati di una parola abbiano la stessa lunghezza (lo spazio è una lettera caduta) e che le soglie scendano.

## Collisioni (`giocatore.rs`)
Il corpo è un rettangolo 6×12 (AABB). Ogni passo:
1. si muove in **x** e, se entra in una cella solida, lo si rimette appena fuori dal lato da cui è entrato;
2. poi si muove in **y**, allo stesso modo. Se è stato fermato scendendo, è **a terra**, e si annota *su cosa* (per il burro).

Separare gli assi è il trucco classico dei platformer: evita quasi tutti i casi ambigui negli angoli.

Due finezze di game feel:
- **coyote time**: si può ancora saltare per 0,1 s dopo aver lasciato il bordo;
- **jump buffer**: se premi il salto 0,12 s prima di atterrare, vale lo stesso.

E il salto variabile: se lasci il tasto presto, la velocità verso l'alto viene tagliata e il salto è più basso.

## La rivelazione
`R` (lascia andare) scioglie le lettere in particelle e apre un cerchio sulla realtà, che dura 5 secondi. Il raggio si apre con una curva morbida e si richiude nell'ultimo secondo. Dentro il cerchio:
- si disegna la **pixel art del dungeon vero**, calcolata una volta sola all'avvio (`genera_realta`): mattoni con variazioni pseudo-casuali, muschio sulle superfici libere, lanterne, il varco luminoso, e l'illuminazione già "cotta" dentro;
- il bordo è a **dithering** con una matrice di Bayer 4×4: la realtà affiora a puntini invece di avere un taglio netto;
- la realtà **regge**: il burrone era un gradino.

## Il puzzle e i due finali
- **Strada del controllo:** affermi `PONTE` e passi (in fretta, prima che diventi `POSTE`), poi affermi `PORTA` e la attraversi. Torni all'Atrio, cioè la stanza ricomincia. Il biglietto lo diceva: *"Ogni porta che ho nominato mi ha riportato qui."*
- **Strada del lasciare:** lasci andare l'abisso e scendi sul gradino vero, poi lasci andare la porta. Sotto c'è un varco di luce. Lo attraversi: *"Non serviva un nome."*

Ci sono anche strade assurde: `FRIGO` come pilastro a metà abisso, `TRAPPOLA` che ti respinge, `ARMADIO` che ti sbarra la strada.

## Cosa abbiamo imparato giocandolo (in headless)
- Le proposte all'inizio sembravano parole già vere, perché avevano lo stesso colore pieno. Adesso sono **retinate**: l'incertezza si vede.
- Le tre proposte della porta, sulla stessa colonna, erano illeggibili: ora stanno su colonne vicine.
- Il titolo copriva il primo biglietto.

## Il primo test vero: "cammino nell'aria?"
Provato da telefono, il prototipo non si capiva. La regola (solo ciò che nomini regge) era giusta ma **invisibile**:
- il buio del vuoto era identico allo sfondo;
- le proposte sembravano già oggetti;
- l'unico suggerimento era una riga fissa in basso, staccata da ciò che stavi guardando;
- cadendo, la Paura diceva "VISTO? BURRONE.", che non spiega niente.

Correzioni, tutte nel segno del "mostrare, non spiegare":
1. **Una stanza in più, l'Atrio** (`livello.rs` ora contiene una lista di `Stanza`). Contiene un vuoto, una voce sola e una parola sola. Ci si cade una volta e si capisce. La PORTA nominata nell'Abisso riporta qui: il loop dell'episodio diventa letterale.
2. **Il vuoto si vede:** `Mondo::colonna_vuota` dice se sotto un punto non c'è niente che regga, e lì il render fa cadere pulviscolo.
3. **Bordo luminoso** sopra ciò che regge, **sbriciolamento** delle lettere solo proposte quando ci passi sopra.
4. **Suggerimenti contestuali** sopra il personaggio, che si spengono dopo due usi (un contatore per tasto).
5. **Messaggi di caduta che insegnano:** la prima volta "QUELLO CHE NON DICO, NON C'È.", sopra una proposta "NON L'AVEVO DETTO.".
6. **Un lampo** quando affermi: la parola passa da tratteggiata a piena e si accende.

Lezione di design: una regola forte va insegnata con una stanza che contiene *solo* quella regola. Poi si aggiungono le complicazioni.

## Cosa manca (prossimi passi)
- Luce con ombre proiettate dalle lettere sulle lettere.
- Lettere come corpi rigidi: che rotolino e si ammucchino davvero.
- Audio sintetizzato: le voci "dette" con suoni, il vento dell'abisso.
- L'Atrio come stanza vera, altre stanze, il finale senza input.
