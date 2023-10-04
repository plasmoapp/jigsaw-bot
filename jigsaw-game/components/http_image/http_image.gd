tool
extends TextureRect
class_name HttpImage

signal image_loaded(image_texture)

export var url: String
export var autoload: bool = false

func _ready():
	$HTTPRequest.connect("request_completed", self, "_on_request_completed")
	if autoload:
		load_image()
	
func load_image(): 
	var http_error = $HTTPRequest.request(url)
	if http_error != OK:
		print("An error occurred in the HTTP request.")
		print(http_error)

func _on_request_completed(result, response_code, headers: PoolStringArray, body):
	
	if result != 0:
		print("Failed to make a HTTP request" )
		return
		
	if response_code < 200 or response_code > 299:
		print("Bad response. Status code: %s" % response_code)
		return
		
	var content_type: String
		
	for header in headers:
		header = header.to_lower()
		if header.begins_with("content-type:"):
			content_type = header.split(":")[1].split(",")[0].strip_edges()
			break
	
	if not content_type:
		print("No content type header")
		return 
		
	content_type = content_type.split(",")[0]
	
	var image = Image.new()
	
	var image_error
	
	match content_type:
		"image/jpeg":
			image_error = image.load_jpg_from_buffer(body)
		"image/png":
			image_error = image.load_png_from_buffer(body)
		"image/webp":
			image_error = image.load_webp_from_buffer(body)
		_:
			print("Unknown content_type: %s" % content_type)
			return
			
	if image_error != OK:
		print("An error occurred while trying to display the image.")

	var image_texture = ImageTexture.new()
	image_texture.create_from_image(image)
	texture = image_texture
	emit_signal("image_loaded", image_texture)
