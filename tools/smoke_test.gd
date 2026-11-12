extends SceneTree
## Smoke test headless per la CI: godot --headless --fixed-fps 120 --script res://tools/smoke_test.gd
## Verifica che la scena si carichi, l'auto acceleri, e che schianto + respawn funzionino.

var main: Node
var car: CarController
var frames := 0
var saw_crash := false
var saw_respawn := false
var start_pos := Vector3.ZERO
var phase := 1


func _initialize() -> void:
	main = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(main)
	car = main.get_node("Car")


func _physics_process(_delta: float) -> bool:
	frames += 1
	if frames == 1:
		start_pos = car.global_position
	var input: CarInput = main.get_node("CarInput")
	if phase == 1 and frames == 600:  # 5 s dritto
		var moved := car.global_position.distance_to(start_pos)
		print("SMOKE phase1: speed=%.1f moved=%.1f state=%s" % [car.speed, moved, car.state])
		if car.speed < 10.0 or moved < 20.0 or car.state == CarController.State.CRASHED:
			return _fail("l'auto non accelera/avanza")
		phase = 2
		car.state_changed.connect(_on_state)
	if phase == 2:
		# Sterzo a fondo con i default: deve strisciare il muro, non schiantarsi.
		input.touch_active = true
		input.touch_steer = 1.0
		if frames == 600 + 720:  # 6 s
			print("SMOKE phase2: crash=%s speed=%.1f state=%s" % [saw_crash, car.speed, car.state])
			if saw_crash:
				return _fail("schianto con sterzo a fondo e soglia di default: soglia troppo bassa")
			phase = 3
			# Forza lo schianto: soglia bassa e auto puntata di traverso contro il muro.
			main.tuning.crash_threshold = 3.0
			input.touch_steer = 0.0
			car.place_at(car.global_position, Vector3(0.0, 0.0, 1.0), 20.0)
	if phase == 3:
		if saw_respawn:
			print("SMOKE_OK crash e respawn funzionano (frame %d)" % frames)
			quit(0)
			return true
		if frames > 600 + 720 + 2400:
			return _fail("nessun crash+respawn in 20 s (crash=%s respawn=%s state=%s)" % [saw_crash, saw_respawn, car.state])
	return false


func _on_state(s: int) -> void:
	if s == CarController.State.CRASHED:
		saw_crash = true
	if s == CarController.State.RESPAWN and saw_crash:
		saw_respawn = true


func _fail(msg: String) -> bool:
	printerr("SMOKE_FAIL: ", msg)
	quit(1)
	return true
