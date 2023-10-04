extends Container

class_name GameContainer

var image_size: Vector2Int = null
var tile_size: Vector2Int = null

var tile_dimensions: Vector2Int = null

func _ready():
	pass # Replace with function body.

func set_jigsaw_meta(image_size: Vector2Int, tile_size: Vector2Int):
	self.image_size = image_size
	self.tile_size = tile_size
	tile_dimensions = image_size.div(tile_size)
	print(image_size.to_string())
	print(tile_size.to_string())
	print(tile_dimensions)
