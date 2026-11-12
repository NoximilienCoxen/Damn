class_name Hud
extends Control
## HUD minimo: velocità e stato dell'auto.

var car: CarController

var _label: Label


func _ready() -> void:
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	_label = Label.new()
	_label.position = Vector2(24, 16)
	_label.add_theme_font_size_override("font_size", 30)
	_label.add_theme_color_override("font_outline_color", Color.BLACK)
	_label.add_theme_constant_override("outline_size", 6)
	add_child(_label)


func _process(_delta: float) -> void:
	if car == null:
		return
	_label.text = "%d km/h  %s\nsterzo %+.2f" % [roundi(car.speed * 3.6), CarController.State.keys()[car.state], car.car_input.steer]
