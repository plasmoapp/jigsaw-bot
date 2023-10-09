extends Control

class_name PuzzlePiece

export var is_draggable: bool = true

export var puzzle_uuid: String
export var tile_uuid: String

func _ready() -> void:
	Events.connect("tile_size_change", self, "_on_tile_size_change")
	$HttpImage.url = "%s/assets/%s/%s.webp" % [Config.base_url, puzzle_uuid, tile_uuid] 
	$HttpImage.load_image()
	
func _on_tile_size_change(new_size: Vector2Int) -> void:
	rect_min_size = new_size.as_vec2float()

func get_preview(position: Vector2) -> Control:
	var preview := Control.new()
	var duplicate := duplicate()
	preview.add_child(duplicate)
	preview.add_to_group("preview")
	duplicate.rect_position = -0.5 * duplicate.rect_size
	return preview

func get_drag_data(position: Vector2):
	if not is_draggable:
		return null
	
	set_drag_preview(get_preview(position))
	return self

