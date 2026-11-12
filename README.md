# Racer Proto

Prototipo di un racer arcade per smartphone (Godot 4.7, orizzontale, renderer Mobile).
Design in [`docs/`](docs/): [GDD](docs/GDD.md), [spec del prototipo](docs/prototipo-guida-spec.md), [setup e build Android](docs/setup-repo-e-build-android.md).

Stato: **M1** — auto che si guida (sterzo touch, muri, schianto/respawn, pannello di debug).

## Controlli

- Telefono: tocca la metà sinistra dello schermo e trascina in orizzontale per sterzare (il punto del primo tocco è il centro). L'accelerazione è automatica.
- PC: frecce o A/D; il mouse emula il touch.
- Pannello di debug: tocca ⚙ in alto a destra. Gli slider cambiano i parametri mentre guidi; *Copia valori* mette i numeri negli appunti, *Ripristina* torna ai default.

## Scaricare l'APK

Ogni push su `main` produce un APK debug e lo pubblica nella pre-release
**"latest"** (tab *Releases* della repo). Dal telefono: apri la pagina Releases,
scarica `racer-proto-debug.apk`, aprilo e consenti l'installazione da origini
sconosciute quando richiesto. L'APK è firmato con una chiave di debug generata a
ogni build: per installare una build nuova sopra una vecchia, disinstalla prima la vecchia.

## Aprire il progetto in Godot

Servono Godot 4.7.x e (per esportare) gli export templates. Apri `project.godot`
da Project Manager. Il preset di export Android è in `export_presets.cfg`.
