# 02: Il motore minimo

L'engine (`src/engine/`) contiene solo ciò che serve per giocare il prototipo. Ogni pezzo è piccolo e testato. Regola: niente in `engine/` sa cosa sia una parola o una benda.

```
engine/
  framebuffer.rs  l'immagine 320×180 in memoria + operazioni sui colori
  app.rs          finestra, game loop, modalità headless
  time.rs         passo fisso
  input.rs        tastiera
  font.rs         font 5×7 scritto a mano
  glow.rs         la luce che si diffonde
  math.rs         Vec2, Rect, numeri casuali
  png.rs          encoder PNG minimo
```

## La finestra (`app.rs`)
Usiamo solo due librerie esterne: **winit** apre la finestra e riceve gli eventi, **softbuffer** ci dà un array di pixel da riempire. Nient'altro: niente GPU, niente motore grafico.

Il gioco disegna sempre a 320×180. Per mostrarlo su uno schermo grande lo ingrandiamo di un **fattore intero** (×3 su una finestra 960×540, ×6 su un 4K): ogni pixel diventa un quadrato perfetto, senza sfocature. Lo spazio che avanza diventa una banda nera. In `scala_su` prima si allarga una riga in orizzontale, poi la si copia `scala` volte con `copy_within`, che è molto veloce.

Il gioco implementa il trait `Game`:
```rust
pub trait Game {
    fn update(&mut self, input: &Input);                 // un passo di simulazione
    fn render(&mut self, fb: &mut Framebuffer, stats: &Stats);
}
```
Un **trait** è un "contratto": l'engine non sa che il gioco si chiama Egresso, sa solo che ha `update` e `render`. È lo stesso principio delle interfacce in altri linguaggi.

## Il game loop a passo fisso (`time.rs`)
Lo schermo può aggiornarsi a 60, 144 o 30 Hz, ma la fisica deve comportarsi sempre allo stesso modo. Allora:

```
tempo_reale → accumulatore → spendi a fette da 1/60 s → update, update, ... → render
```

Se è passato 1/30 di secondo si fanno due update; se è passato 1/144 a volte nessuno. Il salto è sempre alto uguale, su qualunque PC. Un tetto di 5 passi per frame evita la "spirale della morte": se il computer resta indietro, il gioco rallenta invece di bloccarsi.

## L'input (`input.rs`)
Tre domande diverse: il tasto è **giù**? È stato **appena premuto**? È stato **appena rilasciato**? "Appena premuto" deve valere per un solo passo di update, ma gli eventi arrivano quando vogliono. Lo risolve un *latch*: la pressione resta memorizzata finché un update non l'ha vista, poi `fine_passo()` la cancella. Così anche un tocco rapidissimo, premuto e rilasciato fra due update, non si perde (c'è un test che lo verifica).

## Il font scritto a mano (`font.rs`)
Ogni carattere è disegnato come 7 stringhe da 5 caratteri:
```
('A', [".###.", "#...#", "#...#", "#####", "#...#", "#...#", "#...#"]),
```
All'avvio ogni glifo diventa 7 byte, uno per riga, in cui ogni bit è un pixel. Disegnare una lettera significa controllare 35 bit. Sono 62 glifi in tutto (lettere, cifre, punteggiatura, accentate), e nessun file.

Ci sono anche due varianti: il **corsivo finto** della Memoria (le righe alte spostate a destra) e le lettere **retinate** delle proposte (solo metà dei pixel accesi, a scacchiera che si muove). Una parola proposta *sembra* incerta.

## Il glow (`glow.rs`)
È la parte che rende il mondo di parole "fatto di luce". Il gioco disegna tutto ciò che emette luce anche in un secondo framebuffer, `luce`. Alla fine del frame:

1. si sfoca `luce` con un **box blur**: ogni pixel diventa la media dei vicini;
2. si somma il risultato all'immagine.

Il trucco per la velocità è la **somma scorrevole**: invece di sommare 2r+1 vicini per ogni pixel, quando ci si sposta di uno si aggiunge il pixel che entra e si toglie quello che esce. Il costo non dipende dal raggio. Il blur è **separabile**: prima per righe, poi per colonne. Ripetuto due volte approssima una gaussiana. Lo facciamo con due raggi (2 e 9) per avere un alone stretto e uno largo.

C'è anche un dettaglio da non perdere: lavoriamo in **virgola fissa** (valori moltiplicati per 256). Con gli interi puri, dividere a ogni passata azzererebbe le luci deboli. Senza la virgola fissa il test `un_punto_si_allarga_e_resta_simmetrico` fallirebbe: il punto sparirebbe del tutto.

## Memoria e performance
- Tutti i buffer (framebuffer, luce, glow, particelle) sono **allocati una volta sola**. Nel game loop non si crea nessun `Vec` o `String` (l'overlay di debug formatta il testo in un array sullo stack).
- Le particelle stanno in un `Vec` con capacità fissa. Quelle morte si tolgono con `swap_remove`: si mette l'ultima al posto di quella morta, quindi costa O(1) e l'ordine non conta.
- La mappa delle lettere si ricostruisce solo quando qualcosa cambia.
- Profili di compilazione: anche in debug il nostro codice è ottimizzato (`opt-level = 1`) e le dipendenze al massimo (`opt-level = 3`).

Premi **F3** in gioco per vedere FPS e millisecondi di update e render.

## L'encoder PNG (`png.rs`)
Per salvare screenshot senza librerie. Un PNG è una firma più dei "chunk" (IHDR con le dimensioni, IDAT con i pixel, IEND). I pixel vanno compressi con zlib, ma deflate ha un tipo di blocco "non compresso": usiamo quello. Servono solo due checksum, CRC32 e Adler-32, scritti a mano e verificati con valori noti nei test.

## La modalità headless
`cargo run -- --headless --script "..." --shots ...` fa girare il gioco **senza finestra**, con un input registrato, e salva PNG dei frame scelti. È uno strumento serio:
- si verifica la grafica dove non c'è uno schermo (per esempio in un server o in CI);
- un bug si riproduce sempre uguale, perché il gioco è deterministico (stesso seme casuale, passo fisso);
- con `--trace` si stampa lo stato a ogni frame. È così che abbiamo scoperto che il personaggio "bloccato" contro lo scalino in realtà faceva un salto minuscolo: lo script toccava il tasto per un solo frame, e il salto variabile lo accorciava.
