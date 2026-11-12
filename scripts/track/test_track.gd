class_name TestTrack
extends Node3D
## Pista grey-box "palestra di guida": circuito chiuso costruito attorno a una centerline Curve3D.
## Rettilineo lungo, curva larga, curva media, chicane, gomito, tratto stretto.
## Tutto è generato qui da una lista di segmenti, così la larghezza è un parametro.

@export var road_width := 18.0
@export var narrow_factor := 0.6  ## larghezza del tratto stretto rispetto a road_width
@export var wall_height := 2.0
@export var wall_thickness := 1.0
@export var pole_spacing := 14.0
@export var sample_step := 2.0  ## passo di campionamento della centerline (m)
@export var collider_every := 3  ## un collider ogni N campioni (6 m con il passo di 2 m)

## Segmenti: ["S", lunghezza] rettilineo, ["A", raggio, gradi] arco (gradi > 0 = destra).
## La lunghezza 430 e 190 chiudono il circuito (verificato numericamente).
const SEGMENTS: Array = [
	["S", 500.0],  # rettilineo lungo
	["A", 150.0, 90.0],  # curva larga
	["S", 110.0],
	["A", 70.0, 90.0],  # curva media
	["S", 120.0],
	["A", 45.0, -30.0],  # chicane
	["A", 45.0, 60.0],
	["A", 45.0, -30.0],
	["S", 430.0],
	["A", 40.0, 90.0],  # gomito
	["S", 190.0],  # tratto stretto (vedi NARROW_SEGMENT)
	["A", 100.0, 90.0],
]
const NARROW_SEGMENT := 10
const NARROW_MARGIN := 25.0  ## m di transizione a inizio e fine del tratto stretto

var track_info: TrackInfo

var _pts: Array[Vector3] = []  ## centerline campionata (ultimo != primo)
var _dirs: Array[Vector3] = []
var _widths: Array[float] = []


func _ready() -> void:
	_build_centerline()
	_build_curve()
	_build_road_mesh()
	_build_walls()
	_build_poles()
	_build_ground()


func get_start() -> Vector3:
	return _pts[0]


# --- Centerline -------------------------------------------------------------

func _build_centerline() -> void:
	var pos := Vector3.ZERO
	var angle := 0.0  # 0 = +X, cresce verso +Z (curva a destra)
	var dist := 0.0
	var narrow_start := 0.0
	var narrow_end := 0.0
	_pts.append(pos)
	var seg_index := 0
	for seg in SEGMENTS:
		var radius := 0.0
		var total_angle := 0.0
		var length: float = seg[1]
		if seg[0] == "A":
			radius = seg[1]
			total_angle = deg_to_rad(seg[2])
			length = absf(total_angle) * radius
		var n := maxi(1, roundi(length / sample_step))
		var step := length / n
		if seg_index == NARROW_SEGMENT:
			narrow_start = dist
			narrow_end = dist + length
		for i in n:
			var da := 0.0
			if radius > 0.0:
				da = step / radius * signf(total_angle)
			var mid := angle + da * 0.5
			pos += Vector3(cos(mid), 0.0, sin(mid)) * step
			angle += da
			dist += step
			_pts.append(pos)
		seg_index += 1
	# Il circuito è chiuso: l'ultimo punto coincide col primo (errore < 2 cm).
	_pts.remove_at(_pts.size() - 1)

	var count := _pts.size()
	var cum := 0.0
	for i in count:
		var t := _pts[(i + 1) % count] - _pts[(i - 1 + count) % count]
		t.y = 0.0
		_dirs.append(t.normalized())
		var f := 1.0
		if cum > narrow_start - NARROW_MARGIN and cum < narrow_end + NARROW_MARGIN:
			var into := minf(cum - (narrow_start - NARROW_MARGIN), (narrow_end + NARROW_MARGIN) - cum)
			f = lerpf(1.0, narrow_factor, smoothstep(0.0, NARROW_MARGIN * 2.0, into))
		_widths.append(road_width * f)
		cum += _pts[(i + 1) % count].distance_to(_pts[i])


func _build_curve() -> void:
	var curve := Curve3D.new()
	curve.bake_interval = 1.0
	for p in _pts:
		curve.add_point(p)
	curve.add_point(_pts[0])  # chiude l'anello
	var path := Path3D.new()
	path.name = "Centerline"
	path.curve = curve
	add_child(path)
	track_info = TrackInfo.new()
	track_info.name = "TrackInfo"
	track_info.curve = curve
	add_child(track_info)


# --- Geometria --------------------------------------------------------------

func _right_of(i: int) -> Vector3:
	var d := _dirs[i]
	return Vector3(-d.z, 0.0, d.x)


func _edge(i: int, side: float, extra := 0.0) -> Vector3:
	return _pts[i] + _right_of(i) * side * (_widths[i] * 0.5 + extra)


func _material() -> StandardMaterial3D:
	var m := StandardMaterial3D.new()
	m.vertex_color_use_as_albedo = true
	m.cull_mode = BaseMaterial3D.CULL_DISABLED
	return m


func _add_quad(st: SurfaceTool, a0: Vector3, b0: Vector3, a1: Vector3, b1: Vector3, color: Color) -> void:
	st.set_color(color)
	for v in [a0, b0, a1, b0, b1, a1]:
		st.add_vertex(v)


func _mesh_instance(mesh: Mesh) -> MeshInstance3D:
	var mi := MeshInstance3D.new()
	mi.mesh = mesh
	mi.material_override = _material()
	add_child(mi)
	return mi


func _build_road_mesh() -> void:
	var st := SurfaceTool.new()
	st.begin(Mesh.PRIMITIVE_TRIANGLES)
	var count := _pts.size()
	for i in count:
		var j := (i + 1) % count
		var c := Color(0.27, 0.27, 0.29) if (i / 3) % 2 == 0 else Color(0.31, 0.31, 0.33)
		_add_quad(st, _edge(i, -1.0), _edge(i, 1.0), _edge(j, -1.0), _edge(j, 1.0), c)
	st.generate_normals()
	_mesh_instance(st.commit()).name = "Road"


func _build_walls() -> void:
	var st := SurfaceTool.new()
	st.begin(Mesh.PRIMITIVE_TRIANGLES)
	var count := _pts.size()
	var up := Vector3.UP * wall_height
	for i in count:
		var j := (i + 1) % count
		var c := Color(0.85, 0.15, 0.12) if (i / 3) % 2 == 0 else Color(0.92, 0.92, 0.92)
		for side in [-1.0, 1.0]:
			var a := _edge(i, side)
			var b := _edge(j, side)
			var ao := _edge(i, side, wall_thickness)
			var bo := _edge(j, side, wall_thickness)
			_add_quad(st, a, b, a + up, b + up, c)  # faccia interna
			_add_quad(st, a + up, b + up, ao + up, bo + up, c)  # sommità
	st.generate_normals()
	_mesh_instance(st.commit()).name = "WallsVisual"

	# Collisione: una scatola ogni `collider_every` campioni, leggermente sovrapposte.
	var body := StaticBody3D.new()
	body.name = "Walls"
	add_child(body)
	var box := BoxShape3D.new()
	var seg_len := _pts[0].distance_to(_pts[collider_every % count]) * 1.15
	box.size = Vector3(wall_thickness, wall_height, seg_len)
	var i := 0
	while i < count:
		var j := (i + collider_every) % count
		for side in [-1.0, 1.0]:
			var p0 := _edge(i, side, wall_thickness * 0.5)
			var p1 := _edge(j, side, wall_thickness * 0.5)
			var mid := (p0 + p1) * 0.5
			var dir := p1 - p0
			dir.y = 0.0
			var cs := CollisionShape3D.new()
			cs.shape = box
			cs.transform = Transform3D(Basis(Vector3.UP, atan2(-dir.x, -dir.z)), mid + Vector3.UP * wall_height * 0.5)
			body.add_child(cs)
		i += collider_every


func _build_poles() -> void:
	var mesh := BoxMesh.new()
	mesh.size = Vector3(0.4, 5.0, 0.4)
	var mat := StandardMaterial3D.new()
	mat.albedo_color = Color(0.95, 0.8, 0.1)
	mesh.material = mat
	var xforms: Array[Transform3D] = []
	var count := _pts.size()
	var every := maxi(1, roundi(pole_spacing / sample_step))
	var i := 0
	while i < count:
		for side in [-1.0, 1.0]:
			xforms.append(Transform3D(Basis.IDENTITY, _edge(i, side, wall_thickness + 1.5) + Vector3.UP * 2.5))
		i += every
	var mm := MultiMesh.new()
	mm.transform_format = MultiMesh.TRANSFORM_3D
	mm.mesh = mesh
	mm.instance_count = xforms.size()
	for k in xforms.size():
		mm.set_instance_transform(k, xforms[k])
	var mmi := MultiMeshInstance3D.new()
	mmi.name = "Poles"
	mmi.multimesh = mm
	add_child(mmi)


func _build_ground() -> void:
	var mesh := PlaneMesh.new()
	mesh.size = Vector2(4000.0, 4000.0)
	var mat := StandardMaterial3D.new()
	mat.albedo_color = Color(0.22, 0.32, 0.2)
	mesh.material = mat
	var mi := MeshInstance3D.new()
	mi.name = "Ground"
	mi.mesh = mesh
	mi.position = Vector3(300.0, -0.3, 170.0)
	add_child(mi)
