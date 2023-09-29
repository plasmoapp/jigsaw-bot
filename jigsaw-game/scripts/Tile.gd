extends Control

class_name JigsawTile

export var puzzle_uuid: String
export var tile_uuid: String
export var draggable: bool = true

export var processing_color: Color
export var wrong_color: Color
export var correct_color: Color

onready var color_rect := $ColorRect

enum STATE { DEFAULT, PROCESSING, CORRECT, WRONG }

var state: int = STATE.DEFAULT

onready var global: GlobalState = get_node("/root/Global")
onready var image_url := "%s/assets/%s/%s.jpeg" % [global.base_url, puzzle_uuid, tile_uuid]
onready var place_url := "%s/api/puzzle/%s/tile/%s/place" % [global.base_url, puzzle_uuid, tile_uuid]
onready var http_image := $HttpImage
onready var http_request := $HTTPRequest

#var texture: ImageTexture = null

func _ready():
	http_request.connect("request_completed", self, "_on_request_completed")
	http_image.url = image_url
	http_image.load()
	# resize(global.tile_size)
	
func get_preview() -> Control:
	var preview = duplicate()
	var control = Control.new()
	control.add_child(preview)
	preview.rect_position = -0.5 * preview.rect_size
	return control
	
func process_index(index: Vector2Int):
	var body = '{"x":%s,"y":%s}' % [index.x, index.y]
	print(body)
	change_state(STATE.PROCESSING)
	var error = http_request.request(place_url, ["Content-Type: application/json"], true, HTTPClient.METHOD_POST, body)
	if error != OK:
		print("Error while processing place request")
		
func _on_request_completed(result: int, response_code: int, headers, body):
	if result != 0:
		print("Failed to make an HTTP request" )
		return
	if response_code < 200 or response_code > 299:
		print("Bad response. Status code: %s" % response_code)
		return
		
	var json_result = JSON.parse(body.get_string_from_utf8())
	if json_result.error:
		print("Error while parsing JSON")
		return
		
	if json_result.result:
		change_state(STATE.CORRECT)
	else:
		change_state(STATE.WRONG)
	
func change_state(new_state: int):
	var color
	match new_state:
		STATE.DEFAULT:
			color_rect.hide()
		STATE.PROCESSING:
			color = processing_color
		STATE.WRONG:
			color = wrong_color
		STATE.CORRECT:
#			color = correct_color
			color_rect.hide()
		_:
			push_error("Error when changing tile state: Invalid State")
			return
	state = new_state		
	if color:
		color_rect.show()
		color_rect.color = color
	
func get_drag_data(position) -> Object:
	if not draggable or not (state == STATE.DEFAULT or state == STATE.WRONG):
		return null
	change_state(STATE.DEFAULT)
	set_drag_preview(get_preview())
	var data = duplicate()
	queue_free()
	return data
	
func resize(size: Vector2):
	rect_min_size = size
#	rect_size = size
	
func _on_tile_size_change(size: Vector2):
	rect_min_size = size
#	rect_size = size
