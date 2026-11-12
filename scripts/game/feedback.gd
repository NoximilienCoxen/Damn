class_name Feedback
extends Node
## Feedback di guida: suono del motore (segnaposto), vibrazione, scossoni di camera.
## Ascolta i segnali dell'auto; non contiene logica di simulazione.

const ENGINE_RATE := 22000
const ENGINE_CYCLE := 200  ## campioni per ciclo → 110 Hz a pitch 1
const ENGINE_CYCLES := 8
const VIBE_COOLDOWN_MS := 110

var car: CarController
var cam: FollowCamera
var tuning: CarTuning

var _engine: AudioStreamPlayer
var _last_vibe_ms := 0


func setup(c: CarController, camera: FollowCamera, t: CarTuning) -> void:
	car = c
	cam = camera
	tuning = t
	car.wall_contact.connect(_on_wall_contact)
	car.crashed.connect(_on_crashed)
	_engine = AudioStreamPlayer.new()
	_engine.stream = _make_engine_stream()
	add_child(_engine)
	_engine.play()


func _process(_delta: float) -> void:
	if car == null:
		return
	var ratio := clampf(car.speed / tuning.max_speed, 0.0, 1.0)
	_engine.pitch_scale = lerpf(0.6, 2.4, ratio)
	_engine.volume_db = -80.0 if car.state == CarController.State.CRASHED else tuning.engine_volume_db


func _on_wall_contact(impact: float, hit_speed: float) -> void:
	var now := Time.get_ticks_msec()
	if now - _last_vibe_ms < VIBE_COOLDOWN_MS:
		return
	if impact > 4.0:
		_vibrate(int(clampf(40.0 + impact * 4.0, 40.0, 150.0)))
		cam.add_shake(clampf(impact / tuning.crash_threshold, 0.0, 1.0) * 0.3)
	elif hit_speed > 6.0:
		_vibrate(15)  # sfregamento: ronzio leggero


func _on_crashed(_impact: float) -> void:
	_vibrate(300)
	cam.add_shake(0.5)


func _vibrate(ms: int) -> void:
	if tuning.vibration <= 0.0:
		return
	_last_vibe_ms = Time.get_ticks_msec()
	Input.vibrate_handheld(ms, clampf(tuning.vibration, 0.1, 1.0))


## Motore segnaposto: onda a dente di sega + armoniche in loop, il pitch segue la velocità.
func _make_engine_stream() -> AudioStreamWAV:
	var n := ENGINE_CYCLE * ENGINE_CYCLES
	var data := PackedByteArray()
	data.resize(n * 2)
	for i in n:
		var p := float(i % ENGINE_CYCLE) / ENGINE_CYCLE
		var v := 0.5 * (2.0 * p - 1.0) + 0.4 * sin(TAU * p) + 0.2 * sin(TAU * 3.0 * p)
		data.encode_s16(i * 2, int(clampf(v, -1.0, 1.0) * 0.5 * 32767.0))
	var wav := AudioStreamWAV.new()
	wav.format = AudioStreamWAV.FORMAT_16_BITS
	wav.mix_rate = ENGINE_RATE
	wav.stereo = false
	wav.loop_mode = AudioStreamWAV.LOOP_FORWARD
	wav.loop_begin = 0
	wav.loop_end = n
	wav.data = data
	return wav
