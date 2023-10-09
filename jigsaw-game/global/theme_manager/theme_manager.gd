# This script is attached to ThemeManager.tscn scene that is
# loaded using Godot AutoLoad feature

extends Node

# You can set the default colors if you open ThemeManager scene
# These colors will be used in the editor when you test the app  
export var bg_color: Color
export var text_color: Color
export var hint_color: Color
export var link_color: Color
export var button_color: Color
export var button_text_color: Color
export var secondary_bg_color: Color

var transparent := Color(0, 0, 0, 0)

onready var theme: Theme = preload("res://Theme.tres")

var on_theme_change_callback

func _ready():
	# Check if the game is running on the Web platform
	# In our case it's only when exported and used as a Telegram Mini App
	if OS.has_feature('JavaScript'):
		load_telegram_colors()
		# This function will make it so that when the Telegram theme is changed
		# we change our theme as well 
		register_theme_change_handler()
	update_theme()
	
# Get color from Telegram themeParams
func eval_color(color_name: String) -> Color:
	var color: String = JavaScript.eval("window.Telegram.WebApp.themeParams.%s" % color_name)
	return Color(color)


func register_theme_change_handler():
	# We need to keep a reference to the callback in a variable
	# Otherwise the reference will be freed
	# Think of it like we create a JavaScript function that we can call
	# to execute a Godot function '_on_theme_change'
	on_theme_change_callback = JavaScript.create_callback(self, "_on_theme_change")
	# Get an interface to a JavaScript 'window' object
	var window = JavaScript.get_interface("window")
	# With this interface we can basically write JavaScript in Godot and register
	# our event handler
	window.Telegram.WebApp.onEvent("themeChanged", on_theme_change_callback)


# Telegram will call this function from JavaScript when the
# theme is changed
func _on_theme_change(args):
	load_telegram_colors()
	update_theme()


func load_telegram_colors():
	# Assign colors from Telegram to the local variables
	bg_color = eval_color("bg_color")
	text_color = eval_color("text_color")
	hint_color = eval_color("hint_color")
	link_color = eval_color("link_color")
	button_color = eval_color("button_color")
	button_text_color = eval_color("button_text_color")
	secondary_bg_color = eval_color("secondary_bg_color")
	

# When we change properties on the Theme resource it's updated automatically
# You can change most of the styles inside the theme editor and then only change
# colors here
func update_theme():
	theme.set_color("font_color", "Label", text_color)
	theme.get_stylebox("scroll", "HScrollBar").bg_color = transparent
	theme.get_stylebox("grabber", "HScrollBar").bg_color = hint_color	
	VisualServer.set_default_clear_color(bg_color)
