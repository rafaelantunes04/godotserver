using Godot;
using System;

public partial class main_menu : Control
{
    private VBoxContainer buttonsBox;
    private Label title;

    private Button startButton;
    private Button optionsButton;
    private Button exitButton;

    private Vector2 originalWindowSize;
    private Vector2 originalButtonsBoxSize;
    private int originalTitleFontSize;
    private int originalButtonsBoxFontSize;

    private PackedScene startGamePopup = ResourceLoader.Load<PackedScene>("res://scenes/main_menu/start_game_popup.tscn");

    private Control popupInstance;

    public override void _Ready()
    {
        buttonsBox = GetNode<VBoxContainer>("button_container");
        startButton = buttonsBox.GetNode<Button>("start_game_btn");
        optionsButton = buttonsBox.GetNode<Button>("options_btn");
        exitButton = buttonsBox.GetNode<Button>("exit_btn");

        title = GetNode<Label>("title");

        originalWindowSize = GetViewport().GetVisibleRect().Size;
        originalButtonsBoxSize = buttonsBox.Size;
        originalButtonsBoxFontSize = buttonsBox.Theme.DefaultFontSize;
        originalTitleFontSize = title.LabelSettings.FontSize;

        popupInstance = startGamePopup.Instantiate<Control>();
    }

    public override void _Process(double delta)
    {
        if (GetNodeOrNull<Control>("StartGamePopup") == null) 
        {
            startButton.Disabled = false;
            optionsButton.Disabled = false;
            exitButton.Disabled = false;
        }


        Vector2 windowSize = GetViewport().GetVisibleRect().Size;

        // ButtonsBox
        buttonsBox.GlobalPosition = new Vector2(windowSize.X / 2 - buttonsBox.Size.X / 2, windowSize.Y / 2 - buttonsBox.Size.Y / 2) + new Vector2(0, 50);
        buttonsBox.Size = new Vector2((windowSize.X * originalButtonsBoxSize.X) / originalWindowSize.X, (windowSize.Y * originalButtonsBoxSize.Y) / originalWindowSize.Y);

        // ButtonsBox Font Size
        buttonsBox.Theme.DefaultFontSize = (int)Math.Round((windowSize.Y * originalButtonsBoxFontSize) / originalWindowSize.Y);

        // Title Font Size
        title.LabelSettings.FontSize = (int)Math.Round((windowSize.Y * originalTitleFontSize) / originalWindowSize.Y);
        title.GlobalPosition = new Vector2((windowSize.X / 2) - title.Size.X/2 , windowSize.Y / 4 - title.Size.Y/2);
    }

    private void _on_start_game_btn_pressed()
    {
        startButton.Disabled = true;
        optionsButton.Disabled = true;
        exitButton.Disabled = true;
        AddChild(popupInstance);
    }

    private void _on_options_btn_pressed()
    {
        GD.Print("Options");
    }

    private void _on_exit_btn_pressed()
    {
        GD.Print("Exit");
    }
}