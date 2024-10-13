using Godot;
using System;
using System.Net.Sockets;
using System.Text;
using Newtonsoft.Json;
using Newtonsoft.Json.Converters;
using Open.Nat;
using System.Threading.Tasks;

namespace Rb {
	public class ConnectionHandler
	{
		private UdpClient udpClient = null;
		private Tuple<string, int> serverInfo;

		//Packet type that is used in the packet
		public enum PacketType
		{
			Chat,
			Sync,
			SyncHealth,
			SyncState,
			Misc,
		}

		//Packet object that is sent to server
		public class Packet
		{
			[JsonConverter(typeof(StringEnumConverter))]
			public PacketType packet_type { get; set; }
			public string content { get; set; }
		}
		
		//Start Udp Packet Reciever
		public async void StartReciever(PlayerInfo playerInfo)
		{
			while (true) 
			{
				try
				{
					var result = await udpClient.ReceiveAsync();

					if (result.RemoteEndPoint.Address.ToString() == serverInfo.Item1) 
					{
						string sent_packet = Encoding.UTF8.GetString(result.Buffer);
						var packet = JsonConvert.DeserializeObject<Packet>(sent_packet);
						switch (packet.packet_type)
						{
							case PacketType.Chat:
								GD.Print(packet.content);
								break;
							
							case PacketType.Sync:
								if (packet.content == "PlayerInfo")
									{
										SendPacketToServer(new Packet { packet_type = PacketType.Sync, content = JsonConvert.SerializeObject(playerInfo) });
									} else {
										try 
										{
											var updatedPlayerInfo = JsonConvert.DeserializeObject<PlayerInfo>(packet.content);
											playerInfo.name = updatedPlayerInfo.name;
											playerInfo.health = updatedPlayerInfo.health;
											playerInfo.state = updatedPlayerInfo.state;
										}
										catch {
											GD.Print("Not Player Info");
										}
									}
										
								break;

							case PacketType.SyncHealth:
								playerInfo.health = (sbyte)int.Parse(packet.content);
								break;
							
							case PacketType.SyncState:
								playerInfo.state = JsonConvert.DeserializeObject<PlayerState>(packet.content);
								break;
							
							case PacketType.Misc:
								break;
						}
					}
					else 
					{
						GD.PrintErr("Recieved Packet from unauthorized IP: ", result.RemoteEndPoint.Address.ToString());
					}
				}
				catch (Exception e)
				{
					GD.PrintErr("Error receiving data: ", e);
				}
			}
		}

		//Send Packet to server
		public async void SendPacketToServer(Packet packet) 
		{
			try
			{
				string json = JsonConvert.SerializeObject(packet);
				byte[] data = Encoding.UTF8.GetBytes(json);
				await udpClient.SendAsync(data, data.Length, serverInfo.Item1, serverInfo.Item2);
			}
			catch (Exception e)
			{
				GD.PrintErr("Couldn't send Packet to server: ", e);
			}
			
		}
	
		//Start Client, starting Packet Reciever afterwards
		public async Task<string> StartClient(PlayerInfo playerInfo) 
		{
			//fetch port
			int internalPort = 0;
			for (int port = 7777; port <= 8888; port++) 
			{
				try 
				{
					udpClient = new UdpClient(port);
					internalPort = port;
					break;
				}
				catch (SocketException) {}
			}
			if (udpClient == null)
			{
				GD.PrintErr("Couldnt bind port");
				return null;
			}

			//Upnp setup
			var discoverer = new NatDiscoverer();
			var cts = new System.Threading.CancellationTokenSource(20000);
			NatDevice device = await discoverer.DiscoverDeviceAsync(PortMapper.Upnp, cts);

			// Map the port
			await device.CreatePortMapAsync(new Mapping(Protocol.Udp, internalPort, internalPort, "Client UDP Server"));

			StartReciever(playerInfo);
			string session_id = Guid.NewGuid().ToString();
			return session_id;
		}
	
		//Connect to the server in the serverInfo
		public void connect(Tuple<string, int> _serverInfo, string session_id)
		{
			serverInfo = _serverInfo;
			SendPacketToServer(new Packet { packet_type = PacketType.Misc, content = session_id });
		}
	}
}
