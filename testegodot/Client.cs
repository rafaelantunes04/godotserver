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

		connectToServer(udpClient);
	}

	// Called every frame. 'delta' is the elapsed time since the previous frame.
	public override void _Process(double delta)
	{
		if (Input.IsActionJustPressed("mandar")) {
			sendUDP(udpClient);
		}
	}

	private async void StartRecievingAsync() 
	{
		while (true) 
		{
			var result = await udpClient.ReceiveAsync();
			string message = Encoding.ASCII.GetString(result.Buffer);
			GD.Print(message);
		}
	}

	private void startClient() 
	{
		udpClient = new UdpClient(clientPort); // Initialize the UDP client to listen for messages

		_playerId = Guid.NewGuid().ToString();

		StartRecievingAsync();
	}

	private void connectToServer(UdpClient client) 
	{
		string message = _playerId;
		byte[] data = Encoding.ASCII.GetBytes(message);
		client.Send(data, data.Length, serverInfo.Item1, serverInfo.Item2);
		
	}

	private void sendUDP(UdpClient client) 
	{
		string message = "Hello from C#";
		byte[] data = Encoding.ASCII.GetBytes(message);

		client.Send(data, data.Length, serverInfo.Item1, serverInfo.Item2);
	}
}
