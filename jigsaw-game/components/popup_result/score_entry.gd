extends Control

class_name ScoreEntry

var name_label: String
var score_label: String

func _ready() -> void:
	if name_label:
		$Name.text = name_label
	if score_label: 
		$Score.text = score_label
