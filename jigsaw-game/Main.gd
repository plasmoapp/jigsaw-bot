extends Node

export var puzzle_uuid: String

onready var tile_scene := load("res://scenes/Tile.tscn")
onready var field_scene := load("res://scenes/GameField.tscn")

onready var droppable_area := get_node("%DroppableArea")
onready var tile_container := get_node("%TilesContainer")

onready var global: GlobalState = get_node("/root/Global")

var client = WebSocketClient.new()

var initial_tiles

var field_instance

func _process(delta):
	client.poll()

func _ready():

	client.connect("connection_closed", self, "_closed")
	client.connect("connection_error", self, "_closed")
	client.connect("connection_established", self, "_connected")
	client.connect("data_received", self, "_on_data")
	
	if OS.has_feature('JavaScript'):
		puzzle_uuid = JavaScript.eval("window.Telegram.WebApp.initDataUnsafe.start_param", true)
		# puzzle_uuid = JavaScript.eval("(new URLSearchParams(window.location.search)).get('puzzle')")
		print("Got puzzle uuid: %s" % puzzle_uuid)
	
	var url = "%s/api/puzzle/%s/websocket" % [global.base_ws_url, puzzle_uuid]
	
	client.set_verify_ssl_enabled(true)
	
	var protocols := []
	
	if OS.has_feature("editor"):
		protocols.append("jigsaw-not-secure")
	else:
		protocols.append("jigsaw-telegram-auth")
	
	var websocket_error = client.connect_to_url(url, protocols)
	
	if websocket_error != OK:
		push_error("Unable to connect to websocket")
		
		
func _closed(was_clean = false):
	push_error("Closed, clean: %s" % was_clean)

func _connected(proto = ""):
	print("Connected with protocol: ", proto)
	if proto == "jigsaw-telegram-auth":
		var data = JavaScript.eval("window.Telegram.WebApp.initData", true)
		var request = '{"type":"telegram_auth","data_check_string":"%s"}' % data
		var error = client.get_peer(1).put_packet(request.to_utf8())
		
		if error != OK:
			push_error("Error when sending a request: %s" % String(error))

func _on_data():
	
	var result = JSON.parse(client.get_peer(1).get_packet().get_string_from_utf8())
	
	if result.error:
		print("Error while parsing JSON")
	
	var data = result.result
	
	var type = data["type"]
	
	if type == "initial":
		field_instance = field_scene.instance()
		field_instance.puzzle_uuid = puzzle_uuid
		droppable_area.add_child(field_instance)
		initial_tiles = data["data"]
		field_instance.connect("grid_initialized", self, "_on_grid_initalized")
	elif type == "placed":
		var tiles = get_tree().get_nodes_in_group("tiles")
		for tile in tiles:
			if tile.tile_uuid != data.tile_uuid:
				continue
			if tile.state == JigsawTile.STATE.CORRECT or tile.state == JigsawTile.STATE.PROCESSING:
				break
			var duplicate = tile.duplicate()
			var index_vec = Vector2Int.new(data["index"]["x"], data["index"]["y"])
			var cell = field_instance.cell_map[index_vec.as_string()]
			global.connect("tile_size_change", duplicate, "_on_tile_size_change")
			cell.set_tile(duplicate, true)
			tile.queue_free()
			break


func _on_grid_initalized(cell_map: Dictionary):
	for tile_uuid in initial_tiles:
		var tile_instance = tile_scene.instance()
		tile_instance.puzzle_uuid = puzzle_uuid
		tile_instance.tile_uuid = tile_uuid
		var index = initial_tiles[tile_uuid]
		if index:
			var index_vec = Vector2Int.new(index.x, index.y)
			var cell = cell_map[index_vec.as_string()]
			global.connect("tile_size_change", tile_instance, "_on_tile_size_change")
			cell.set_tile(tile_instance, true)
		else:
			global.connect("tile_size_change", tile_instance, "_on_tile_size_change")
			tile_container.add_child(tile_instance)
