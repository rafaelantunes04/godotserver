using Newtonsoft.Json;
using Newtonsoft.Json.Converters;

namespace Rb {
	public enum PlayerState
	{
		Loading,
		Dead,
		Alive,
		Error,
	}

	public class PlayerInfo
	{
		public string name { get; set; }
		public sbyte health { get; set; }
		public string session_id { get; set; }
		public int last_ping_time { get; set; }

		[JsonConverter(typeof(StringEnumConverter))]
		public PlayerState state { get; set; }

		public void Populate(string sessionId, string playerName)
		{
			session_id = sessionId;
			name = playerName;
			health = 0;
			state = PlayerState.Loading;
			last_ping_time = 0;
		}
	}
}
