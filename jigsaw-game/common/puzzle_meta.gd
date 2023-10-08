extends Object

class_name PuzzleMeta

var puzzle_uuid: String
var dimensions_px: Vector2Int
var tile_size_px: Vector2Int
var dimensions_tiles: Vector2Int

func _init(puzzle_uuid: String, image_dimensions_px: Vector2Int, tile_size_px: int):
	self.puzzle_uuid = puzzle_uuid
	self.dimensions_px = image_dimensions_px
	self.tile_size_px = Vector2Int.new(tile_size_px, tile_size_px)
	self.dimensions_tiles = image_dimensions_px.div(self.tile_size_px)
