extends Control

@onready var IPField = $CenterContainer/Panel/VBoxContainer/HBoxContainer/IP
@onready var portField = $CenterContainer/Panel/VBoxContainer/HBoxContainer/Port
var ChatScene = load("res://scenes/chat_room/chat_room.tscn")


func _on_join_button_pressed() -> void:
	var ip = IPField.text 
	var port = portField.text
	var chatInstance = ChatScene.instantiate()
	chatInstance.Username = get_parent().username
	chatInstance.Address = ip
	if port.is_valid_int(): 
		chatInstance.Port = int(port)
		get_parent().queue_free();
		get_tree().root.add_child(chatInstance);
		get_tree().current_scene = chatInstance;
	else:
		print("Write a Valid Port")
	
	
	
