extends Object

class_name Vector2Int

var x: int = 0
var y: int = 0

func _init(x: int, y: int):
	self.x = x
	self.y = y
	
func as_string() -> String:
	return "(%s, %s)" % [x, y]

func plus(vec: Vector2Int) -> Vector2Int:
	return get_script().new(x + vec.x, y + vec.y)

func minus(vec: Vector2Int) -> Vector2Int:
	return get_script().new(x - vec.x, y - vec.y)
	
func div(vec: Vector2Int) -> Vector2Int:
	return get_script().new(x / vec.x, y / vec.y)
	
func mul(vec: Vector2Int) -> Vector2Int:
	return get_script().new(x * vec.x, y * vec.y)

func as_vec2float() -> Vector2:
	return Vector2(x, y)
