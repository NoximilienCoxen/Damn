class_name TrackInfo
extends Node
## Query sulla pista a partire dalla centerline (Curve3D chiusa: ultimo punto = primo).

var curve: Curve3D


func get_length() -> float:
	return curve.get_baked_length()


func closest_offset(p: Vector3) -> float:
	return curve.get_closest_offset(p)


func point_at(offset: float) -> Vector3:
	return curve.sample_baked(fposmod(offset, get_length()))


func tangent_at(offset: float) -> Vector3:
	var a := point_at(offset - 1.0)
	var b := point_at(offset + 1.0)
	var t := b - a
	t.y = 0.0
	return t.normalized()
