class_name FollowCamera
extends Camera3D
## Camera in terza persona: FOV che cresce con la velocità, elastica in accelerazione.

@export var target: CarController
@export var tuning: CarTuning

var _cam_yaw := 0.0
var _accel_smooth := 0.0
var _speed_smooth := 0.0
var _roll_smooth := 0.0
var _shake := 0.0


func _ready() -> void:
	top_level = true
	near = 0.2
	far = 1500.0


## Scossone puntuale (impatti, schianto). Ampiezza in metri.
func add_shake(amount: float) -> void:
	_shake = maxf(_shake, amount)


func snap() -> void:
	_cam_yaw = target.rotation.y
	_speed_smooth = 0.0
	_update_transform()


func _physics_process(delta: float) -> void:
	if target == null or tuning == null:
		return
	_cam_yaw = lerp_angle(_cam_yaw, target.rotation.y, 1.0 - exp(-tuning.cam_follow_rate * delta))
	_accel_smooth = lerpf(_accel_smooth, clampf(target.long_accel / tuning.accel, 0.0, 1.0), 1.0 - exp(-4.0 * delta))
	_speed_smooth = lerpf(_speed_smooth, clampf(target.speed / tuning.max_speed, 0.0, 1.0), 1.0 - exp(-3.0 * delta))
	_roll_smooth = lerpf(_roll_smooth, target.yaw_norm * _speed_smooth, 1.0 - exp(-6.0 * delta))
	_shake = move_toward(_shake, 0.0, 1.5 * delta)
	_update_transform()
	var amp := tuning.cam_shake * 0.2 * _speed_smooth * _speed_smooth + _shake
	h_offset = randf_range(-1.0, 1.0) * amp
	v_offset = randf_range(-1.0, 1.0) * amp


func _update_transform() -> void:
	var fwd := Vector3(-sin(_cam_yaw), 0.0, -cos(_cam_yaw))
	var back := tuning.cam_distance + tuning.cam_accel_stretch * _accel_smooth
	var car_pos := target.global_position
	global_position = car_pos - fwd * back + Vector3.UP * tuning.cam_height
	look_at(car_pos + fwd * 6.0 + Vector3.UP * 1.0, Vector3.UP)
	rotate_object_local(Vector3.BACK, _roll_smooth * deg_to_rad(tuning.cam_roll))
	fov = lerpf(tuning.fov_base, tuning.fov_max, _speed_smooth)
