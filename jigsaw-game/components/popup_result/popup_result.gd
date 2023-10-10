tool

extends PanelContainer

class_name PopupResult

export var score_entry_scene: PackedScene

onready var new_puzzle_button = $"%NewPuzzleButton"
onready var close_button = $"%CloseButton"
onready var score_entry_container = $"%ScoreEntryContainer"

onready var animation_player := $AnimationPlayer


func _ready():
	
	hide()
	
#	var test_scores = {"0": 10, "1": 1, "2": 5}
#
#	var test_users = {
#		"0": {"name": "kpids"},
#		"1": {"name": "apehum"},
#		"2": {"name": "loh"},
#	}
#
#	set_score(test_users, test_scores)
	
	close_button.connect("pressed", self, "_on_close_pressed")
	new_puzzle_button.connect("pressed", self, "_on_new_puzzle_pressed")
	connect("resized", self, "resize")
	resize()
	
	
func set_score(users: Dictionary, scores: Dictionary):
	
	var dict_array = []
	for user_id in scores:
		if scores[user_id] == 0:
			continue
		var entry = {"user_id": user_id, "score": scores[user_id]}
		dict_array.append(entry)
		
	dict_array.sort_custom(self, "_compare_score")
	
	for child in score_entry_container.get_children():
		child.queue_free()
	
	for user_score in dict_array:
		var instance = score_entry_scene.instance() as ScoreEntry
		instance.score_label = str(user_score["score"])
		instance.name_label = users[user_score["user_id"]]["name"]
		score_entry_container.add_child(instance)
	
	animation_player.play("Show")
	

func _compare_score(a, b):
	return a["score"] > b["score"]

	
func _on_close_pressed():
	animation_player.play("Hide")	
	
func _on_new_puzzle_pressed():
	
	if not OS.has_feature("JavaScript"):
		return
		
	JavaScript.eval("window.Telegram.WebApp.openTelegramLink('https://t.me/%s')" % Config.bot_name)
	
	
func resize():
	rect_pivot_offset = rect_size / 2
	
#func 


# Called every frame. 'delta' is the elapsed time since the previous frame.
#func _process(delta):
#	pass
