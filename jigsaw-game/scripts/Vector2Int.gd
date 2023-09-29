extends Object

class_name Vector2Int

var x: int = 0
var y: int = 0

func _init(x: int, y: int):
	self.x = x
	self.y = y
	
func as_string() -> String:
	return "(%s, %s)" % [x, y]
