using System;
using Godot;
using Newtonsoft.Json;

namespace Rb
{
	public partial class Client : Control
	{
		ConnectionHandler connectionHandler = new ConnectionHandler();

		public PlayerInfo playerInfo = new PlayerInfo();

		public override void _Ready()
		{
			String session_id = connectionHandler.StartClient(playerInfo);
			playerInfo.populate(session_id, "jogador");
			GD.Print("Client Online");
		
			//Connect to server
			connectionHandler.connect(new Tuple<string, int>("127.0.0.1", 5000), playerInfo.session_id);
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
