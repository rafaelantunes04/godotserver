using Godot;
using Rb;
using System;

public partial class address_popup : Control
{

	public string Username { get; set; }

	private Label title;
	private HBoxContainer buttonContainer;
	private Button confirmButton;
	private Button cancelButton;
	private HBoxContainer inputContainer;

	private Vector2 originalButtonContainerSize;
	private Vector2 originalWindowSize;
	private Vector2 originalInputContainerSize;
	
	private int originalTitleFontSize;
	private int originalButtonFontSize;
	private int originalInputContainerFontSize;

	private PackedScene client = ResourceLoader.Load<PackedScene>("res://scenes/client.tscn");
	private Node clientInstance;

	public override void _Ready()
	{
		title = GetNode<Label>("title");
		buttonContainer = GetNode<HBoxContainer>("button_container");
		inputContainer = GetNode<HBoxContainer>("input_container");
		confirmButton = buttonContainer.GetNode<Button>("confirm_button");
		cancelButton = buttonContainer.GetNode<Button>("cancel_button");


		originalWindowSize = DisplayServer.WindowGetSize();
		originalButtonContainerSize = buttonContainer.Size;
		originalInputContainerSize = inputContainer.Size;

		originalTitleFontSize = title.LabelSettings.FontSize;
		originalButtonFontSize = confirmButton.Theme.DefaultFontSize;
		originalInputContainerFontSize = inputContainer.Theme.DefaultFontSize;
	}

	// Called every frame. 'delta' is the elapsed time since the previous frame.
	public override void _Process(double delta)
	{
		Vector2 windowSize = DisplayServer.WindowGetSize();

		Size = windowSize / 2;
		Position = (windowSize - Size) / 2;

		buttonContainer.Size = new Vector2((Size.X*originalButtonContainerSize.X)/originalWindowSize.X, (Size.Y * originalButtonContainerSize.Y) / originalWindowSize.Y);
		buttonContainer.Position = new Vector2(Size.X / 2 - buttonContainer.Size.X / 2, ((Size.Y * 4) / 5) - buttonContainer.Size.Y / 2);

		inputContainer.Size = new Vector2((Size.X*originalInputContainerSize.X)/originalWindowSize.X, (Size.Y * originalInputContainerSize.Y) / originalWindowSize.Y);
		inputContainer.Position = new Vector2((Size.X / 2) - (inputContainer.Size.X / 2), (Size.Y / 2) - ((inputContainer.Size.Y * 3) / 5));

		confirmButton.Theme.DefaultFontSize = (int)Math.Round((Size.Y * originalButtonFontSize) / originalWindowSize.Y);
		cancelButton.Theme.DefaultFontSize = (int)Math.Round((Size.Y * originalButtonFontSize) / originalWindowSize.Y);
		inputContainer.Theme.DefaultFontSize = (int)Math.Round((Size.Y * originalInputContainerFontSize) / originalWindowSize.Y);

		title.LabelSettings.FontSize = (int)Math.Round((Size.Y * originalTitleFontSize) / originalWindowSize.Y);
		title.Position = new Vector2(Size.X / 2 - title.Size.X / 2, Size.Y / 6 - title.Size.Y / 2);
	}

	private void _on_confirm_button_pressed()
	{
		if ((inputContainer.GetNode<LineEdit>("ip").Text.Length > 7) && (inputContainer.GetNode<LineEdit>("port").Text.Length > 0)) 
		{
			Client clientInstance = client.Instantiate<Client>();
			clientInstance.Username = Username;
			clientInstance.Address = inputContainer.GetNode<LineEdit>("ip").Text;
			try
			{
				clientInstance.Port = int.Parse(inputContainer.GetNode<LineEdit>("port").Text);
				GetParent().QueueFree();
				GetTree().Root.AddChild(clientInstance);
				GetTree().CurrentScene = clientInstance;
			}
			catch 
			{
				GD.Print("Must be a valid port");
			}
		}
	}

	private void _on_cancel_button_pressed()
	{
		GetParent().RemoveChild(this);
	}
}
