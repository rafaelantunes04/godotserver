extends Control

var Username
var Address
var Port

@export var client: Node

@onready var chatBox = $Panel/VBoxContainer/ChatBox
@onready var chatWrite = $Panel/VBoxContainer/ChatWrite

signal chatSend(contents)

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	client.Username = Username
	client.Address = Address
	client.Port = Port


func _on_client_chat_rec(contents: String) -> void:
	chatBox.text += contents + "\n"

func _input(event):
	if event.is_action_pressed("send_message"):
		var message = chatWrite.text.strip_edges()
		if message:
			client.SendMessage(message)
			chatWrite.text = ""
