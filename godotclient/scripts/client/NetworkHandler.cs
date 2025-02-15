using System;
using System.Net.Sockets;
using System.Text;
using System.Threading.Tasks;
using Newtonsoft.Json;
using Newtonsoft.Json.Converters;
using Godot;
using Open.Nat;

namespace Rb {
	public class NetworkManager
	{
		private UdpClient udpClient;
		private Tuple<string, int> serverInfo;


		// PacketType Enum
		public enum PacketType
		{
			Chat,
			Sync,
			Misc,
		}



		// Packet Class
		public class Packet
		{
			[JsonConverter(typeof(StringEnumConverter))]
			public PacketType packet_type { get; set; }
			public string content { get; set; }
		}



		/// StartClient Function
		/// 
		/// The StartClient() Function is for starting UDP Transmitter and Reciever.
		/// 
		/// # Returns: Guid
		/// 
		public async Task<string> StartClient()
		{
			int internalPort = 0;
			for (int port = 7777; port <= 8888; port++)
			{
				try
				{
					udpClient = new UdpClient(port);
					internalPort = port;
					break;
				}
				catch (SocketException) { }
			}

			if (udpClient == null)
			{
				GD.PrintErr("Couldn't bind port");
				return null;
			}

			// UPnP
			try
			{
				var discoverer = new NatDiscoverer();
				var cts = new System.Threading.CancellationTokenSource(20000);
				NatDevice device = await discoverer.DiscoverDeviceAsync(PortMapper.Upnp, cts);
				await device.CreatePortMapAsync(new Mapping(Protocol.Udp, internalPort, internalPort, "Client UDP Server"));
			}
			catch (Exception e)
			{
				GD.PrintErr("UPnP setup failed: ", e);
			}

			return Guid.NewGuid().ToString();
		}

		public void Connect(string serverIp, int serverPort, string sessionId)
		{
			serverInfo = new Tuple<string, int>(serverIp, serverPort);
			SendPacket(new Packet { packet_type = PacketType.Misc, content = sessionId });
		}

		public async void SendPacket(Packet packet)
		{
			try
			{
				string json = JsonConvert.SerializeObject(packet);
				byte[] data = Encoding.UTF8.GetBytes(json);
				await udpClient.SendAsync(data, data.Length, serverInfo.Item1, serverInfo.Item2);
			}
			catch (Exception e)
			{
				GD.PrintErr("Couldn't send packet to server: ", e);
			}
		}

		public async Task StartReceiver(Action<Packet> onPacketReceived)
		{
			while (true)
			{
				try
				{
					var result = await udpClient.ReceiveAsync();
					string sentPacket = Encoding.UTF8.GetString(result.Buffer);
					var packet = JsonConvert.DeserializeObject<Packet>(sentPacket);
					onPacketReceived(packet);
				}
				catch (Exception e)
				{
					GD.PrintErr("Error receiving data: ", e);
				}
			}
		}
	}
}
