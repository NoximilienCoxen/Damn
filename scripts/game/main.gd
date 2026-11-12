extends Node3D
## Scena di test: cabla pista, auto, camera e UI. Nessuna logica di guida qui.

const TUNING_PATH := "res://tuning/default_tuning.tres"

@onready var track: TestTrack = $Track
@onready var car: CarController = $Car
@onready var car_input: CarInput = $CarInput
@onready var cam: FollowCamera = $FollowCamera
@onready var hud: Hud = $UI/Hud
@onready var touch: TouchControls = $UI/TouchControls
@onready var debug_panel: DebugPanel = $UI/DebugPanel

var tuning: CarTuning


func _ready() -> void:
	# Copia di lavoro: i cambi dal pannello non toccano il file dei default.
	tuning = (load(TUNING_PATH) as CarTuning).duplicate()

	car.tuning = tuning
	car.car_input = car_input
	car.track_info = track.track_info
	var forward := track.track_info.tangent_at(0.0)
	car.place_at(track.track_info.point_at(0.0), forward)

	cam.target = car
	cam.tuning = tuning
	cam.snap()

	hud.car = car
	touch.car_input = car_input
	touch.tuning = tuning
	debug_panel.setup(tuning, load(TUNING_PATH) as CarTuning)
	touch.blocker = debug_panel.blocks

	_build_environment()


func _build_environment() -> void:
	var sky_mat := ProceduralSkyMaterial.new()
	var sky := Sky.new()
	sky.sky_material = sky_mat
	var env := Environment.new()
	env.background_mode = Environment.BG_SKY
	env.sky = sky
	env.ambient_light_source = Environment.AMBIENT_SOURCE_SKY
	var we := WorldEnvironment.new()
	we.environment = env
	add_child(we)
	var sun := DirectionalLight3D.new()
	sun.rotation_degrees = Vector3(-50.0, -30.0, 0.0)
	add_child(sun)
