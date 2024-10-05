use std::net::{UdpSocket, SocketAddr};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::str;

fn main() {
    let socket = UdpSocket::bind("127.0.0.1:5001").expect("Could not bind socket");
    let mut player_list: HashMap<SocketAddr, String> = HashMap::new();
    
    let tick_rate = Duration::from_millis(1000/128);
    let mut tick_count = 0;
    let mut last_update = Instant::now();
    let mut start_time = Instant::now();

    println!("Online");
    
    loop {
        let now = Instant::now();
        if now.duration_since(last_update) >= tick_rate {
            tick_count += 1;

            if now.duration_since(start_time) >= Duration::from_secs(1) {
                println!("Ticks in the last second: {}", tick_count);
                tick_count = 0;
                start_time = now;
            }
            
            //Server Code
            let mut buf = [0; 1024];

            let (amt, src) = socket.recv_from(&mut buf).expect("Failed to receive data");
            let message = str::from_utf8(&buf[..amt]).expect("Failed to convert bytes to string");

            if player_list.contains_key(&src) {
                if is_valid_guid(&message){
                    if message != player_list.get(&src).expect("Erro") {
                        println!("Packet from {} just got rejected due to having session mismatch", src );
                        send_to_client(&socket, src, "Session Mismatch");
                        player_list.remove(&src);
                    }
                } else {
                    println!("Received from {}: {}", src, message);
                }
            } else {
                if is_valid_guid(&message) {
                    player_list.insert(src, message.to_string());
                    println!("Player from {} just connected to the server", src);
                    send_to_client(&socket, src, "Welcome to the Server");
                } else {
                    println!("Packet from {} just got rejected due to not being in the server", src)
                }
            }

            last_update = now;
        }
    }
}

fn is_valid_guid(guid: &str) -> bool {
    let guid_regex = regex::Regex::new(r"^[{(]?([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})[)}]?$").unwrap();
    guid_regex.is_match(guid)
}

fn send_to_client(socket: &UdpSocket, src: SocketAddr, message: &str) {
    if let Err(e) = socket.send_to(message.as_bytes(), src) {
        println!("Couldnt send message to {}: {}", src, e);
    }
}