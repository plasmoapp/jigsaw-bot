extends Node

# Used in the editor
export var test_url: String

onready var base_url = "http://%s" % test_url
onready var base_ws_url = "ws://%s" % test_url

func _ready():
	if OS.has_feature("JavaScript"):
		var host = JavaScript.eval("window.location.host")
		base_url = "https://%s" % host
		base_ws_url = "wss://%s" % host
