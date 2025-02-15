extends Control

@onready var startButton = $VContainer/StartButton
@onready var nameButton = $VContainer/nameButton
@onready var quitButton = $VContainer/QuitButton
@onready var userField = $VContainer/HBoxContainer/Name
var username = "Example"
var addressPopup = load("res://scenes/main_menu/address_popup.tscn")
var namePopup = load("res://scenes/main_menu/name_popup.tscn")

func _ready() -> void:
	userField.placeholder_text = username

func reset():
	startButton.disabled = false;
	nameButton.disabled = false;
	quitButton.disabled = false;
	

func _on_start_button_pressed() -> void:
	if userField.text.is_valid_identifier():
		username = userField.text
	else: username = "Example"
	var pop = addressPopup.instantiate()
	startButton.disabled = true;
	nameButton.disabled = true;
	quitButton.disabled = true;
	add_child(pop)


func _on_quit_button_pressed() -> void:
	get_tree().quit()
