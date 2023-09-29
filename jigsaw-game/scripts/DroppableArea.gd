extends Control


# Declare member variables here. Examples:
# var a = 2
# var b = "text"


# Called when the node enters the scene tree for the first time.
func _ready():
	print()
	pass # Replace with function body.
	
func can_drop_data(position: Vector2, data: Object) -> bool:
	return data is JigsawTile
	
func drop_data(position: Vector2, data: JigsawTile) -> void:
	data.rect_position = rect_global_position + position - data.rect_size / 2
	add_child(data)


# Called every frame. 'delta' is the elapsed time since the previous frame.
#func _process(delta):
#	pass
