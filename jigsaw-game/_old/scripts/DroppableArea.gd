extends Control
	
func can_drop_data(position: Vector2, data: Object) -> bool:
	return data is JigsawTile
	
func drop_data(position: Vector2, data: JigsawTile) -> void:
	data.rect_position = rect_global_position + position - data.rect_size / 2
	add_child(data)
