extends Node

signal color_update

export var bg_color: Color
export var text_color: Color
export var hint_color: Color
export var link_color: Color
export var button_color: Color
export var button_text_color: Color
export var secondary_bg_color: Color

func _ready():
	connect("color_update", self, "_on_color_update")
	if OS.has_feature('JavaScript'):
		load_telegram_colors()

func load_telegram_colors():
	bg_color = eval_color("bg_color")
	text_color = eval_color("text_color")
	hint_color = eval_color("hint_color")
	link_color = eval_color("link_color")
	button_color = eval_color("button_color")
	button_text_color = eval_color("button_text_color")
	secondary_bg_color = eval_color("secondary_bg_color")
	emit_signal("color_update")
	
func _on_color_update():
	VisualServer.set_default_clear_color(bg_color)
	
func eval_color(color_string: String) -> Color:
	var color: String = JavaScript.eval("window.Telegram.WebApp.themeParams.%s" % color_string)
	return Color(color)
