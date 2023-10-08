extends Node

export var test_url: String
export var prod_url: String

onready var base_url = "https://%s" % prod_url
onready var base_ws_url = "wss://%s" % prod_url

func _ready():
	if OS.has_feature("editor"):
		base_url = "http://%s" % test_url
		base_ws_url = "ws://%s" % test_url
