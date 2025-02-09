use std::{collections::HashMap, sync::Arc, time::Duration};

use tokio::{
    sync::{mpsc::Sender, Mutex},
    task::{spawn, JoinHandle},
    time::sleep,
};

use rusqlite::Connection;

pub struct Timer {
    online_users: Arc<Mutex<Vec<(i32, u16, String)>>>,
    unprocessed_messages: Arc<Mutex<HashMap<(i32, i32), Vec<(String, Option<i32>)>>>>,
    communication_channels: Arc<Mutex<HashMap<u16, Sender<String>>>>,
    data_base: Arc<Mutex<Connection>>,
    exit: Arc<Mutex<bool>>,
}

impl Timer {
    pub async fn new(
        ou: Arc<Mutex<Vec<(i32, u16, String)>>>,
        um: Arc<Mutex<HashMap<(i32, i32), Vec<(String, Option<i32>)>>>>,
        cc: Arc<Mutex<HashMap<u16, Sender<String>>>>,
        db: Arc<Mutex<Connection>>,
        exit_flag: Arc<Mutex<bool>>,
    ) -> Self {
        Self {
            online_users: ou.clone(),
            unprocessed_messages: um.clone(),
            communication_channels: cc.clone(),
            data_base: db.clone(),
            exit: exit_flag.clone(),
        }
    }

    pub async fn spawn(&self) -> JoinHandle<()> {
        let ou = self.online_users.clone();
        let um = self.unprocessed_messages.clone();
        let cc = self.communication_channels.clone();
        let db = self.data_base.clone();
        let exit = self.exit.clone();

        spawn(async move {
            loop {
                {
                    let flag = exit.lock().await;
                    if *flag {
                        println!("Exit from timer!");
                        break;
                    }
                }

                println!("Trying to send messages on a 5 sec interval!");
                sleep(Duration::from_secs(5)).await;

                let (users, mut messages, channels, conn) = (
                    ou.lock().await,
                    um.lock().await,
                    cc.lock().await,
                    db.lock().await,
                );
                {
                    let mut rm = Vec::new();
                    for ((sender, receiver), values) in messages.iter() {
                        if let Some(client_id) = users
                            .iter()
                            .find(|(user_id, _, _)| user_id == sender)
                            .map(|(_, client, _)| *client)
                        {
                            if let Some(send) = channels.get(&client_id) {
                                let mut mes = format!("Message from {}:\n", sender);
                                for (content, rep) in values.iter() {
                                    if let Some(reply_to) = rep {
                                        mes += &format!("replied to {}: ", reply_to);
                                        mes += content;
                                        mes.push('\n');
                                        conn.execute(
                                            "INSERT INTO message (sender_id, receiver_id, content, reply_to) VALUES (?1,?2,?3,?4)",
                                            (sender, receiver, content.clone(), reply_to),
                                        ).unwrap();
                                    } else {
                                        mes += content;
                                        conn.execute(
                                            "INSERT INTO message (sender_id, receiver_id, content) VALUES (?1,?2,?3)",
                                            (sender, receiver, content.clone()),
                                        ).unwrap();
                                        mes.push('\n');
                                    }
                                    rm.push((*sender, *receiver));
                                }
                                if let Err(e) = send.send(mes).await {
                                    println!("Couldn't send message: {:?}", e);
                                } else {
                                    println!("Messages sent!");
                                }
                            } else {
                                println!("Couldn't extract communication channel!");
                            }
                        } else {
                            println!("Couldn't find receiver id!");
                        }
                    }
                    for index in rm {
                        messages.remove(&index);
                    }
                }
            }
        })
    }
}
