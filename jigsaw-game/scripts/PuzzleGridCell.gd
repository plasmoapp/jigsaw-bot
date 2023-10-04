extends Control

class_name PuzzleGridCell

var index: Vector2Int

var tile: JigsawTile = null

onready var config: GameConfig = get_node("/root/Config")

onready var tile_container = $TileContainer

func _ready():
	$Label.text = index.as_string()

func can_drop_data(position: Vector2, data: Object) -> bool:
	return data is JigsawTile and tile_container.get_child_count() == 0
	
func set_tile(data: JigsawTile, correct: bool = false):
	data.rect_position = Vector2.ZERO
	tile_container.add_child(data)
	tile = data
	if correct:
		tile.change_state(JigsawTile.STATE.CORRECT)
	else:
		tile.process_index(index)
	
func drop_data(_position: Vector2, data: JigsawTile) -> void:
	set_tile(data, false)
	
func resize_tile_size(tile_size: Vector2) -> void:
	rect_min_size = tile_size
