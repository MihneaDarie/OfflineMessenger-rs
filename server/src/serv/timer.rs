use std::{collections::HashMap, sync::Arc, time::Duration};

use tokio::{
    sync::{mpsc::Sender, Mutex},
    task::{spawn, JoinHandle},
    time::sleep,
};

use tokio_rusqlite::Connection;

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
        exit: Arc<Mutex<bool>>,
    ) -> Self {
        Self {
            online_users: ou.clone(),
            unprocessed_messages: um.clone(),
            communication_channels: cc.clone(),
            data_base: db.clone(),
            exit: exit.clone(),
        }
    }

    pub async fn spawn(&self) {
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

                let users = ou.lock().await;
                let mut messages = um.lock().await;
                let channels = cc.lock().await;

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
                                let conn = db.lock().await;
                                let mes_cloned = mes.clone();
                                let sender_val = *sender;
                                let receiver_val = *receiver;
                                let content_owned = content.clone();
                                let rep_copied = rep.clone();
                                
                                let new_mes: String = conn
                                    .call(move |conn| {
                                        let mut mes = mes_cloned;
                                        if let Some(reply_to) = rep_copied {
                                            mes += &format!("replied to {}: ", reply_to);
                                            mes += &content_owned;
                                            mes.push('\n');
                                            conn.execute(
                                                "INSERT INTO message (sender_id, receiver_id, content, reply_to) VALUES (?1, ?2, ?3, ?4)",
                                                (sender_val, receiver_val, content_owned.clone(), reply_to),
                                            )
                                            .unwrap();
                                        } else {
                                            mes += &content_owned;
                                            conn.execute(
                                                "INSERT INTO message (sender_id, receiver_id, content) VALUES (?1, ?2, ?3)",
                                                (sender_val, receiver_val, content_owned.clone()),
                                            )
                                            .unwrap();
                                            mes.push('\n');
                                        }
                                        Ok(mes)
                                    })
                                    .await
                                    .unwrap();
                                mes = new_mes;
                                rm.push((sender_val, receiver_val));
                            }
                            if let Err(e) = send.send(mes.clone()).await {
                                println!("Couldn't send message: {:?}", e);
                            } else {
                                println!("Messages sent!!!!!!!!!!");
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
        });
    }
}
