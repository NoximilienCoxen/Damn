# EGRESSO: game design doc

*Nome provvisorio. Versione 0.1.*

## Il gancio
**Il gioco disegna ciò che il personaggio ricorda, non ciò che esiste. E a occhi chiusi i ricordi diventano realtà.**

## Ispirazione
Lo spunto viene dall'episodio di Adventure Time *The Hall of Egress* (stagione 7, di Tom Herpich, 2016). Un eroe resta chiuso in un dungeon, e l'unica porta si attraversa solo a occhi chiusi; appena li riapre, si ritrova al punto di partenza. Una lettura diffusa dell'episodio è legata al **Wu Wei**, il "non-agire" taoista: l'eroe esce solo quando smette di voler controllare tutto.

Dall'episodio prendiamo l'idea e il tema, **non** personaggi, dialoghi o immagini.

## Il nome
"The Hall of Egress" è il titolo dell'episodio (Cartoon Network/WBD) ed è già usato da un fan game su itch.io. Per evitare confusione e restare trovabili, il gioco ha un nome suo. Prima di un'uscita commerciale va fatta una verifica formale dei marchi (EUIPO/USPTO).

## Meccaniche

### Occhi aperti
È il dungeon reale, in pixel art scura con luci dinamiche (riferimento visivo: Animal Well). È un labirinto che si richiude su se stesso, e l'uscita non c'è.

### Occhi chiusi
Lo schermo mostra la **memoria** del personaggio, ricostruita da ciò che ha visto:
- ogni tile ricordato ha un valore di **fiducia**, che sale guardandolo e cala col tempo;
- con fiducia alta il ricordo è nitido, con fiducia media è sfocato e reso a dithering;
- con fiducia bassa il ricordo **muta** in qualcosa di assurdo: uno spuntone diventa una paperella di gomma, una porta un frigorifero.

### Il twist
A occhi chiusi il mondo *è* la memoria: le collisioni si calcolano sui ricordi, non sulla realtà. Un burrone ricordato male come ponte si può attraversare. I puzzle si risolvono scegliendo **cosa guardare, quanto a lungo e cosa lasciar dimenticare**.

### Tensione
- Il tempo a occhi chiusi è limitato, perché la palpebra si stanca.
- Ogni volta che chiudi gli occhi, la memoria si deteriora un po'.
- Riaprirli ti riporta nel mondo reale, dove i ponti immaginari non esistono.

## Il tema: lasciar andare
Il protagonista è un maniaco del controllo, e la progressione è **al contrario**: il gioco ti dà strumenti di controllo e poi ti insegna a mollarli.

| Strumento di controllo | Cosa dà | Perché lasciarlo |
|---|---|---|
| **Il filo**, srotolato per non perdersi | il ritorno sicuro | ti ancora al mondo a occhi aperti; tagliarlo apre zone nuove |
| **Fissare** (tenere premuto "osserva") | ricordi nitidi | i puzzle avanzati chiedono di lasciar sbiadire una zona, così che si ricomponga |
| **Gli attaccamenti** (diario, mappa, lanterna) | abilità | pesano; per proseguire vanno lasciati uno a uno |

### Il finale senza input
Nell'ultima stanza l'uscita compare solo se il giocatore **smette di premere tasti**. A occhi chiusi il personaggio cammina da solo e il mondo-memoria scorre come un fiume attorno a lui; ogni tasto premuto lo riporta all'inizio della stanza.

## Tono
Ironico che si fa via via più quieto. Il personaggio si lascia biglietti ("NON fidarti della paperella", "ho contato 412 mattoni. Il 413 mi guarda"), e un narratore-memoria sbaglia con convinzione. Verso la fine i biglietti si diradano e si fanno sereni.

## Vincoli di produzione
- Risoluzione interna 320×180, palette di circa 12 colori.
- Sprite 8×8 scritti come testo nel codice: nessun file immagine.
- Livelli disegnati in ASCII nel codice.
- Audio sintetizzato a runtime: nessun file audio.
- Durata 1-2 ore, circa 30 stanze, una sola area.
