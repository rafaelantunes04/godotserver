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




/// ENUMS

/// Packet Type
/// This is an enum used to define the utility of the packet sent or recieved
///
#[derive(Serialize, Deserialize)]
pub enum PacketType {
    Chat,
    Sync,
    Misc,
}




/// Player State
/// This is an enum used in the player to define the state of the player
///
#[derive(Serialize, Deserialize, Clone)]
pub enum PlayerState {
    Loading,
    Dead,
    Alive,
    Error,
}




/// CLASSES

/// Packet
/// This is the object that is sent as json in the udp packet
/// 
/// # Fields
/// - `packet-type`: With the options in range of [PacketType], is used to define the utility of the packet.
/// - `content`: With the type of String, is used as the content of the packet, can contain json or just a normal message.
///
#[derive(Serialize, Deserialize)]
pub struct Packet {
    pub packet_type: PacketType,
    pub content: String,
}




/// Player
/// This is the object that is used to contain a player information
/// 
/// # Fields
/// - `name`: With the type of String, contains the player name.
/// - `health`: With the type of u8, contains a value between 0 and 255 for the player's health.
/// - `session_id`: With the type of String, contains the guid of a player's session.
/// - `state`: With the options in range of [PlayerState], represents the state the player is in.
/// - `last_ping_time`: last time the player got pinged according to server time.
///
#[derive(Serialize, Deserialize, Clone)]
pub struct Player {
    name: String,
    health: u8,
    session_id: String,
    state: PlayerState,
    last_ping_time: u64,
}




/// PlayerList
/// This is the object for the player list
/// 
/// # Fields
/// - `players`: A [HashMap] with the keys as the players ip addresses.
/// 
/// # Functions
/// - `new()`: Returns a new [PlayerList] object.
/// 
/// - `add_player(addr: SocketAddr, player: Player)`: Adds player as a [Player] to the [HashMap] with the key addr as a [SocketAddr].
/// 
/// - `remove_player(addr: &SocketAddr)`: Removes a player with the address addr from the [HashMap].
/// 
/// - `get_player(addr: &SocketAddr)`: Returns a player with the address addr if it exist, as a [Option<Player>].
/// 
/// - `update_player(addr: SocketAddr, update: impl FnOnce(&mut Player))`: Updates the player with the address addr with the values in update, Example:
///     player_list.update_player(&addr, |player| {
///         player.health = 100;
///         player.name = "player_name".to_string();
///         }).await;
/// 
/// - `get_all_players()`: Returns a non mutable [HashMap] with all players.
/// 
/// - `contains_player(addr: &SocketAddr)`: Returns a [bool] if the address addr is on the [HashMap].
/// 
/// - `broadcast(socket: &UdpSocket, packet: &Packet)`: Sends the packet as a [Packet] to every player in the player_list.
///
#[derive(Clone)]
pub struct PlayerList {
    players: Arc<Mutex<HashMap<SocketAddr, Player>>>,
}

impl PlayerList {
    pub fn new() -> Self {
        PlayerList {
            players: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn add_player(&self, addr: SocketAddr, player: Player) {
        let mut players = self.players.lock().await;
        players.insert(addr, player);
    }

    pub async fn remove_player(&self, addr: &SocketAddr) {
        let mut players = self.players.lock().await;
        players.remove(addr);
    }

    pub async fn get_player(&self, addr: &SocketAddr) -> Option<Player> {
        let players = self.players.lock().await;
        players.get(addr).cloned()
    }
 
    pub async fn update_player(&self, addr: &SocketAddr, update: impl FnOnce(&mut Player)) {
        let mut players = self.players.lock().await;
        if let Some(player) = players.get_mut(addr) {
            update(player);
        }
    }

    pub async fn get_all_players(&self) -> HashMap<SocketAddr, Player> {
        let players = self.players.lock().await;
        players.clone()
    }

    pub async fn contains_player(&self, addr: &SocketAddr) -> bool {
        let players = self.players.lock().await;
        players.contains_key(addr)
    }

    pub async fn broadcast(&self, socket: &UdpSocket, packet: &Packet) -> io::Result<()> {
        let players = self.players.lock().await;
        for addr in players.keys() {
            send_to_client(socket, addr, packet).await?;
        }
        Ok(())
    }
}




/// FUNCTIONS

/// Handle Packets Function
/// 
/// The function handle_packet is the logic to how the server interacts with recieved packets, these get separated in the categories according to its [PacketType].
/// 
/// # Arguments
/// 
/// - `socket`: Its type is &[Arc<UdpSocket>] so it can be accessed in coroutines, and it represents the current Udp Socket that is recieving packets.
/// - `addr`: Its type is &[SocketAddr] and it represents the ip of the packet sender.
/// - `player_list`: Its type is &[PlayerList] and it represents the player list.
/// - `current_time`: Its type is [u64] and it represents the server timestamp.
///
async fn handle_packet(socket: &Arc<UdpSocket>, addr: &SocketAddr, packet: &Packet, player_list: &PlayerList, current_time: u64) {
    match packet.packet_type {
        PacketType::Chat => {
            if let Some(player) = player_list.get_player(addr).await {
                let message = format!("{}: {}", player.name, packet.content);
                println!("{}", message);
                let _ = player_list.broadcast(socket, &Packet { packet_type: PacketType::Chat, content: message }).await;
            }
        },
        PacketType::Sync => {
            if let Some(player) = player_list.get_player(addr).await {
                let new_player = decode_player_info(&packet.content);
                if player.session_id != new_player.session_id {
                    println!("Session ID mismatch from {}", addr);
                    let _ = send_to_client(socket, addr, &Packet { packet_type: PacketType::Misc, content: "Session Mismatch, Kicked".to_string() }).await;
                    player_list.remove_player(addr).await;
                } else {
                    player_list.update_player(addr, |player| {
                        player.name = new_player.name;
                        player.health = 100;
                        player.state = PlayerState::Alive;
                    }).await;
                    let _ = send_to_client(socket, addr, &Packet { packet_type: PacketType::Sync, content: object_to_json(&player) }).await;
                    let _ = send_to_client(socket, addr, &Packet { packet_type: PacketType::Chat, content: "Welcome to the server".to_string() }).await;
                }
            }

            
        },
        PacketType::Misc => {
            if packet.content == "Pong" {
                player_list.update_player(addr, |player| {
                    player.last_ping_time = current_time;
                }).await;
            }
        }
    }
}




/// Handle New Connections Function
/// 
/// The function handle_new_connection is how the server reacts to new connections.
/// 
/// # Arguments
/// 
/// - `socket`: Its type is &[Arc<UdpSocket>] so it can be accessed in coroutines, and it represents the current Udp Socket that is recieving packets.
/// - `addr`: Its type is &[SocketAddr] and it represents the ip of the packet sender.
/// - `player_list`: Its type is &[PlayerList] and it represents the player list.
/// - `current_time`: Its type is [u64] and it represents the server timestamp.
///
async fn handle_new_connection(socket: &Arc<UdpSocket>, addr: &SocketAddr, packet: &Packet, player_list: &PlayerList, current_time: u64) {
    if is_valid_guid(&packet.content) {
        let _ = send_to_client(socket, addr, &Packet { packet_type: PacketType::Sync, content: "PlayerInfo".to_string() }).await;
        player_list.add_player(*addr, Player { 
            name: "".to_string(), 
            health: 0, 
            session_id: packet.content.clone(), 
            state: PlayerState::Loading,
            last_ping_time: current_time,
        }).await;
        println!("Player from {} just connected to the server", addr);
    } else {
        println!("Packet from {} just got rejected due to not being in the server", addr);
    }
}




/// Is Valid Guid Function
/// 
/// The function is_valid_guid is a function to return true or false if the entered string is a valid guid
/// 
/// # Arguments
/// 
/// - `guid`: Its type is &[str] and its supposed to represent a valid guid.
///
fn is_valid_guid(guid: &str) -> bool {
    let guid_regex = Regex::new(r"^[{(]?([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})[)}]?$").unwrap();
    guid_regex.is_match(guid)
}




/// Send to Client Function
/// 
/// The function send_to_client is a function to send a packet to a certain ip address.
/// 
/// # Arguments
/// 
/// - `socket`: Its type is &[UdpSocket] and it represents a server udp socket.
/// - `addr`: Its type is &[SocketAddr] and it represents the ip address of the client reciever.
/// - `packet`: Its type is &[Packet] and it represents the packet to be sent to the client.
///
async fn send_to_client(socket: &UdpSocket, addr: &SocketAddr, packet: &Packet) -> io::Result<()> {
    let packet = serde_json::to_string(packet)?;
    socket.send_to(packet.as_bytes(), addr).await?;
    Ok(())
}




/// Decode Udp Packet Function
/// 
/// The function decode_udp_packet is a function used to decode a udp packet in &[u8] type into a [Packet] type.
/// 
/// # Arguments
/// 
/// - `bytes`: Its type is &[u8] and it represents information recieved from a Udp Packet.
///
fn decode_udp_packet(bytes: &[u8]) -> Packet {
    serde_json::from_str(str::from_utf8(bytes).unwrap()).unwrap()
}




/// Decode Player Info Function
/// 
/// The function decode_player_info is a function to transform a &[str] into a [Player] object, the string is supposed to be a json sent from a packet.
/// 
/// # Arguments
/// 
/// - `json_player`: Its type is &[str] and its supposed to represents a [Player] type in a json notation.
///
fn decode_player_info(json_player: &str) -> Player {
    serde_json::from_str(json_player).unwrap_or_else(|_| Player {
        name: "".to_string(),
        health: 0,
        session_id: "".to_string(),
        state: PlayerState::Error,
        last_ping_time: 0,
    })
}




/// Object to Json Function
/// 
/// The function object_to_json is a function to transform any type of object into a json [String] object.
/// 
/// # Arguments
/// 
/// - `obj`: Its type is any.
///
fn object_to_json<T: Serialize>(obj: &T) -> String {
    serde_json::to_string(obj).unwrap_or_else(|_| "Error Converting Object to Json".to_string())
}


#[tokio::main]
async fn main() -> io::Result<()> {
    let socket = Arc::new(UdpSocket::bind("0.0.0.0:25565").await?);
    let player_list = PlayerList::new();

    println!("Server is online.");

    // Ping coroutine
    let socket_task = Arc::clone(&socket);
    let player_list_task = player_list.clone();
    
    tokio::spawn(async move {
        let start_time = Instant::now();
        loop {

            let player_list = player_list_task.get_all_players().await;
            let mut to_remove = Vec::new();

            let current_time = start_time.elapsed().as_millis() as u64;

            for (addr, player) in player_list.iter() {
                if current_time - player.last_ping_time > 10000 {
                    println!("Player {}, with the ip {} timed out, kicking.", player.name, addr);
                    to_remove.push(*addr);
                    continue;
                }
                println!("{} Ping: {}", player.name, (current_time - player.last_ping_time));
                let packet = Packet { packet_type: PacketType::Misc, content: "Ping".to_string() };
                if let Err(e) = send_to_client(&socket_task, addr, &packet).await {
                    println!("Error sending to {}: {:?}", addr, e);
                    to_remove.push(*addr);
                }
            }

            for addr in to_remove {
                player_list_task.remove_player(&addr).await;
            }
            
            sleep(Duration::from_secs(5)).await;
        }
    });

    // Main server loop
    let socket_receiver = Arc::clone(&socket);
    let start_time = Instant::now();

    loop {  
        let mut buf = [0; 1024];
        let (len, addr) = match socket_receiver.recv_from(&mut buf).await {
            Ok(data) => data,
            Err(_) => continue,
        };        
        
        let packet = decode_udp_packet(&buf[..len]);

        if player_list.contains_player(&addr).await {
            handle_packet(&socket_receiver, &addr, &packet, &player_list, start_time.elapsed().as_millis() as u64).await;
        } else {
            handle_new_connection(&socket_receiver, &addr, &packet, &player_list, start_time.elapsed().as_millis() as u64).await;
        }
    }
}
