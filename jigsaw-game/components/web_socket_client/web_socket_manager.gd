extends Node

class_name WebSocketManager

signal initial(message)
signal placed(message)
signal join(message)
signal leave(message)
signal chat(message)

var client := WebSocketClient.new()

func _ready() -> void:
	client.set_verify_ssl_enabled(true)
	client.connect("connection_closed", self, "_closed")
	client.connect("connection_error", self, "_closed")
	client.connect("connection_established", self, "_connected")
	client.connect("data_received", self, "_on_data")

	
func _process(_delta: float) -> void:
	client.poll()


func connect_with_puzzle_uuid(puzzle_uuid: String) -> void:
	
	var url = "%s/api/puzzle/%s/websocket" % [Config.base_ws_url, puzzle_uuid]

	var protocols := []
	
	# In the debug mode our server supports authorization
	# protocol that doesn't require user to autorize with Telegram
	# This is only used for testing purposes in the editor
	# In release mode this protocol is not available 
	if OS.has_feature("editor"):
		protocols.append("jigsaw-not-secure")
	else:
		protocols.append("jigsaw-telegram-auth")
	
	var websocket_error = client.connect_to_url(url, protocols)
	
	if websocket_error != OK:
		push_error("Unable to connect to websocket")


func _connected(proto = ""):
	if proto == "jigsaw-telegram-auth":
		# With this protocol the server will expect the client to send
		# the 'initData' from Telegram as the first packet
		# That's the only data we need to authorize the user
		# More info: https://core.telegram.org/bots/webapps#validating-data-received-via-the-mini-app
		var data = JavaScript.eval("window.Telegram.WebApp.initData", true)
		var request = '{"type":"telegram_auth","init_data":"%s"}' % data
		var error = client.get_peer(1).put_packet(request.to_utf8())
		
		if error != OK:
			push_error("Error when sending a request: %s" % String(error))


func send_message(message: String) -> int:
	return client.get_peer(1).put_packet(message.to_utf8())


func _on_data():
	var string = client.get_peer(1).get_packet().get_string_from_utf8()
	# All messages are JSON encoded
	var result = JSON.parse(string)
	if result.error:
		print("Error while parsing JSON")
	var message = result.result
	
	# Message type matches with the signal name
	# We could just emit one signal for all the messages
	# But this is more convenient, because you can subscribe to
	# a message of a specific type and don't need to check it
	# every time
	emit_signal(message["type"], message)


func _closed(was_clean = false):
	push_error("Closed, clean: %s" % was_clean)
