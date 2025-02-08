use common::{ServerDetails,IP};
use std::io::stdin;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
pub struct ClientManager {
    server_info: ServerDetails,
}

impl ClientManager {
    pub fn new(ip_address: IP, port: u16) -> Self {
        let sd = match ip_address {
            IP::V4(a, b, c, d) => ServerDetails::new_ipv4(a, b, c, d, port),
            IP::V6(a, b, c, d, e, f) => ServerDetails::new_ipv6(a, b, c, d, e, f, port),
        };
        Self {
            server_info: sd,
        }
    }

    pub async fn run(&self) {
        let address = format!(
            "{}:{}",
            self.server_info.ip_to_string().unwrap(),
            self.server_info.port_to_string().unwrap()
        );
        let mut stream = TcpStream::connect(address).await;
        let mut buffer = [0u8; 1024];

        if let Ok(s) = &mut stream {
            println!("Connected to the server!");
            loop {
                let mut user_input = String::from("");
                match stdin().read_line(&mut user_input) {
                    Ok(_) => {
                        let send = user_input.trim_end().as_bytes();
                        if let Ok(()) = s.write_all(send).await {
                            println!("Message sent !");
                        }
                    }
                    Err(_) => {
                        println!("Invalid message from stdin");
                    }
                }
                if let Ok(n) = s.read(&mut buffer).await {
                    let answear = String::from_utf8_lossy(&buffer[..n]);
                    println!(
                        "Received from server: {}",
                        answear
                    );
                    if answear == "exit!" {
                        break;
                    }
                }
            }
        } else {
            println!("Couldn't connect !");
        }
    }
}
