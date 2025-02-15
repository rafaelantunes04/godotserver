using Godot;
using Newtonsoft.Json;

namespace Rb {
	public partial class Client : Node
	{
		private NetworkManager networkManager = new NetworkManager();
		private PlayerInfo playerInfo = new PlayerInfo();

		public string Username { get; set; }
		public string Address { get; set; }
		public int Port { get; set; }

		public override async void _Ready()
		{
			string sessionId = await networkManager.StartClient();
			playerInfo.Populate(sessionId, Username);
			GD.Print("Client Online");

			// Connect to server
			networkManager.Connect(Address, Port, playerInfo.session_id);

			// Start packet receiver
			_ = networkManager.StartReceiver(HandlePacket);
		}

		public override void _Process(double delta)
		{
			if (Input.IsActionJustPressed("mandar"))
			{
				networkManager.SendPacket(new NetworkManager.Packet
				{
					packet_type = NetworkManager.PacketType.Chat,
					content = "Hello"
				});
			}
		}

		private void HandlePacket(NetworkManager.Packet packet)
		{
			switch (packet.packet_type)
			{
				case NetworkManager.PacketType.Chat:
					GD.Print(packet.content);
					break;

				case NetworkManager.PacketType.Sync:
					if (packet.content == "PlayerInfo")
					{
						networkManager.SendPacket(new NetworkManager.Packet
						{
							packet_type = NetworkManager.PacketType.Sync,
							content = JsonConvert.SerializeObject(playerInfo)
						});
					}
					else
					{
						try
						{
							var updatedPlayerInfo = JsonConvert.DeserializeObject<PlayerInfo>(packet.content);
							playerInfo.name = updatedPlayerInfo.name;
							playerInfo.health = updatedPlayerInfo.health;
							playerInfo.state = updatedPlayerInfo.state;
							playerInfo.last_ping_time = updatedPlayerInfo.last_ping_time;
						}
						catch
						{
							GD.Print("Not Player Info");
						}
					}
					break;

				case NetworkManager.PacketType.Misc:
					if (packet.content == "Ping")
					{
						networkManager.SendPacket(new NetworkManager.Packet
						{
							packet_type = NetworkManager.PacketType.Misc,
							content = "Pong"
						});
					}
					break;
			}
		}
	}
}
