extends Control

class_name PuzzleGridCell

var index: Vector2Int

var piece: PuzzlePiece = null

onready var tile_container := $TileContainer
onready var tween := $Tween

func _ready():
	Events.connect("tile_size_change", self, "_on_tile_size_change")
	$Label.text = index.as_string()


func can_drop_data(position: Vector2, data: Object) -> bool:
	return data is PuzzlePiece and not piece


func drop_data(position: Vector2, data: PuzzlePiece) -> void:
	
	if not is_instance_valid(data):
		return
	
	data.get_parent().remove_child(data)
	tile_container.add_child(data)
	data.rect_position = position - rect_size * 0.5
	
	Events.emit_signal("request_place", data.tile_uuid, index)


func _on_tile_size_change(tile_size: Vector2Int) -> void:
	rect_min_size = tile_size.as_vec2float()
