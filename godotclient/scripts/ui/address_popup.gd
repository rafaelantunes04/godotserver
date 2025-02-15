extends Control

@onready var IPField = $CenterContainer/ColorRect/VBoxContainer/HBoxContainer/IP
@onready var portField = $CenterContainer/ColorRect/VBoxContainer/HBoxContainer/Port
var ClientScene = load("res://scenes/client.tscn")


func _on_join_button_pressed() -> void:
	var ip = IPField.text
	var port = portField.text
	var clientInstance = ClientScene.instantiate()
	clientInstance.Username = get_parent().username
	clientInstance.Address = ip
	if port.is_valid_int(): 
		clientInstance.Port = int(port)
		get_parent().queue_free();
		get_tree().root.add_child(clientInstance);
		get_tree().current_scene = clientInstance;
	else:
		print("Write a Valid Port")
	
	
	
