class_name SpeedLines
extends Control
## Linee di velocità ai bordi dello schermo: intensità legata alla velocità. Solo grafica.

const SHADER_CODE := """
shader_type canvas_item;
uniform float intensity = 0.0;
float hash(float n) {
	return fract(sin(n * 127.1) * 43758.5453);
}
void fragment() {
	vec2 p = UV - vec2(0.5);
	p.x *= 1.8;
	float r = length(p);
	float a = atan(p.y, p.x);
	float seg = floor(a * 28.0 + 0.5);
	float t = floor(TIME * 18.0);
	float n = hash(seg + t * 13.37);
	float streak = step(1.0 - 0.18 * intensity, n);
	float edge = smoothstep(0.45, 1.0, r);
	COLOR = vec4(1.0, 1.0, 1.0, streak * edge * 0.55 * intensity);
}
"""

var car: CarController
var tuning: CarTuning

var _rect: ColorRect
var _mat: ShaderMaterial


func _ready() -> void:
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	var shader := Shader.new()
	shader.code = SHADER_CODE
	_mat = ShaderMaterial.new()
	_mat.shader = shader
	_rect = ColorRect.new()
	_rect.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_rect.material = _mat
	add_child(_rect)
	_rect.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)


func _process(_delta: float) -> void:
	if car == null or tuning == null:
		return
	var ratio := clampf(car.speed / tuning.max_speed, 0.0, 1.0)
	_mat.set_shader_parameter("intensity", smoothstep(0.45, 1.0, ratio) * tuning.speed_lines)
