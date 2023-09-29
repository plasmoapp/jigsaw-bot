extends Node

class_name GlobalState

signal tile_size_change(tile_size)

var test_url = "localhost:3000"
var prod_url = "d95b-176-36-229-75.ngrok-free.app"

var base_url = "https://%s" % prod_url
var base_ws_url = "wss://%s" % prod_url

var tile_size

func _ready():
	if OS.has_feature("editor"):
		base_url = "http://%s" % test_url
		base_ws_url = "ws://%s" % test_url
