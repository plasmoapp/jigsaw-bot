extends HBoxContainer


# Declare member variables here. Examples:
# var a = 2
# var b = "text"


# Called when the node enters the scene tree for the first time.
func _ready():
	pass # Replace with function body.

func can_drop_data(position, data):
	return true
	
func drop_data(position, data):
	add_child(data)

# Called every frame. 'delta' is the elapsed time since the previous frame.
#func _process(delta):
#	pass
