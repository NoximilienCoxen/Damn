class_name CarController
extends CharacterBody3D
## Controller arcade custom (niente VehicleBody3D / RigidBody3D).
## Legge SOLO CarInput. Simulazione in _physics_process.

enum State { GRIP, DRIFT, IMPACT, CRASHED, RESPAWN }

signal state_changed(new_state: State)
## Contatto con un muro (anche sfregamento): impact = componente normale della velocità (m/s).
signal wall_contact(impact: float, hit_speed: float)
signal crashed(impact: float)

const CAR_HEIGHT := 0.6  ## quota fissa dell'origine dell'auto
const IMPACT_STATE_TIME := 0.25

@export var tuning: CarTuning
@export var car_input: CarInput
@export var track_info: TrackInfo

var state: State = State.GRIP
var speed := 0.0  ## velocità avanti (m/s), per HUD e camera
var long_accel := 0.0  ## accelerazione avanti istantanea (m/s²), per la camera
var yaw_norm := 0.0  ## rotazione corrente / rotazione massima, -1..1 (negativo = destra), per camera e carrozzeria

var _yaw := 0.0
var _yaw_rate := 0.0
var _tilt := Vector2.ZERO  ## x = beccheggio, y = rollio (rad, smussati)
var _state_timer := 0.0
var _crash_speed := 0.0
var _materials: Array[StandardMaterial3D] = []


func _ready() -> void:
	motion_mode = CharacterBody3D.MOTION_MODE_FLOATING
	var body_mat := StandardMaterial3D.new()
	body_mat.albedo_color = Color(0.9, 0.25, 0.2)
	var cabin_mat := StandardMaterial3D.new()
	cabin_mat.albedo_color = Color(0.15, 0.17, 0.22)
	_materials = [body_mat, cabin_mat]
	$Visual/Body.material_override = body_mat
	$Visual/Cabin.material_override = cabin_mat
	_refresh_visual()


## Posiziona l'auto e la orienta lungo `forward`. Velocità iniziale zero.
func place_at(pos: Vector3, forward: Vector3, start_speed := 0.0) -> void:
	global_position = Vector3(pos.x, CAR_HEIGHT, pos.z)
	_yaw = atan2(-forward.x, -forward.z)
	rotation = Vector3(0.0, _yaw, 0.0)
	velocity = forward * start_speed
	speed = start_speed
	long_accel = 0.0
	_yaw_rate = 0.0
	yaw_norm = 0.0


func _physics_process(delta: float) -> void:
	_state_timer -= delta
	match state:
		State.CRASHED:
			velocity = Vector3.ZERO
			speed = 0.0
			long_accel = 0.0
			yaw_norm = 0.0
			_yaw_rate = 0.0
			if _state_timer <= 0.0:
				_respawn()
		_:
			_drive(delta)


func _drive(delta: float) -> void:
	# 1. Sterzo: la rotazione dipende dalla velocità (più veloce = meno sterzo).
	var speed_ratio := clampf(speed / tuning.max_speed, 0.0, 1.0)
	var authority := lerpf(1.0, 1.0 - tuning.steer_speed_reduction, speed_ratio)
	# La rotazione segue lo sterzo con un po' di inerzia: dà peso e smorza il nervosismo.
	var max_rate := deg_to_rad(tuning.turn_rate)
	var target_rate := -car_input.steer * max_rate * authority
	_yaw_rate = lerpf(_yaw_rate, target_rate, 1.0 - exp(-tuning.yaw_response * delta))
	_yaw += _yaw_rate * delta
	yaw_norm = _yaw_rate / max_rate
	rotation.y = _yaw

	# 2. Componenti della velocità nel nuovo riferimento dell'auto.
	var fwd := _forward()
	var right := _right()
	var vf := velocity.dot(fwd)
	var vl := velocity.dot(right)

	# 3. Accelerazione automatica (e frenata passiva se sopra il massimo).
	var old_vf := vf
	vf = move_toward(vf, tuning.max_speed, tuning.accel * delta)
	# Curvare costa velocità: l'esterno della curva si paga, il rettilineo la ridà.
	vf = maxf(vf - tuning.turn_speed_loss * absf(yaw_norm) * speed_ratio * delta, 0.0)
	long_accel = (vf - old_vf) / delta

	# 4. Aderenza: la velocità laterale viene smorzata (in M2 il DRIFT abbassa questo valore).
	vl *= exp(-tuning.grip_lateral * delta)

	velocity = fwd * vf + right * vl
	velocity.y = 0.0

	# 5. Movimento e muri.
	var pre_velocity := velocity
	move_and_slide()
	global_position.y = CAR_HEIGHT
	velocity.y = 0.0
	_handle_walls(pre_velocity, delta)
	speed = maxf(velocity.dot(_forward()), 0.0)

	_update_tilt(delta)

	# 6. Timer degli stati.
	if state == State.IMPACT and _state_timer <= 0.0:
		_set_state(State.GRIP)
	elif state == State.RESPAWN and _state_timer <= 0.0:
		_set_state(State.GRIP)


func _handle_walls(pre_velocity: Vector3, delta: float) -> void:
	var count := get_slide_collision_count()
	if count == 0:
		return
	var impact := 0.0
	for i in count:
		var n := get_slide_collision(i).get_normal()
		n.y = 0.0
		if n.length_squared() < 0.0001:
			continue
		impact = maxf(impact, -pre_velocity.dot(n.normalized()))

	if impact > tuning.crash_threshold and state != State.RESPAWN:
		_crash(pre_velocity.length(), impact)
		return
	wall_contact.emit(impact, pre_velocity.length())

	# Sfregamento: rallenta e devia, non ferma.
	velocity *= maxf(0.0, 1.0 - tuning.wall_scrape_loss * delta)
	if velocity.length() > 2.0:
		var target_yaw := atan2(-velocity.x, -velocity.z)
		_yaw = lerp_angle(_yaw, target_yaw, 1.0 - exp(-tuning.wall_align * delta))
		rotation.y = _yaw
	if state == State.GRIP:
		_set_state(State.IMPACT)
	if state == State.IMPACT:
		_state_timer = IMPACT_STATE_TIME


func _crash(pre_speed: float, impact: float) -> void:
	crashed.emit(impact)
	_crash_speed = pre_speed
	velocity = Vector3.ZERO
	_state_timer = tuning.crash_wait
	_set_state(State.CRASHED)


func _respawn() -> void:
	# Sulla centerline, leggermente dietro al punto dello schianto.
	var off := track_info.closest_offset(global_position) - tuning.respawn_back
	var forward := track_info.tangent_at(off)
	place_at(track_info.point_at(off), forward, _crash_speed * tuning.respawn_speed_pct / 100.0)
	_state_timer = tuning.immunity_time
	_set_state(State.RESPAWN)


## Rollio e beccheggio solo visivi (non influenzano la simulazione).
func _update_tilt(delta: float) -> void:
	var lateral := yaw_norm * clampf(speed / tuning.max_speed, 0.0, 1.0)
	var target := Vector2(
		clampf(long_accel / tuning.accel, -1.0, 1.0) * deg_to_rad(tuning.body_pitch),
		-lateral * deg_to_rad(tuning.body_roll)
	)
	_tilt = _tilt.lerp(target, 1.0 - exp(-10.0 * delta))
	$Visual.rotation = Vector3(_tilt.x, 0.0, _tilt.y)


func _set_state(new_state: State) -> void:
	if state == new_state:
		return
	state = new_state
	_refresh_visual()
	state_changed.emit(new_state)


func _refresh_visual() -> void:
	for m in _materials:
		m.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA if state == State.RESPAWN else BaseMaterial3D.TRANSPARENCY_DISABLED
	if _materials.size() == 2:
		var a := 0.4 if state == State.RESPAWN else 1.0
		var crashed := state == State.CRASHED
		_materials[0].albedo_color = Color(0.25, 0.05, 0.05) if crashed else Color(0.9, 0.25, 0.2, a)
		_materials[1].albedo_color = Color(0.1, 0.1, 0.1) if crashed else Color(0.15, 0.17, 0.22, a)


func _forward() -> Vector3:
	return Vector3(-sin(_yaw), 0.0, -cos(_yaw))


func _right() -> Vector3:
	return Vector3(cos(_yaw), 0.0, -sin(_yaw))
