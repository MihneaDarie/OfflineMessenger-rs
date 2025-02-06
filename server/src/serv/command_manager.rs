use std::{
    clone,
    collections::HashMap,
    sync::{Arc, Mutex},
};

use common::Message;
use rusqlite::Connection;
use tokio::task::spawn_blocking;

pub struct CommandManager {
    command: String,
    arguments: Vec<String>,
    user_id: u16,
    answear: String,

    data_base: Arc<Mutex<Connection>>,
    online_users: Arc<Mutex<Vec<(i32, u16, String)>>>,
    unprocessed_messages: Arc<Mutex<HashMap<(i32, i32), Vec<String>>>>,
}

impl CommandManager {
    pub fn new(
        data_base: Arc<Mutex<Connection>>,
        ou: Arc<Mutex<Vec<(i32, u16, String)>>>,
        um: Arc<Mutex<HashMap<(i32, i32), Vec<String>>>>,
    ) -> Self {
        Self {
            command: String::default(),
            arguments: Vec::new(),
            answear: String::default(),
            data_base: data_base.clone(),
            online_users: ou,
            unprocessed_messages: um,
            user_id: 0,
        }
    }

    pub fn parse_command(&mut self, input: &str, id: u16) {
            self.arguments.clear();
            self.command.clear();
            self.user_id = id;
        
            let mut words = Vec::new();
            let mut inside_quotes = false;
            let mut current_arg = String::new();
        
            let chars: Vec<char> = input.chars().collect();
            let mut i = 0;
        
            while i < chars.len() {
                if chars[i] == '"' {
                    inside_quotes = !inside_quotes;
                    i += 1;
                    continue;
                }
        
                if chars[i].is_whitespace() && !inside_quotes {
                    if !current_arg.is_empty() {
                        words.push(current_arg.clone());
                        current_arg.clear();
                    }
                } else {
                    current_arg.push(chars[i]);
                }
        
                i += 1;
            }
        
            if !current_arg.is_empty() {
                words.push(current_arg.clone());
            }
        
            if inside_quotes {
                let mes = "Didn't close string with \'\"\' !";
                println!("{mes}");
            }
        
            if words.is_empty() {
                return;
            }
        
            self.command = words[0].clone();
            self.arguments = words[1..].to_vec();
        
    }
    pub fn print(&self) {
        println!("{}", self.command);
        for i in self.arguments.iter() {
            print!("{}|", i);
        }
        println!("");
    }
    pub fn identify_command(&mut self) {
        match self.command.as_str() {
            "sign_in" => {
                self.sign_in();
            }
            "sign_up" => {
                self.sign_up();
            }
            "sign_out" => {
                self.sign_out();
            }
            "reply" => {
                self.reply();
            }
            "send" => {
                self.send();
            }
            "show_past_chat_with" => {
                self.show_past_chat();
            }
            "show_users" => {
                self.show_users();
            }
            "check_inbox" => {
                self.check_inbox();
            }
            _ => {
                self.invalid();
            }
        }
    }
    fn sign_in(&mut self) {
        if self.arguments.len() != 2 {
            let mes = "invalid syntax !<sign_in> <username> <password> !";
            println!("{mes}");
            self.answear = String::from(mes);
            return;
        }

        let username = self.arguments[0].clone();
        let password = self.arguments[1].clone();

        if let Ok(conn) = self.data_base.lock() {
            let mut stmt = conn
                .prepare("SELECT user_id, password, username from users where username = ?1")
                .unwrap();

            let mut rows = stmt
                .query_map([username.clone()], |row| {
                    let id: i32 = row.get(0).unwrap();
                    let pass: String = row.get(1).unwrap();
                    let usr: String = row.get(2).unwrap();
                    Ok((id, pass, usr))
                })
                .unwrap();

            if let Some(result) = match rows.next() {
                Some(Ok((id, pass, usr))) => Some((id, pass, usr)),
                None => None,
                Some(e) => {
                    println!("Error at printing row: {:?} !", e);
                    None
                }
            } {
                if result.1 == password {
                    if let Ok(list) = &mut self.online_users.lock() {
                        list.push((result.0, self.user_id, result.2));
                    }

                    let mes = "Hello ";
                    println!("{}", mes);
                    self.answear = String::from(mes) + username.as_str();
                    return;
                } else {
                    let mes = "Missmatched password !";
                    println!("{}", mes);
                    self.answear = String::from(mes);
                    return;
                }
            }
            self.answear = String::from("Couldn't execute command");
            println!("{}", self.answear);
        }
    }
    fn sign_out(&mut self) {
        if self.arguments.len() != 0 {
            let mes = "invalid syntax !<sign_out> !";
            println!("{mes}");
            self.answear = String::from(mes);
            return;
        }

        if let Ok(list) = &mut self.online_users.lock() {
            let mut ind: i32 = -5;
            for i in list.iter().enumerate() {
                if i.1 .1 == self.user_id {
                    ind = i.0 as i32;
                }
            }
            if ind >= 0 {
                list.swap_remove(ind as usize);
            } else {
                let mes = "User is not online !";
                println!("{mes}");
                self.answear = String::from(mes);
                return;
            }
        }
        let mes = "Logged out !";
        println!("{mes}");
        self.answear = String::from(mes);
    }
    fn sign_up(&mut self) {
        if self.arguments.len() != 4 {
            let mes = "invalid syntax !<sign_up> <first_name> <last_name> <username> <password> !";
            println!("{mes}");
            self.answear = String::from(mes);
            return;
        }

        let first_name = self.arguments[0].clone();
        let last_name = self.arguments[1].clone();
        let username = self.arguments[2].clone();
        let password = self.arguments[3].clone();

        let db_conn = self.data_base.clone();
        let conn = db_conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT user_id FROM users WHERE username = ?1")
            .unwrap();
        let mes;
        let mut rows = stmt
            .query_map([username.clone()], |row| row.get::<usize, i64>(0))
            .unwrap();

        let user_exists = match rows.next() {
            Some(Ok(_)) => true,
            None => false,
            Some(Err(e)) => {
                eprintln!("Error reading row: {e}");
                false
            }
        };
        if user_exists {
            mes = "User already exists !";
        } else {
            conn.execute(
                "INSERT INTO users (first_name,last_name,username,password) VALUES (?1,?2,?3,?4)",
                [first_name, last_name, username, password],
            )
            .unwrap();
            mes = "User created !";
            println!("{mes}");
        }

        self.answear = String::from(mes);
    }
    fn reply(&mut self) {
        self.answear = "reply".to_string();
    }
    fn send(&mut self) {
        if self.arguments.len() != 2 {
            let mes = "invalid syntax !<send> <\"message\"> <receiver> !";
            println!("{mes}");
            self.answear = String::from(mes);
            return;
        }
        let mut sender_id = -5;
        if let Ok(list) = self.online_users.lock() {
            for i in list.iter() {
                if i.1 == self.user_id {
                    sender_id = i.0;
                    break;
                }
            }
        }
        let message = self.arguments[0].clone();
        let username = self.arguments[1].clone();

        let receiver_id = if let Ok(conn) = self.data_base.lock() {
            let mut stmt = conn
                .prepare("SELECT user_id FROM users WHERE username = ?1;")
                .unwrap();

            stmt.query_row([username.clone()], |row| row.get::<usize, i32>(0))
                .unwrap_or(-5)
        } else {
            -5
        };

        if receiver_id == -5 {
            let mes = "User does not exists !";
            println!("{mes}");
            self.answear = String::from(mes);
            return;
        }

        if let Ok(m) = &mut self.unprocessed_messages.lock() {
            let index = (sender_id, receiver_id);
            m.entry(index).or_insert_with(Vec::new).push(message);
        }
        println!("Message sent !");
        self.answear = String::from("Message sent !");
    }
    fn invalid(&mut self) {
        self.answear = "invalid".to_string();
    }
    fn check_inbox(&mut self) {
        if self.arguments.len() != 0 {
            let mes = "invalid syntax !<check_inbox> !";
            println!("{mes}");
            self.answear = String::from(mes);
            return;
        }

        let mut id = -5;
        if let Ok(list) = self.online_users.lock() {
            for i in list.iter() {
                if i.1 == self.user_id {
                    id = i.0;
                    break;
                }
            }
        }
        self.answear = String::from("Unread messages:\n");
        if let Ok(m) = &mut self.unprocessed_messages.lock() {
            let mut rm = Vec::new();
            for i in m.iter() {
                if i.0 .1 == id {
                    self.answear += "from ";
                    self.answear += format!("{}", i.0 .0).as_str();
                    self.answear.push('\n');

                    for j in i.1 {
                        self.answear += j.as_str();
                        self.answear.push('\n');
                    }
                    rm.push(*i.0);
                }
            }

            for i in &rm {
                m.remove_entry(i);
            }
        }
    }

    fn show_past_chat(&mut self) {}

    fn show_users(&mut self) {

        if self.arguments.len() != 0 {
            let mes = "invalid syntax !<show_users> !";
            println!("{mes}");
            self.answear = String::from(mes);
            return;
        }

        if let Ok(list) = self.online_users.lock() {
            let mut ind = -5;
            for i in list.iter().enumerate() {
                if i.1 .1 == self.user_id {
                    ind = i.0 as i32;
                }
            }
            if ind < 0 {
                let mes = "Sign in first !";
                self.answear = String::from(mes);
                println!("{mes}");
                return;
            }
        }

        self.answear = String::from("Online users:\n");
        if let Ok(list) = self.online_users.lock() {
            for i in list.iter() {
                self.answear.push_str(i.2.as_str());
                self.answear.push('\n');
            }
        }
    }

    pub fn get_answear(&self) -> &str {
        self.answear.as_str()
    }
}
