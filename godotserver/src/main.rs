use serde::{Serialize, Deserialize};
use tokio::{
    net::UdpSocket,
    time::{sleep, Duration}
};
use core::str;
use std::{
    collections::HashMap, fmt, net::SocketAddr, sync::{Arc, Mutex}
};

#[derive(Serialize, Deserialize)]
pub enum PacketType {
    Chat,
    Sync,
    SyncHealth,
    SyncState,
    Misc,
}

#[derive(Serialize, Deserialize)]
pub enum PlayerState {
    Loading,
    Dead,
    Alive,
    Error,
}

impl fmt::Display for PlayerState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Match on the enum variant and return a corresponding string
        let state = match self {
            PlayerState::Loading => "Loading",
            PlayerState::Dead => "Dead",
            PlayerState::Alive => "Alive",
            PlayerState::Error => "Error",
        };
        write!(f, "{}", state)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Packet {
    pub packet_type: PacketType,
    pub content: String,
}

impl Packet {
    pub fn as_bytes(&self) -> Vec<u8> {
        self.content.as_bytes().to_vec()
    }
}


#[derive(Serialize, Deserialize)]
pub struct Player {
    name: String,
    health: i8,
    session_id: String,
    state: PlayerState,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:5000").await?;

    let player_list: Arc<Mutex<HashMap<SocketAddr, Player>>> = Arc::new(Mutex::new(HashMap::new()));
    
    println!("Online");
    //Player Connection Checker
    let player_list_task = Arc::clone(&player_list);
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(3)).await;

            let player_list = player_list_task.lock().unwrap();

            if player_list.is_empty() {
                continue
            }

            for (addr, player) in player_list.iter() {
                println!("Address: {}, Name: {}, Health: {}, Session: {}, State: {}", addr, player.name, player.health, player.session_id, player.state);
            }
        }
    });
    
    //Main Server Code
    let player_list_server = Arc::clone(&player_list);
    loop {  
        let mut buf = [0; 1024];
        let (len, addr) = socket.recv_from(&mut buf).await?;
        let packet = decode_udp_packet(&buf[..len]);
        
        let mut players = player_list_server.lock().unwrap();

        //check if the ip address is in the playerlist
        if players.contains_key(&addr) {
            match packet.packet_type {
                PacketType::Chat => {
                    for (addr, _) in players.iter() {
                        send_to_client(&socket, addr, &Packet { packet_type: PacketType::Chat, content: ( players.get(&addr).unwrap().name.to_string() + ": " + &packet.content) }).await;
                    }
                }
                
                PacketType::Sync => {
                    let mut player = decode_player_info(&packet.content);
                    match player.state {
                        PlayerState::Loading => {
                            player.health = 100;
                            player.state = PlayerState::Alive;
                            let mut session_mismatch: bool = false;
                            if let Some(serverplayerdata) = players.get_mut(&addr) {
                                if serverplayerdata.session_id != player.session_id {
                                    println!("Packet from {} just got rejected and kicked due to having session_id mismatch", addr );
                                    send_to_client(&socket, &addr, &Packet { packet_type: PacketType::Misc, content: "Session Mismatch, Kicked".to_string() }).await;
                                    session_mismatch = true;
                                } else {
                                    serverplayerdata.name = player.name;
                                    serverplayerdata.health = player.health;
                                    serverplayerdata.session_id = player.session_id;
                                    serverplayerdata.state = player.state;
                                    send_to_client(&socket, &addr, &Packet { packet_type: PacketType::Chat, content: "Welcome to the server: ".to_string() + &serverplayerdata.name }).await;
                                    send_to_client(&socket, &addr, &Packet { packet_type: PacketType::Sync, content: object_to_json(serverplayerdata) }).await;
                                }
                                if session_mismatch {
                                    players.remove(&addr);
                                }
                            };

                            
                        }

                        PlayerState::Alive => {

                        }

                        PlayerState::Dead => {

                        }

                        PlayerState::Error => {

                        }
                    }
                }
                PacketType::SyncHealth => {
                }
                PacketType::SyncState => {
                    
                }
                PacketType::Misc => {
                    if is_valid_guid(&packet.content) {
                        if packet.content != players.get(&addr).unwrap().session_id {
                            println!("Packet from {} just got rejected and kicked due to having session_id mismatch", addr );
                            send_to_client(&socket, &addr, &Packet { packet_type: PacketType::Misc, content: "Session Mismatch, Kicked".to_string() }).await;
                            players.remove(&addr);
                        }
                    }
                }
            }
        } else {
            //check if message is a guid to connect to the server
            if is_valid_guid(&packet.content) {
                send_to_client(&socket, &addr, &Packet { packet_type: PacketType::Sync, content: "PlayerInfo".to_string() }).await;
                players.insert(addr, Player { name: "".to_string(), health: 0, session_id: packet.content, state: PlayerState::Loading });
                println!("Player from {} just connected to the server", addr);
            } else {
                println!("Packet from {} just got rejected due to not being in the server", addr)
            }
        }
    }
}

fn is_valid_guid(guid: &str) -> bool {
    let guid_regex = regex::Regex::new(r"^[{(]?([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})[)}]?$").unwrap();
    guid_regex.is_match(guid)
}

async fn send_to_client(socket: &UdpSocket, addr: &SocketAddr, packet: &Packet) {
    let packet = serde_json::to_string::<Packet>(&packet).unwrap();
    let bites = packet.as_bytes();
    if let Err(e) = socket.send_to(bites, addr).await {
        println!("Couldnt send message to {}: {}", addr, e);
    }
}
fn decode_udp_packet(bytes: &[u8]) -> Packet {
    return serde_json::from_str::<Packet>(str::from_utf8(bytes).unwrap()).unwrap();
}

fn decode_player_info(json_player: &String) -> Player{
    match serde_json::from_str::<Player>(&json_player) {
        Ok(player) => {
            return player;
        },
        Err(_e) => {
        return Player { name: "".to_string(), health: 0, session_id: "".to_string(), state: PlayerState::Error };
        }
    }
}

fn object_to_json<T: Serialize>(obj: &T) -> String {
    match serde_json::to_string(obj) {
        Ok(json_string) => json_string,
        Err(_) => "ERROR".to_string(),
    }
}