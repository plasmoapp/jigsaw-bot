extends Control

class_name Game

export var puzzle_piece_scene: PackedScene
export var puzzle_grid_cell_scene: PackedScene

var meta: PuzzleMeta

onready var preview_http_image := $Puzzle/PreviewHttpImage
onready var puzzle_control := $Puzzle
onready var tray := $Tray
onready var margin_container := $Tray/MarginContainer
onready var hbox_container := $Tray/MarginContainer/HBoxContainer
onready var grid_container := $Puzzle/GridContainer

var cell_map: Dictionary = {}

func _ready():
	connect("resized", self, "resize")

func load_puzzle(new_meta: PuzzleMeta, initial_state: Dictionary):
	meta = new_meta

	var url = "%s/assets/%s/source.webp" % [Config.base_url, meta.puzzle_uuid]
	preview_http_image.url = url
	preview_http_image.load_image()
	
	grid_container.columns = int(meta.dimensions_tiles.x)
	
	for y in range(0, meta.dimensions_tiles.y):
		for x in range(0, meta.dimensions_tiles.x):
			var index := Vector2Int.new(x, y)
			var instance = puzzle_grid_cell_scene.instance()
			instance.index = index
			# instance.resize_tile_size(meta.tile_size_px)
			grid_container.add_child(instance)
			cell_map[index.as_string()] = instance
	
	for tile_uuid in initial_state:
		var instance = puzzle_piece_scene.instance() as PuzzlePiece
		instance.puzzle_uuid = meta.puzzle_uuid
		instance.tile_uuid = tile_uuid
		var cell = initial_state[tile_uuid]
		if not cell:
			hbox_container.add_child(instance)
		else:
			var index = Vector2Int.new(cell["x"], cell["y"])
			instance.is_draggable = false
			cell_map[index.as_string()].tile_container.add_child(instance)
	
	resize()
	
	
func place_tile(tile_uuid: String, index: Vector2Int):
	var cell = cell_map[index.as_string()] as PuzzleGridCell
	
	if cell.piece:
		cell.piece.get_parent().remove_child(cell.piece)
		hbox_container.add_child(cell.piece)
	
	var pieces = get_tree().get_nodes_in_group("puzzle_piece")
	
	for piece in pieces:		
		if piece.tile_uuid != tile_uuid:
			continue
			
		if piece.is_in_group("preview"):
			piece.queue_free()
			force_drag(null, null)
			continue
		
		piece.is_draggable = false
		
		var start_position: Vector2 = piece.rect_global_position - cell.rect_global_position
		
		cell.tween.stop_all()
		cell.tween.interpolate_property(
			piece,
			"rect_position",
			piece.rect_global_position - cell.rect_global_position,
			Vector2.ZERO,
			0.1 + start_position.distance_to(Vector2.ZERO) * 0.0001
		)
		cell.tween.start()
		
		piece.get_parent().remove_child(piece)
		cell.add_child(piece)

func resize():
	
	if not meta:
		return
	
	var tile_size = get_physical_tile_size()
	
	puzzle_control.rect_size	= meta.dimensions_tiles.as_vec2float() * tile_size.as_vec2float()
	puzzle_control.rect_position.x = rect_size.x / 2 - puzzle_control.rect_size.x / 2
	puzzle_control.rect_position.y = (rect_size.y - tile_size.y * 2) / 2 - puzzle_control.rect_size.y / 2
	
	tray.rect_position.y = rect_size.y - tile_size.y * 2
	tray.rect_size.y = tile_size.y * 2
	tray.rect_size.x = rect_size.x

	var margin = tile_size.x / 2
	
	margin_container.add_constant_override("margin_top", margin)
	margin_container.add_constant_override("margin_left", margin)
	margin_container.add_constant_override("margin_bottom", margin)
	margin_container.add_constant_override("margin_right", margin)
	
	hbox_container.add_constant_override("separation", margin * 1)
	
	Events.emit_signal("tile_size_change", tile_size)
	

func get_physical_tile_size() -> Vector2Int:
	
	var dimensions_plus_tray_tiles = Vector2(meta.dimensions_tiles.x, meta.dimensions_tiles.y + 2)

	var width_scale = rect_size.x / dimensions_plus_tray_tiles.x
	var height_scale = rect_size.y / dimensions_plus_tray_tiles.y

	var min_scale = min(width_scale, height_scale)
	
	return Vector2Int.new(min_scale, min_scale)
