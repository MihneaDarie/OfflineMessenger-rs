use common::{Message, ServerDetails, IP};
use std::io::stdin;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub struct ClientManager {
    server_info: ServerDetails,
    message: Message,
}

impl ClientManager {
    pub fn new(ip_address: IP, port: u16) -> Self {
        let sd = match ip_address {
            IP::V4(a, b, c, d) => ServerDetails::new_ipv4(a, b, c, d, port),
            IP::V6(a, b, c, d, e, f) => ServerDetails::new_ipv6(a, b, c, d, e, f, port),
        };
        Self {
            server_info: sd,
            message: Message::default(),
        }
    }

    pub async fn run(&self) {
        let mut stream = TcpStream::connect("127.0.0.1:2098").await;
        let mut buffer = [0u8; 1024];

        if let Ok(s) = &mut stream {
            println!("Connected to the server!");
            loop {
                let mut received = String::from("");
                match stdin().read_line(&mut received) {
                    Ok(_) => {
                        println!("{}", received);
                        if let Ok(()) = s.write_all(received.as_bytes()).await {
                            println!("Message sent !");
                        }
                    }
                    Err(_) => {
                        println!("Invalid message from stdin");
                    }
                }
                if let Ok(n) = s.read(&mut buffer).await {
                    println!(
                        "Received from server: {}",
                        String::from_utf8_lossy(&buffer[..n])
                    );
                }
            }
        } else {
            println!("Couldn't connect !");
        }
    }
}
