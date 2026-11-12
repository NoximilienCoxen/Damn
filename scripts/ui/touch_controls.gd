class_name TouchControls
extends Control
## Sterzo relativo sulla metà sinistra dello schermo. Scrive SOLO su CarInput.
## Nessuna logica di gioco qui dentro. (Il pulsante drift arriva in M2.)

var car_input: CarInput
var tuning: CarTuning
## Callable(Vector2) -> bool: true se il punto è coperto da un'altra UI (es. pannello di debug).
var blocker: Callable

var _index := -1
var _origin := Vector2.ZERO
var _pos := Vector2.ZERO


func _ready() -> void:
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_IGNORE


func _input(event: InputEvent) -> void:
	if event is InputEventScreenTouch:
		var p := (make_input_local(event) as InputEventScreenTouch).position
		if event.pressed:
			if _index == -1 and p.x < size.x * 0.5 and not _is_blocked(event.position):
				_index = event.index
				_origin = p
				_pos = p
				_update_steer()
		elif event.index == _index:
			_release()
	elif event is InputEventScreenDrag and event.index == _index:
		_pos = (make_input_local(event) as InputEventScreenDrag).position
		_update_steer()


func _is_blocked(p: Vector2) -> bool:
	return blocker.is_valid() and blocker.call(p)


func _release() -> void:
	_index = -1
	car_input.touch_active = false
	car_input.touch_steer = 0.0
	queue_redraw()


func _update_steer() -> void:
	var w := size.x
	var travel := tuning.steer_travel_pct / 100.0 * w
	var dead := tuning.steer_deadzone_pct / 100.0 * w
	var dx := _pos.x - _origin.x
	var raw := clampf((absf(dx) - dead) / maxf(travel - dead, 1.0), 0.0, 1.0)
	car_input.touch_steer = signf(dx) * pow(raw, tuning.steer_expo)
	car_input.touch_active = true
	queue_redraw()


func _draw() -> void:
	if _index == -1:
		return
	draw_arc(_origin, 46.0, 0.0, TAU, 48, Color(1, 1, 1, 0.35), 3.0)
	draw_circle(Vector2(_pos.x, _origin.y), 20.0, Color(1, 1, 1, 0.5))
