use serde::{Serialize, Deserialize};
use tokio::{
    net::UdpSocket,
    sync::Mutex,
    time::{sleep, Duration}
};
use core::str;
use std::{
    collections::HashMap, net::SocketAddr, sync::Arc, io
};


// ERRO AO DISCONECTAR DE CLIENTES, PROBLEMA É EM TIRAR DA HASHTABLE O JOGADOR QUE ESTA A DAR PROBLEMAS POIS ESTA A SER FEITO EM UMA COROTINA DIFERENTE
// QUE MANDA MENSAGEM, SENDO QUE TEM O RESULTADO NA RECEPÇÃO






//CLASSES


//Packet Type
/// This is an enum used to define the utility of the packet sent or recieved
/// 
#[derive(Serialize, Deserialize)]
pub enum PacketType {
    Chat,
    Sync,
    SyncHealth,
    SyncState,
    Misc,
}



//Player State
/// This is an enum used in the player to define the state of the player
/// 
#[derive(Serialize, Deserialize, Clone)]
pub enum PlayerState {
    Loading,
    Dead,
    Alive,
    Error,
}



//Packet
/// This is the object that is sent as json in the udp packet
/// 
/// # Fields
/// - `packet-type`: With the options in range of [`PacketType`], is used to define the utility of the packet.
/// - `content`: With the type of String, is used as the content of the packet, can contain json or just a normal message.
#[derive(Serialize, Deserialize)]
pub struct Packet {
    pub packet_type: PacketType,
    pub content: String,
}



//Player
/// This is the object that is used to contain a player information
/// 
/// # Fields
/// - `name`: With the type of String, contains the player name.
/// - `health`: With the type of u8, contains a value between 0 and 255 for the player's health.
/// - `session_id`: With the type of String, contains the guid of a player's session.
/// - `state`: With the options in range of [`PlayerState`], represents the state the player is in.
#[derive(Serialize, Deserialize, Clone)]
pub struct Player {
    name: String,
    health: u8,
    session_id: String,
    state: PlayerState,
}

#[tokio::main]
async fn main() -> io::Result<()> {

    let socket = Arc::new(UdpSocket::bind("127.0.0.1:5000").await?);
    let player_list: Arc<Mutex<HashMap<SocketAddr, Player>>> = Arc::new(Mutex::new(HashMap::new()));
    
    println!("Server is Online");
    
    // Player Connection Checker
    let socket_task = Arc::clone(&socket);
    let player_list_task = Arc::clone(&player_list);
    
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(5)).await;
            let mut player_list = player_list_task.lock().await;
            let addrs: Vec<_> = player_list.keys().cloned().collect();
            for addr in addrs {
                let packet = Packet { packet_type: PacketType::Misc, content: "Ping".to_string() };
                if let Err(e) = send_to_client(&socket_task, &addr, &packet).await {
                    println!("Error sending to {}: {:?}", addr, e);
                    player_list.remove(&addr);
                    println!("Address {} kicked.", addr);
                }
            }
        }
    });
    
    //Main Server Code
    let socket_receiver = Arc::clone(&socket);
    let player_list_server = Arc::clone(&player_list);
    loop {  
        let mut buf = [0; 1024];

        let len: usize;
        let addr: SocketAddr;

        match socket_receiver.recv_from(&mut buf).await {
            Ok(data) => {(len, addr) = data}
            Err(_) => {continue}
        }        
        
        let packet = decode_udp_packet(&buf[..len]);
        
        let mut players = player_list_server.lock().await;

        //check if the ip address is in the playerlist
        if players.contains_key(&addr) {
            match packet.packet_type {
                PacketType::Chat => {
                    for (addr, _) in players.iter() {
                        send_to_client(&socket_receiver, addr, &Packet { packet_type: PacketType::Chat, content: ( players.get(&addr).unwrap().name.to_string() + ": " + &packet.content) }).await;
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
                                    send_to_client(&socket_receiver, &addr, &Packet { packet_type: PacketType::Misc, content: "Session Mismatch, Kicked".to_string() }).await;
                                    session_mismatch = true;
                                } else {
                                    serverplayerdata.name = player.name;
                                    serverplayerdata.health = player.health;
                                    serverplayerdata.session_id = player.session_id;
                                    serverplayerdata.state = player.state;
                                    send_to_client(&socket_receiver, &addr, &Packet { packet_type: PacketType::Chat, content: "Welcome to the server: ".to_string() + &serverplayerdata.name }).await;
                                    send_to_client(&socket_receiver, &addr, &Packet { packet_type: PacketType::Sync, content: object_to_json(serverplayerdata) }).await;
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
                            send_to_client(&socket_receiver, &addr, &Packet { packet_type: PacketType::Misc, content: "Session Mismatch, Kicked".to_string() }).await;
                            players.remove(&addr);
                        }
                    } 
                    if &packet.content == "Pong" {
                        println!("Player: {}, Confirmed Connection", &addr);
                    }
                }
            }
        } else {
            //check if message is a guid to connect to the server
            if is_valid_guid(&packet.content) {
                send_to_client(&socket_receiver, &addr, &Packet { packet_type: PacketType::Sync, content: "PlayerInfo".to_string() }).await;
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

async fn send_to_client(socket: &UdpSocket, addr: &SocketAddr, packet: &Packet) -> io::Result<()> {
    let packet = serde_json::to_string::<Packet>(&packet).unwrap();
    let bites = packet.as_bytes();
    match socket.send_to(bites, addr).await {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("Could not send message to {}: {}", addr, e);
            Err(e)
        }
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