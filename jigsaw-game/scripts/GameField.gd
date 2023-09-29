extends Container

signal grid_initialized(cell_map)

export var puzzle_uuid: String

export var original_tile_size: Vector2 = Vector2(80, 80)

onready var global = get_node("/root/Global")

onready var http_image = get_node("%HttpImage")

onready var cell_scene := load("res://scenes/PuzzleGridCell.tscn")

onready var grid_container = $Control/GridContainer

var cell_map: Dictionary = {}

func _ready():
	http_image.connect("image_loaded", self, "_on_image_loaded")
	http_image.url = "%s/assets/%s/source.jpeg" % [global.base_url, puzzle_uuid]
	http_image.load()
	connect("resized", self, "_on_image_resized")
	
func _on_image_resized():
	resize(http_image.texture.get_size())

func resize(image_size: Vector2):
	var tile_dimensions = image_size / original_tile_size
	
	var container_size = rect_size
	
	var width_scale = container_size.x / image_size.x
	var height_scale = container_size.y / image_size.y

	var min_scale = min(width_scale, height_scale)

	var rect_size = Vector2(image_size.x * min_scale, image_size.y * min_scale)

	var tile_size = (rect_size / tile_dimensions).floor()
	
	rect_size = tile_size * tile_dimensions
	
	$Control.margin_left = -rect_size.x / 2
	$Control.margin_right = rect_size.x / 2
	
	$Control.margin_top = -rect_size.y / 2
	$Control.margin_bottom = rect_size.y / 2
	
	global.emit_signal("tile_size_change", tile_size)


func _on_image_loaded(image: ImageTexture): 
	resize(image.get_size())
	var tile_dimensions = image.get_size() / original_tile_size
	var tile_size = (rect_size / tile_dimensions).floor()
	grid_container.columns = int(tile_dimensions.x)
	for y in range(0, tile_dimensions.y):
		for x in range(0, tile_dimensions.x):
			var index := Vector2Int.new(x, y)
			var instance = cell_scene.instance()
			instance.index = index
			instance.resize_tile_size(tile_size)
			global.connect("tile_size_change", instance, "resize_tile_size")
			grid_container.add_child(instance)
			cell_map[index.as_string()] = instance
	emit_signal("grid_initialized", cell_map)
