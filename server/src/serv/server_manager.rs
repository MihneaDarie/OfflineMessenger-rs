use common::{ServerDetails, IP};
use std::{collections::HashMap, sync::Arc};
use tokio::{
    io::{split, AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::{
        mpsc::{channel, Sender},
        Mutex,
    },
    task::spawn_local,
};
use tokio_rusqlite::Connection;

use super::{command_manager::CommandManager, Timer};

const DATA_BASE_SCRIPT_USERS: &str = "CREATE TABLE IF NOT EXISTS users (
                                user_id INTEGER PRIMARY KEY AUTOINCREMENT,
                                first_name TEXT NOT NULL,
                                last_name TEXT NOT NULL,
                                username TEXT NOT NULL,
                                password TEXT NOT NULL
                                );";

const DATA_BASE_SCRIPT_MESSAGE: &str = "CREATE TABLE IF NOT EXISTS message (
                                message_id INTEGER PRIMARY KEY AUTOINCREMENT,
                                sender_id INTEGER NOT NULL,
                                receiver_id INTEGER NOT NULL,
                                content TEXT NOT NULL,
                                reply_to INTEGER,
                                time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                                FOREIGN KEY (sender_id) REFERENCES users(user_id),
                                FOREIGN KEY (receiver_id) REFERENCES users(user_id),
                                FOREIGN KEY (reply_to) REFERENCES message(message_id)
                                );";

pub struct ServerManager {
    server_info: ServerDetails,
    max_clints: u8,
    online_users: Arc<Mutex<Vec<(i32, u16, String)>>>,
    unprocessed_messages: Arc<Mutex<HashMap<(i32, i32), Vec<(String, Option<i32>)>>>>,
    communication_channels: Arc<Mutex<HashMap<u16, Sender<String>>>>,
    command_manager: Arc<Mutex<CommandManager>>,
    timer: Timer,
    cvar: Arc<Mutex<bool>>,
    conn: Arc<Mutex<Connection>>,
}

impl ServerManager {
    pub async fn new(clients_number: u8, ip_address: IP, port: u16) -> Self {
        let c = Arc::new(Mutex::new(Connection::open("server.db").await.unwrap()));
        {
            let db = c.lock().await;

            db.call(|conn| {
                conn.execute(DATA_BASE_SCRIPT_USERS, []).unwrap();
                conn.execute(DATA_BASE_SCRIPT_MESSAGE, []).unwrap();
                Ok(())
            })
            .await
            .unwrap();
        }

        let sd = match ip_address {
            IP::V4(a, b, c, d) => ServerDetails::new_ipv4(a, b, c, d, port),
            IP::V6(a, b, c, d, e, f) => ServerDetails::new_ipv6(a, b, c, d, e, f, port),
        };
        let ou = Arc::new(Mutex::new(Vec::new()));
        let um = Arc::new(Mutex::new(HashMap::new()));
        let cm = Arc::new(Mutex::new(CommandManager::new(
            c.clone(),
            ou.clone(),
            um.clone(),
        )));
        let cc = Arc::new(Mutex::new(HashMap::new()));
        let cv = Arc::new(Mutex::new(false));
        let t = Timer::new(ou.clone(), um.clone(), cc.clone(), c.clone(), cv.clone()).await;
        Self {
            server_info: sd,
            max_clints: clients_number,
            conn: c,
            timer: t,
            command_manager: cm,
            online_users: ou,
            unprocessed_messages: um,
            cvar: cv,
            communication_channels: cc,
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
        self.timer.spawn().await;

        loop {
            if let Ok((sock, addr)) = listener.accept().await {
                let (mut read, write) = split(sock);
                println!("New client connected: {}", addr.port());
                let writer = Arc::new(tokio::sync::Mutex::new(write));

                //we create a channel for each client so it can receive specific messages from the server and insert it
                //in the map
                let (producer, mut receiver) = channel::<String>(500);
                let comm = &mut self.communication_channels.lock().await;
                {
                    comm.insert(addr.port(), producer);
                }

                let command_manager = self.command_manager.clone();
                tokio::spawn(async move {
                    let mut buffer = [0u8; 1024];
                    loop {
                        tokio::select! {
                            
                            msg = receiver.recv() => {
                                if let Some(msg) = msg {
                                    writer.lock().await.write_all(msg.as_bytes()).await.unwrap();
                                }
                            },
                            res = read.read(&mut buffer) => {
                                match res {
                                    Ok(n) if n == 0 => {
                                        println!("Client {} disconnected", addr);
                                        return;
                                    },
                                    Ok(n) => {
                                        if let Ok(input) = std::str::from_utf8(&buffer[..n]) {
                                            let mut cm = command_manager.lock().await;
                                            cm.parse_command(input, addr.port());
                                            cm.identify_command().await;
                                            let resp = cm.get_answer();
                                            writer.lock().await.write_all(resp.as_bytes()).await.unwrap();
                                        }
                                    },
                                    Err(e) => {
                                        eprintln!("Failed to read from client {}: {}", addr, e);
                                        return;
                                    }
                                }
                            }
                        }
                    }
                });
                
            }
            let guard = self.cvar.lock().await;
            {
                if *guard {
                    break;
                }
            }
        }
        println!("Server is down !");
    }
}
