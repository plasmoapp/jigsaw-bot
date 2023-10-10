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
	
	var dpi = OS.get_screen_dpi()
	
	var ui_scale
	
	if dpi < 240:
		ui_scale = 1
	elif dpi < 480:
		ui_scale = 2
	else:
		ui_scale = 3
		
	get_tree().set_screen_stretch(
		SceneTree.STRETCH_MODE_DISABLED,
		SceneTree.STRETCH_ASPECT_EXPAND,
		Vector2.ZERO,
		ui_scale
	)
