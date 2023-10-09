extends ScrollContainer


onready var hbox_container = $MarginContainer/HBoxContainer


func can_drop_data(position: Vector2, data: Object) -> bool:
	return data is PuzzlePiece



func drop_data(position: Vector2, data: PuzzlePiece) -> void:
	
	if not is_instance_valid(data):
		return
	
	data.get_parent().remove_child(data)
	data.rect_position = Vector2.ZERO
	data.margin_left = -32
	data.margin_right = -32
	
	for child in hbox_container.get_children():
		if child.rect_position.x > position.x:
			hbox_container.add_child(data)
			hbox_container.move_child(data, child.get_index() - 1)
			return
			
	hbox_container.add_child(data)
