use std::{
    collections::HashMap,
    sync::{mpsc::Sender, Arc, Condvar, Mutex},
    thread::{spawn, JoinHandle},
    time::Duration,
};

use rusqlite::Connection;

pub struct Timer {
    online_users: Arc<Mutex<Vec<(i32, u16, String)>>>,
    unprocessed_messages: Arc<Mutex<HashMap<(i32, i32), Vec<(String, Option<i32>)>>>>,
    communication_channels: Arc<Mutex<HashMap<u16, Sender<String>>>>,
    data_base: Arc<Mutex<Connection>>,
    cvar: Arc<(Mutex<bool>, Condvar)>,
}

impl Timer {
    pub fn new(
        ou: Arc<Mutex<Vec<(i32, u16, String)>>>,
        um: Arc<Mutex<HashMap<(i32, i32), Vec<(String, Option<i32>)>>>>,
        cc: Arc<Mutex<HashMap<u16, Sender<String>>>>,
        db: Arc<Mutex<Connection>>,
        cv: Arc<(Mutex<bool>, Condvar)>,
    ) -> Self {
        Self {
            online_users: ou.clone(),
            unprocessed_messages: um.clone(),
            communication_channels: cc.clone(),
            data_base: db.clone(),
            cvar: cv.clone(),
        }
    }

    pub fn spawn(&self) -> JoinHandle<()> {
        let ou = self.online_users.clone();
        let um = self.unprocessed_messages.clone();
        let cc = self.communication_channels.clone();
        let db = self.data_base.clone();
        let cv = self.cvar.clone();

        spawn(move || {
            let mut guard = cv.0.lock().unwrap();
            if *guard {
                println!("Exit from timer !");
                return;
            }
            loop {
                println!("Trying to send messages on a 5 sec interval !");
                let (new_guard, timeout) =
                    cv.1.wait_timeout_while(guard, Duration::from_secs(5), |stop| !*stop)
                        .unwrap();
                guard = new_guard;
                if timeout.timed_out() {
                    if let (Ok(users), Ok(messages), Ok(channels), Ok(conn)) = (
                        &mut ou.lock(),
                        &mut um.lock(),
                        &mut cc.lock(),
                        &mut db.lock(),
                    ) {
                        let mut rm = Vec::new();
                        for ((sender, receiver), values) in messages.iter() {
                            let mut client = None;
                            for (user_id, client_id, name) in users.iter() {
                                if user_id == receiver {
                                    client = Some(*client_id);
                                    rm.push((*sender, *receiver));
                                    break;
                                }
                            }
                            if let Some(client_id) = client {
                                if let Some(send) = channels.get(&client_id) {
                                    let mut mes = format!("Message from {}:\n", sender);
                                    for (content, rep) in values.iter() {
                                        if let Some(reply_to) = rep {
                                            mes += format!("replied to {}: ", reply_to).as_str();
                                            mes += content.as_str();
                                            mes.push('\n');
                                            conn.execute("INSERT INTO message (sender_id, receiver_id, content, reply_to) VALUES (?1,?2,?3,?4)", (sender,receiver,content.clone(),reply_to)).unwrap();
                                        } else {
                                            mes += content.as_str();
                                            conn.execute("INSERT INTO message (sender_id, receiver_id, content) VALUES (?1,?2,?3)", (sender,receiver,content.clone())).unwrap();
                                            mes.push('\n');
                                        }
                                    }
                                    if let Ok(_) = send.send(mes) {
                                        println!("Messages sent !")
                                    } else {
                                        println!("Couldn't send message !");
                                    }
                                } else {
                                    println!("Couldn't extract communication channel !");
                                }
                            } else {
                                println!("Couldn't find receiver id !");
                            }
                        }
                        for index in rm {
                            messages.remove(&index);
                        }
                    }

                    break;
                } else if *guard {
                    println!("Exit from timer !");
                    break;
                }
            }
        })
    }
}
