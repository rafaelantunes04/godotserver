using System;
using Godot;

namespace Rb
{
	public partial class Client : Control
	{
		ConnectionHandler connectionHandler = new ConnectionHandler();

		public PlayerInfo playerInfo = new PlayerInfo();

		public override async void _Ready()
		{
			string session_id = await connectionHandler.StartClient(playerInfo);
			playerInfo.populate(session_id, "jogador");
			GD.Print("Client Online");
		
			//Connect to server
			connectionHandler.connect(new Tuple<string, int>("85.139.143.153", 5000), playerInfo.session_id);
		}

		// Called every frame. 'delta' is the elapsed time since the previous frame.
		public override void _Process(double delta)
		{
			if (Input.IsActionJustPressed("mandar")) 
			{
				connectionHandler.SendPacketToServer(new ConnectionHandler.Packet { packet_type = ConnectionHandler.PacketType.Chat, content= "Hello" });
			}
		}
	}
}
