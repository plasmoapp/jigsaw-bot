# This Node is used to tie everything together:
# WebSocketManager, Game, Label and PopupResult

extends Node

export var puzzle_uuid: String

export(NodePath) onready var web_socket_manager = get_node(web_socket_manager) as WebSocketManager

export(NodePath) onready var game = get_node(game) as Game

export(NodePath) onready var status_label = get_node(status_label) as FadingLabel

export(NodePath) onready var popup_result = get_node(popup_result) as PopupResult

var users: Dictionary = {}

var scores: Dictionary = {}


func _ready() -> void:
	
	web_socket_manager.connect("initial", self, "_on_initial")
	web_socket_manager.connect("placed", self, "_on_placed")
	web_socket_manager.connect("join", self, "_on_join")
	web_socket_manager.connect("leave", self, "_on_leave")
	
	Events.connect("request_place", self, "_on_request_place")
	
	# Get puzzle_uuid from Telegram start_param
	# It's okay to use initDataUnsafe because we autorize user on the WebSocket
	# later anyway 
	if OS.has_feature('JavaScript'):
		puzzle_uuid = JavaScript.eval("window.Telegram.WebApp.initDataUnsafe.start_param", true)
		print("Got puzzle uuid from Telegram: %s" % puzzle_uuid)
	
	web_socket_manager.connect_with_puzzle_uuid(puzzle_uuid)


func _on_initial(data) -> void:
	var meta = PuzzleMeta.new(
		puzzle_uuid,
		Vector2Int.new(
			data["meta"]["image_dimensions_px"][0],
			data["meta"]["image_dimensions_px"][1]
		),
		data["meta"]["tile_size_px"]
	)
	
	users = data["users"]
	scores = data["scores"]
	
	game.load_puzzle(meta, data["state"])

	check_if_game_ended()


func check_if_game_ended():
	var not_placed_count = get_tree().get_nodes_in_group("not_placed").size()
	
	if not_placed_count != 0:
		return
		
	popup_result.set_score(users, scores)
	

func _on_placed(message) -> void:
	game.place_tile(message["tile_uuid"], Vector2Int.new(message["index"]["x"], message["index"]["y"]))
	
	var user_id = str(message["user"])
	var user_data = users[user_id]
	
	scores[user_id] += 1
	
	status_label.update_text("%s placed a piece" % user_data["name"])
	
	check_if_game_ended()
	
	
func _on_join(message) -> void:
	var user_id = str(message["user"][0])
	var user_data = message["user"][1]
	
	users[user_id] = user_data
	
	status_label.update_text("%s joined the game" % user_data["name"])
	
	
func	 _on_leave(message) -> void:
	var user_id = str(message["user"])
	var user_data = users[user_id]
	
	status_label.update_text("%s left the game" % user_data["name"])
	
	
func _on_request_place(tile_uuid: String, index: Vector2Int) -> void:
	var message = '{"type": "place", "tile_uuid": "%s", "index": {"x": %s, "y": %s}}' % [tile_uuid, index.x, index.y]
	var error = web_socket_manager.send_message(message)
	if error != OK:
		push_error("Error when sending a place request")
