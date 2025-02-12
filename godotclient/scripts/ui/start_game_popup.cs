using Godot;
using Rb;
using System;

public partial class start_game_popup : Control
{
	private Label title;
	private HBoxContainer buttonContainer;
	private Button confirmButton;
	private Button cancelButton;
	private LineEdit nameInput;

	private Vector2 originalButtonContainerSize;
	private Vector2 originalWindowSize;
	private Vector2 originalNameInputSize;
	
	private int originalTitleFontSize;
	private int originalButtonFontSize;
	private int originalNameInputFontSize;

	private PackedScene client = ResourceLoader.Load<PackedScene>("res://scenes/client.tscn");
	private Node clientInstance;

	public override void _Ready()
	{
		title = GetNode<Label>("title");
		buttonContainer = GetNode<HBoxContainer>("button_container");
		nameInput = GetNode<LineEdit>("name_input");
        confirmButton = buttonContainer.GetNode<Button>("confirm_button");
        cancelButton = buttonContainer.GetNode<Button>("cancel_button");


        originalWindowSize = DisplayServer.WindowGetSize();
		originalButtonContainerSize = buttonContainer.Size;
		originalNameInputSize = nameInput.Size;

		originalTitleFontSize = title.LabelSettings.FontSize;
		originalButtonFontSize = confirmButton.Theme.DefaultFontSize;
		originalNameInputFontSize = nameInput.Theme.DefaultFontSize;
	}

	// Called every frame. 'delta' is the elapsed time since the previous frame.
	public override void _Process(double delta)
	{
		Vector2 windowSize = DisplayServer.WindowGetSize();

		Size = windowSize / 2;
        Position = (windowSize - Size) / 2;

        buttonContainer.Size = new Vector2((Size.X*originalButtonContainerSize.X)/originalWindowSize.X, (Size.Y * originalButtonContainerSize.Y) / originalWindowSize.Y);
		buttonContainer.Position = new Vector2(Size.X / 2 - buttonContainer.Size.X / 2, ((Size.Y * 4) / 5) - buttonContainer.Size.Y / 2);

		nameInput.Size = new Vector2((Size.X*originalNameInputSize.X)/originalWindowSize.X, (Size.Y * originalNameInputSize.Y) / originalWindowSize.Y);
		nameInput.Position = new Vector2((Size.X / 2) - (nameInput.Size.X / 2), (Size.Y / 2) - ((nameInput.Size.Y * 3) / 5));

        confirmButton.Theme.DefaultFontSize = (int)Math.Round((Size.Y * originalButtonFontSize) / originalWindowSize.Y);
		cancelButton.Theme.DefaultFontSize = (int)Math.Round((Size.Y * originalButtonFontSize) / originalWindowSize.Y);
		nameInput.Theme.DefaultFontSize = (int)Math.Round((Size.Y * originalNameInputFontSize) / originalWindowSize.Y);

        title.LabelSettings.FontSize = (int)Math.Round((Size.Y * originalTitleFontSize) / originalWindowSize.Y);
		title.Position = new Vector2(Size.X / 2 - title.Size.X / 2, Size.Y / 6 - title.Size.Y / 2);
    }

	private void _on_confirm_button_pressed()
	{
		if (nameInput.Text.Length > 0) 
		{
			Client clientInstance = client.Instantiate<Client>();
			clientInstance.Username = nameInput.Text;
			GetParent().QueueFree();
			GetTree().Root.AddChild(clientInstance);
			GetTree().CurrentScene = clientInstance;
		}
	}

	private void _on_cancel_button_pressed()
	{
		GetParent().RemoveChild(this);
    }
}
