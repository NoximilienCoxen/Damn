# Setup repo GitHub + build Android automatica

> Integra `docs/prototipo-guida-spec.md`. Dove i due documenti sono in conflitto, **vale questo**.
> Motivo: il lavoro avviene in una sessione cloud su una repo GitHub, non in locale.

---

## 1. Documenti nella repo

I documenti di progetto sono in `docs/`:

```
docs/GDD.md
docs/prototipo-guida-spec.md
docs/setup-repo-e-build-android.md   # questo file
```

---

## 2. Cosa cambia lavorando in cloud

- La sessione cloud **non ha schermo né telefono**: può scrivere ed eseguire codice, controllare che il progetto si apra e si esporti senza errori, ma **non può giudicare il feeling della guida**.
- Il giudizio sul feeling è **sempre dell'utente, sul telefono**. Per questo ogni milestone deve arrivare sul telefono come APK installabile.
- Conseguenza: la build Android non è l'ultima milestone (M4), è la **prima (M0)**.

---

## 3. M0 — Repo e pipeline APK (prima di M1)

Obiettivo: ogni push su `main` produce un **APK debug** scaricabile dal telefono.

### Da fare
1. Progetto Godot 4.x minimo (scena vuota con una scritta), orientamento **landscape**, renderer Mobile.
2. `export_presets.cfg` con preset **Android** committato nella repo. Package name provvisorio tipo `com.example.racerproto`. Nessun segreto nella repo.
3. **GitHub Actions** che, su push a `main`:
   - usa un'immagine/azione Godot per CI (es. `godot-ci`), con tag **uguale alla versione di Godot del progetto**
   - installa gli export templates
   - esporta l'APK **debug** (keystore di debug generato nel workflow, non committato)
   - carica l'APK come artifact e, meglio, lo pubblica come **pre-release "latest"** della repo, per scaricarlo comodamente dal telefono
4. `README.md` breve: cosa è il progetto, come scaricare l'APK, come aprire il progetto in Godot.

> Seguire la documentazione ufficiale dell'azione/immagine scelta per i dettagli di keystore ed export Android: non indovinarli.

### Test di M0 (lo fa l'utente)
- Scarica l'APK dal telefono, lo installa (serve abilitare "installa da origini sconosciute"), si apre e mostra la scritta in orizzontale.
- Se M0 non funziona, **non si passa a M1**: tutto il resto dipende da questo.

---

## 4. Flusso di lavoro per ogni milestone

1. La sessione cloud implementa la milestone su un **branch** (es. `m1-guida-base`).
2. Controlla da sola quello che si può controllare senza schermo (il progetto si carica, nessun errore di script, l'export funziona).
3. Apre una **pull request** verso `main` con: cosa è stato fatto, come provarlo, quali parametri sono regolabili dal pannello.
4. Dopo il merge, la pipeline produce l'APK.
5. L'utente prova sul telefono e riporta le impressioni (cosa gli sembra nervoso, lento, poco leggibile).
6. Le correzioni sono nuovi commit, non nuove funzioni.

I valori messi a punto col pannello di debug sul telefono vanno **riportati nel codice come nuovi default** (la sessione cloud non li vede): l'utente copia i numeri stampati dal pannello e li incolla nella conversazione.

---

## 5. Aggiornamento alle milestone

| Milestone | Cambia |
|---|---|
| **M0** (nuova) | Repo, progetto vuoto, pipeline APK |
| M1-M3 | Invariate. Ognuna si chiude con APK pronto da provare |
| M4 | Ridotta: salvataggio parametri, pulizia, `NOTE.md`. L'export ormai esiste già |

---

## 6. Primo messaggio per la sessione cloud

> Leggi `docs/GDD.md`, `docs/prototipo-guida-spec.md` e `docs/setup-repo-e-build-android.md`. Parti dalla **M0** (progetto vuoto + pipeline GitHub Actions che produce l'APK Android debug). Quando funziona, fermati e dimmi come scaricare l'APK. Non iniziare M1 finché non confermo che l'APK si installa e si apre sul telefono. Rispetta le regole di lavoro della sezione 0 della spec.
