use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};
use rusqlite::{Connection, Result};
use std::sync::{Arc, Mutex, RwLock};
use common::{IP,ServerDetails};

pub struct ServerManager {
    server_info: ServerDetails,
    max_clints: u8,
    conn: Arc<Mutex<Connection>>,
}

impl ServerManager {
    pub fn new(clients_number: u8, ip_address: IP, port: u16) -> Self {
        let c = Arc::new(Mutex::new(Connection::open("server.db").unwrap()));
        let sd = match ip_address {
            IP::V4(a, b, c, d) => ServerDetails::new_ipv4(a, b, c, d, port),
            IP::V6(a, b, c, d, e, f) => ServerDetails::new_ipv6(a, b, c, d, e, f, port),
        };
        Self {
            server_info: sd,
            max_clints: clients_number,
            conn: c,
        }
    }

    pub async fn run(&self) {
        let adress = format!("{}:{}",self.server_info.ip_to_string().unwrap().as_str(),self.server_info.port_to_string().unwrap().as_str());
        let listener = TcpListener::bind(adress).await.unwrap();
        println!("Server listening on IPv4 127.0.0.1:2098");

        loop {
            if let Ok((mut sock, addr)) = listener.accept().await {
                println!("New client connected: {}", addr);

                tokio::spawn(async move {
                    let mut buffer = [0u8; 1024];
                    loop {
                        let n = match sock.read(&mut buffer).await {
                            Ok(n) if n == 0 => {
                                println!("Client {} disconnected", addr);
                                return;
                            }
                            Ok(n) => n,
                            Err(e) => {
                                eprintln!("Failed to read from client {}: {}", addr, e);
                                return;
                            }
                        };
                        if let Ok(mes) = std::str::from_utf8(&buffer[..n]) {
                            println!("Message received <{}>", mes);
                        } else {
                            println!("Couldn't ")
                        }

                        if let Err(e) = sock.write_all(&buffer[..n]).await {
                            eprintln!("Failed to write to client {}: {}", addr, e);
                            return;
                        }
                    }
                });
            }
        }
    }
}
