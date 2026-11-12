class_name FollowCamera
extends Camera3D
## Camera in terza persona: FOV che cresce con la velocità, elastica in accelerazione.

@export var target: CarController
@export var tuning: CarTuning

var _cam_yaw := 0.0
var _accel_smooth := 0.0
var _speed_smooth := 0.0


func _ready() -> void:
	top_level = true
	near = 0.2
	far = 1500.0


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
	_update_transform()


func _update_transform() -> void:
	var fwd := Vector3(-sin(_cam_yaw), 0.0, -cos(_cam_yaw))
	var back := tuning.cam_distance + tuning.cam_accel_stretch * _accel_smooth
	var car_pos := target.global_position
	global_position = car_pos - fwd * back + Vector3.UP * tuning.cam_height
	look_at(car_pos + fwd * 6.0 + Vector3.UP * 1.0, Vector3.UP)
	fov = lerpf(tuning.fov_base, tuning.fov_max, _speed_smooth)
