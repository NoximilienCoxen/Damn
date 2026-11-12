class_name CarInput
extends Node
## Astrazione dell'input: la simulazione legge SOLO questo stato.
## Non sa se arriva da touch, tastiera o rete.

var steer := 0.0  ## -1..1
var drift_held := false  ## non usato in M1

## Scritti dai controlli touch.
var touch_steer := 0.0
var touch_active := false
var touch_drift := false


func _ready() -> void:
	process_priority = -100  # aggiorna prima dell'auto


func _physics_process(_delta: float) -> void:
	var kb := 0.0
	if Input.is_physical_key_pressed(KEY_A) or Input.is_physical_key_pressed(KEY_LEFT):
		kb -= 1.0
	if Input.is_physical_key_pressed(KEY_D) or Input.is_physical_key_pressed(KEY_RIGHT):
		kb += 1.0
	steer = touch_steer if touch_active else kb
	drift_held = touch_drift or Input.is_physical_key_pressed(KEY_SPACE)
