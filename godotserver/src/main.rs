use serde::{Serialize, Deserialize};
use tokio::{
    net::UdpSocket,
    sync::Mutex,
    time::{sleep, Duration, Instant}
};
use core::str;
use std::{
    collections::HashMap, net::SocketAddr, sync::Arc, io
};
use regex::Regex;



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
/// 
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
/// 
#[derive(Serialize, Deserialize, Clone)]
pub struct Player {
    name: String,
    health: u8,
    session_id: String,
    state: PlayerState,
    last_ping_time: u64,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let socket = Arc::new(UdpSocket::bind("127.0.0.1:5000").await?);
    let player_list: Arc<Mutex<HashMap<SocketAddr, Player>>> = Arc::new(Mutex::new(HashMap::new()));
    
    println!("Server is online.");

    // Ping coroutine
    let socket_task = Arc::clone(&socket);
    let player_list_task = Arc::clone(&player_list);
    
    tokio::spawn(async move {
        let start_time = Instant::now();
        loop {

            let mut player_list = player_list_task.lock().await;
            let mut to_remove = Vec::new();

            let current_time = start_time.elapsed().as_millis() as u64;

            for (addr, player) in player_list.iter_mut() {
                if current_time - player.last_ping_time > 10_000 {
                    println!("Player {} timed out, kicking.", addr);
                    to_remove.push(*addr);
                    continue;
                }
                println!("{}: {}", player.name.clone(), (start_time.elapsed().as_millis() as u64 - player.last_ping_time.clone()));
                let packet = Packet { packet_type: PacketType::Misc, content: "Ping".to_string() };
                if let Err(e) = send_to_client(&socket_task, addr, &packet).await {
                    println!("Error sending to {}: {:?}", addr, e);
                    to_remove.push(*addr);
                }
            }

            for addr in to_remove {
                player_list.remove(&addr);
            }
            
            sleep(Duration::from_secs(5)).await;
        }
    });

    // Main server loop
    let socket_receiver = Arc::clone(&socket);
    let player_list_server = Arc::clone(&player_list);
    let start_time = Instant::now();

    loop {  
        let mut buf = [0; 1024];
        let (len, addr) = match socket_receiver.recv_from(&mut buf).await {
            Ok(data) => data,
            Err(_) => continue,
        };        
        
        let packet = decode_udp_packet(&buf[..len]);
        let mut players = player_list_server.lock().await;

        if players.contains_key(&addr) {
            handle_packet(&socket_receiver, &addr, &packet, &mut players, start_time.elapsed().as_millis() as u64).await;
        } else {
            handle_new_connection(&socket_receiver, &addr, &packet, &mut players, start_time.elapsed().as_millis() as u64).await;
        }
    }
}

async fn handle_packet(socket: &Arc<UdpSocket>, addr: &SocketAddr, packet: &Packet, players: &mut HashMap<SocketAddr, Player>, current_time: u64) {
    match packet.packet_type {
        PacketType::Chat => {
            let player_name = players.get(addr).unwrap().name.clone();
            let message = format!("{}: {}", player_name, packet.content);
            println!("{}", message);
            broadcast_message(socket, players, Packet { packet_type: PacketType::Chat, content: message }).await;
        },
        PacketType::Sync => {
            if let Some(player) = players.get_mut(addr) {
                let new_player = decode_player_info(&packet.content);
                if player.session_id != new_player.session_id {
                    println!("Session ID mismatch from {}", addr);
                    let _ = send_to_client(socket, addr, &Packet { packet_type: PacketType::Misc, content: "Session Mismatch, Kicked".to_string() }).await;
                    players.remove(addr);
                } else {
                    update_player_info(player, new_player);
                    let _ = send_to_client(socket, addr, &Packet { packet_type: PacketType::Sync, content: object_to_json(player) }).await;
                    let _ = send_to_client(socket, addr, &Packet { packet_type: PacketType::Chat, content: "Welcome to the server".to_string() }).await;
                }
            }
        },
        PacketType::Misc => {
            if packet.content == "Pong" {
                if let Some(player) = players.get_mut(addr) {
                    player.last_ping_time = current_time;
                }
            }
        },
        _ => {}
    }
}

async fn handle_new_connection(socket: &Arc<UdpSocket>, addr: &SocketAddr, packet: &Packet, players: &mut HashMap<SocketAddr, Player>, current_time: u64) {
    if is_valid_guid(&packet.content) {
        let _ = send_to_client(socket, addr, &Packet { packet_type: PacketType::Sync, content: "PlayerInfo".to_string() }).await;
        players.insert(*addr, Player { 
            name: "".to_string(), 
            health: 0, 
            session_id: packet.content.clone(), 
            state: PlayerState::Loading,
            last_ping_time: current_time,
        });
        println!("Player from {} just connected to the server", addr);
    } else {
        println!("Packet from {} just got rejected due to not being in the server", addr);
    }
}

async fn broadcast_message(socket: &Arc<UdpSocket>, players: &HashMap<SocketAddr, Player>, packet: Packet) {
    for addr in players.keys() {
        let _ = send_to_client(socket, addr, &packet).await;
    }
}

fn update_player_info(player: &mut Player, new_player: Player) {
    player.name = new_player.name;
    player.health = 100;
    player.state = PlayerState::Alive;
}

fn is_valid_guid(guid: &str) -> bool {
    let guid_regex = Regex::new(r"^[{(]?([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})[)}]?$").unwrap();
    guid_regex.is_match(guid)
}

async fn send_to_client(socket: &UdpSocket, addr: &SocketAddr, packet: &Packet) -> io::Result<()> {
    let packet = serde_json::to_string(packet)?;
    socket.send_to(packet.as_bytes(), addr).await?;
    Ok(())
}

fn decode_udp_packet(bytes: &[u8]) -> Packet {
    serde_json::from_str(str::from_utf8(bytes).unwrap()).unwrap()
}

fn decode_player_info(json_player: &str) -> Player {
    serde_json::from_str(json_player).unwrap_or_else(|_| Player {
        name: "".to_string(),
        health: 0,
        session_id: "".to_string(),
        state: PlayerState::Error,
        last_ping_time: 0,
    })
}

fn object_to_json<T: Serialize>(obj: &T) -> String {
    serde_json::to_string(obj).unwrap_or_else(|_| "ERROR".to_string())
}