extends Node

class_name GameConfig

export var test_url: String = "localhost:3000"
export var prod_url: String = "5d8b-176-36-229-75.ngrok-free.app"

var base_url = "https://%s" % prod_url
var base_ws_url = "wss://%s" % prod_url

var tile_size

func _ready():
	if OS.has_feature("editor"):
		base_url = "http://%s" % test_url
		base_ws_url = "ws://%s" % test_url
