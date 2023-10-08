extends Label

class_name FadingLabel

onready var timer = $Timer
onready var tween = $Tween

export var modulate_on = Color(1.0, 1.0, 1.0, 1.0)
export var modulate_off = Color(1.0, 1.0, 1.0, 0.0)


func _ready():
	modulate = modulate_off
	timer.connect("timeout", self, "_on_timeout")


func update_text(update_text: String) -> void:
	text = update_text
	timer.start()
	tween.stop_all()
	tween.interpolate_property(self, "modulate",
		modulate_off, modulate_on, 0.1,
		Tween.TRANS_LINEAR, Tween.EASE_IN_OUT
	)
	tween.start()


func _on_timeout() -> void:
	tween.stop_all()
	tween.interpolate_property(self, "modulate",
		modulate_on, modulate_off, 0.2,
		Tween.TRANS_LINEAR, Tween.EASE_IN_OUT
	)
	tween.start()


