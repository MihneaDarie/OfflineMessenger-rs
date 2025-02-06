use common::{Message, ServerDetails, IP};
use rusqlite::{Connection, Result};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

use super::command_manager::CommandManager;

const DATA_BASE_SCRIPT: &str = "CREATE TABLE IF NOT EXISTS users (
                                user_id INTEGER PRIMARY KEY AUTOINCREMENT,
                                first_name TEXT NOT NULL,
                                last_name TEXT NOT NULL,
                                username TEXT NOT NULL,
                                password TEXT NOT NULL
                                );
                                
                                CREATE TABLE IF NOT EXISTS message (
                                message_id INTEGER PRIMARY KEY AUTOINCREMENT,
                                sender_id INTEGER NOT NULL,
                                receiver_id INTEGER NOT NULL,
                                content TEXT NOT NULL,
                                reply_to INTEGER,
                                is_read BOOLEAN DEFAULT FALSE,
                                time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                                FOREIGN KEY (sender_id) REFERENCES users(user_id),
                                FOREIGN KEY (receiver_id) REFERENCES users(receiver_id),
                                FOREIGN KEY (reply_to) REFERENCES message(message_id)
                                )";

pub struct ServerManager {
    server_info: ServerDetails,
    max_clints: u8,
    online_users: Arc<Mutex<Vec<(i32,u16,String)>>>,
    unprocessed_messages: Arc<Mutex<HashMap<(i32, i32), Vec<String>>>>,
    command_manager: Arc<Mutex<CommandManager>>,
    conn: Arc<Mutex<Connection>>,
}

impl ServerManager {
    pub fn new(clients_number: u8, ip_address: IP, port: u16) -> Self {
        let c = Arc::new(Mutex::new(Connection::open("server.db").unwrap()));
        if let Ok(db) = c.lock() {
            db.execute(DATA_BASE_SCRIPT, ()).unwrap();
        }
        let sd = match ip_address {
            IP::V4(a, b, c, d) => ServerDetails::new_ipv4(a, b, c, d, port),
            IP::V6(a, b, c, d, e, f) => ServerDetails::new_ipv6(a, b, c, d, e, f, port),
        };
        let ou = Arc::new(Mutex::new(Vec::new()));
        let um = Arc::new(Mutex::new(HashMap::new()));
        let cm = Arc::new(Mutex::new(CommandManager::new(c.clone(),ou.clone(),um.clone())));

        Self {
            server_info: sd,
            max_clints: clients_number,
            conn: c,
            command_manager: cm,
            online_users: ou,
            unprocessed_messages: um,
        }
    }

    pub async fn run(&self) {
        let adress = format!(
            "{}:{}",
            self.server_info.ip_to_string().unwrap().as_str(),
            self.server_info.port_to_string().unwrap().as_str()
        );
        let listener = TcpListener::bind(adress).await.unwrap();
        println!("Server listening !4");

        loop {
            if let Ok((mut sock, addr)) = listener.accept().await {
                println!("New client connected: {}", addr);
                let command_manager = self.command_manager.clone();
                tokio::spawn(async move {
                    let mut buffer = [0u8; 1024];
                    let mut len = 0;
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
                            if let Ok(m) = &mut command_manager.lock() {
                                m.parse_command(mes,addr.port());
                                m.identify_command();
                                let answear = m.get_answear().as_bytes();
                                len = answear.len();
                                let mut ct = 0;
                                for i in answear {
                                    buffer[ct] = *i;
                                    ct += 1;
                                }
                            }
                        } else {
                            println!("Couldn't ")
                        }
                        if let Err(e) = sock.write_all(&buffer[..len]).await {
                            eprintln!("Failed to write to client {}: {}", addr, e);
                            return;
                        }
                    }
                });
            }
        }
    }
}
