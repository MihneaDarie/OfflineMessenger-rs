use std::{
    collections::HashMap,
    sync::{mpsc::Sender, Arc, Condvar, Mutex},
    thread::{spawn, JoinHandle},
    time::Duration,
};

pub struct Timer {
    online_users: Arc<Mutex<Vec<(i32, u16, String)>>>,
    unprocessed_messages: Arc<Mutex<HashMap<(i32, i32), Vec<(String, Option<i32>)>>>>,
    communication_channels: Arc<Mutex<HashMap<u16, Sender<String>>>>,
    cvar: Arc<(Mutex<bool>, Condvar)>,
}

impl Timer {
    pub fn new(
        ou: Arc<Mutex<Vec<(i32, u16, String)>>>,
        um: Arc<Mutex<HashMap<(i32, i32), Vec<(String, Option<i32>)>>>>,
        cc: Arc<Mutex<HashMap<u16, Sender<String>>>>,
        cv: Arc<(Mutex<bool>, Condvar)>,
    ) -> Self {
        Self {
            online_users: ou.clone(),
            unprocessed_messages: um.clone(),
            communication_channels: cc.clone(),
            cvar: cv.clone(),
        }
    }

    pub fn spawn(&self) -> JoinHandle<()> {
        let ou = self.online_users.clone();
        let um = self.unprocessed_messages.clone();
        let cc = self.communication_channels.clone();
        let cv = self.cvar.clone();

        spawn(move || {
            let mut guard = cv.0.lock().unwrap();
            if *guard {
                println!("Exit from timer !");
                return;
            }
            loop {
                let (new_guard, timeout) =
                    cv.1.wait_timeout_while(guard, Duration::from_secs(5), |stop| !*stop)
                        .unwrap();
                guard = new_guard;
                if timeout.timed_out() {

                    

                    break;
                } else if *guard {
                    println!("Exit from timer !");
                    break;
                }
            }
        })
    }
}
