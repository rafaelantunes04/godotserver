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
        
        [JsonConverter(typeof(StringEnumConverter))]
        public PlayerState state { get; set; }

        public void populate(string _session_id, string _name) 
        {
            session_id = _session_id;
            name = _name;
            health = 0;
            state = PlayerState.Loading;
        }
	}
}