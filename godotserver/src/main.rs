use std::net::SocketAddr;
use std::thread::sleep;
use tokio::net::UdpSocket;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::str;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:5001").await?;
    let mut player_list: HashMap<SocketAddr, String> = HashMap::new();
    
    let tick_rate = Duration::from_millis(1000/128);
    let mut tick_count = 0;
    let mut last_update = Instant::now();
    let mut start_time = Instant::now();

    println!("Online");

    tokio::spawn(async {
        loop {
            monitor_players().await;
            sleep(Duration::from_secs(3));
        }
    });
    
    loop {
        //tick per second calculator
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

            let (len, addr) = socket.recv_from(&mut buf).await?;
            let message = str::from_utf8(&buf[..len]).expect("Failed to convert bytes to string");

            //check if the address is in the playerlist
            if player_list.contains_key(&addr) {

                //check if the sesssion is the same
                if is_valid_guid(&message){
                    if message != player_list.get(&addr).expect("Erro") {
                        println!("Packet from {} just got rejected and kicked due to having session mismatch", addr );
                        send_to_client(&socket, &addr, "Session Mismatch").await;
                        player_list.remove(&addr);
                    }
                } else {
                    println!("Received from {}: {}", addr, message);
                }
            } else {
                //check if message is a guid to connect to the server
                if is_valid_guid(&message) {
                    player_list.insert(addr, message.to_string());
                    println!("Player from {} just connected to the server", addr);
                    send_to_client(&socket, &addr, "Welcome to the Server").await;
                } else {
                    println!("Packet from {} just got rejected due to not being in the server", addr)
                }
            }

            last_update = now;
        }
    }
}

async fn monitor_players() {
    println!("Function is runnning")
}

fn is_valid_guid(guid: &str) -> bool {
    let guid_regex = regex::Regex::new(r"^[{(]?([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})[)}]?$").unwrap();
    guid_regex.is_match(guid)
}

async fn send_to_client(socket: &UdpSocket, addr: &SocketAddr, message: &str) {
    if let Err(e) = socket.send_to(message.as_bytes(), addr).await {
        println!("Couldnt send message to {}: {}", addr, e);
    }
}