class_name CarTuning
extends Resource
## Tutti i parametri che influenzano guida, muri, schianto e camera.
## Nessuna costante nascosta nel codice: i valori vivono qui e si cambiano dal pannello di debug.
## I default sono duplicati in tuning/default_tuning.tres (quello è il file da aggiornare con i valori messi a punto).

# --- Sterzo ---
@export var steer_travel_pct := 18.0  ## corsa massima, % larghezza schermo
@export var steer_expo := 1.6  ## esponente curva di risposta (1 = lineare)
@export var steer_deadzone_pct := 2.0  ## zona morta, % larghezza schermo
@export var steer_speed_reduction := 0.5  ## quanto sterzo si perde alla velocità massima (0..1)
@export var turn_rate := 120.0  ## velocità di rotazione massima a bassa velocità (gradi/s)

# --- Motore e aderenza ---
@export var max_speed := 40.0  ## m/s
@export var accel := 14.0  ## m/s²
@export var grip_lateral := 12.0  ## smorzamento velocità laterale in GRIP (1/s)

# --- Muri ---
@export var wall_scrape_loss := 1.5  ## frazione di velocità persa al secondo durante lo sfregamento
@export var wall_align := 6.0  ## quanto in fretta l'auto si allinea al muro (1/s)
@export var crash_threshold := 18.0  ## velocità d'impatto contro il muro (m/s, componente normale) oltre cui è schianto

# --- Schianto e respawn ---
@export var crash_wait := 1.8  ## s
@export var respawn_speed_pct := 70.0  ## % della velocità
@export var respawn_back := 8.0  ## m dietro il punto dello schianto
@export var immunity_time := 1.0  ## s

# --- Camera ---
@export var fov_base := 70.0
@export var fov_max := 100.0
@export var cam_distance := 7.0
@export var cam_height := 2.8
@export var cam_follow_rate := 5.0  ## quanto in fretta la camera segue la direzione dell'auto (1/s)
@export var cam_accel_stretch := 2.0  ## m di distanza extra in accelerazione

## Tabella per il pannello di debug: [gruppo, proprietà, etichetta, min, max, step]
const PARAMS: Array = [
	["Sterzo", "steer_travel_pct", "Corsa (% schermo)", 5.0, 40.0, 0.5],
	["Sterzo", "steer_expo", "Esponente risposta", 1.0, 3.0, 0.05],
	["Sterzo", "steer_deadzone_pct", "Zona morta (% schermo)", 0.0, 8.0, 0.1],
	["Sterzo", "steer_speed_reduction", "Riduzione con la velocità", 0.0, 0.9, 0.01],
	["Sterzo", "turn_rate", "Rotazione max (°/s)", 30.0, 240.0, 1.0],
	["Motore", "max_speed", "Velocità max (m/s)", 15.0, 80.0, 0.5],
	["Motore", "accel", "Accelerazione (m/s²)", 3.0, 40.0, 0.5],
	["Aderenza", "grip_lateral", "Aderenza laterale GRIP", 1.0, 40.0, 0.5],
	["Muri", "wall_scrape_loss", "Perdita sfregamento (1/s)", 0.0, 6.0, 0.1],
	["Muri", "wall_align", "Allineamento al muro", 0.0, 20.0, 0.5],
	["Muri", "crash_threshold", "Soglia schianto (m/s)", 5.0, 40.0, 0.5],
	["Schianto", "crash_wait", "Attesa (s)", 1.0, 3.0, 0.05],
	["Schianto", "respawn_speed_pct", "Velocità respawn (%)", 0.0, 100.0, 1.0],
	["Schianto", "respawn_back", "Distanza dietro (m)", 0.0, 30.0, 0.5],
	["Schianto", "immunity_time", "Immunità (s)", 0.0, 3.0, 0.1],
	["Camera", "fov_base", "FOV base", 50.0, 90.0, 1.0],
	["Camera", "fov_max", "FOV massimo", 60.0, 120.0, 1.0],
	["Camera", "cam_distance", "Distanza", 3.0, 14.0, 0.1],
	["Camera", "cam_height", "Altezza", 1.0, 8.0, 0.1],
	["Camera", "cam_follow_rate", "Elasticità rotazione", 1.0, 20.0, 0.5],
	["Camera", "cam_accel_stretch", "Elasticità accelerazione (m)", 0.0, 6.0, 0.1],
]


func copy_from(other: CarTuning) -> void:
	for p in PARAMS:
		set(p[1], other.get(p[1]))


## Testo copiabile, una riga per parametro: "nome = valore".
func to_text() -> String:
	var lines: PackedStringArray = []
	for p in PARAMS:
		lines.append("%s = %s" % [p[1], str(snappedf(get(p[1]), 0.0001))])
	return "\n".join(lines)
