using Godot;
using System;
using System.Net.Sockets;
using System.Text;

public partial class Client : Node3D
{

	private UdpClient udpClient;
	private string _playerId;
	private const int clientPort = 7777;

	private Tuple<string, int> serverInfo = new Tuple<string, int>("127.0.0.1", 5001);

	public override void _Ready()
	{
		startClient();
		GD.Print("Client Online");

		//Connect to server
		sendMessageToServer(_playerId);
	}

	// Called every frame. 'delta' is the elapsed time since the previous frame.
	public override void _Process(double delta)
	{
		if (Input.IsActionJustPressed("mandar")) 
		{
			sendMessageToServer("Hello");
		}
	}

	private async void StartRecievingAsync() 
	{
		while (true) 
		{
			try
			{
				var result = await udpClient.ReceiveAsync();
				string message = Encoding.ASCII.GetString(result.Buffer);
				GD.Print(message);
			}
			catch (Exception e)
			{
				GD.PrintErr("Error receiving data: ", e);
			}
		}
	}

	//Make the client be able to recieve messages
	private void startClient() 
	{
		udpClient = new UdpClient(clientPort);

		_playerId = Guid.NewGuid().ToString();

		StartRecievingAsync();
	}

	private async void sendMessageToServer(String message) 
	{
		if (string.IsNullOrWhiteSpace(message)) return;

		byte[] data = Encoding.ASCII.GetBytes(message);
		try
		{
			await udpClient.SendAsync(data, data.Length, serverInfo.Item1, serverInfo.Item2);
		}
		catch (Exception e)
		{
			GD.PrintErr("Couldn't send message to server: ", e);
		}
	}
}
