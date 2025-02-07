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
    unprocessed_messages: Arc<Mutex<HashMap<(i32, i32), Vec<(String, Option<i32>)>>>>,
}

impl CommandManager {
    pub fn new(
        data_base: Arc<Mutex<Connection>>,
        ou: Arc<Mutex<Vec<(i32, u16, String)>>>,
        um: Arc<Mutex<HashMap<(i32, i32), Vec<(String, Option<i32>)>>>>,
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
        self.answear.clear();
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
            "show_past_chat" => {
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

        if let Ok(list) = self.online_users.lock() {
            let mut ind = -5;
            for i in list.iter().enumerate() {
                if i.1 .1 == self.user_id {
                    ind = i.0 as i32;
                }
            }
            if ind >= 0 {
                let mes = "You are logged in !";
                self.answear = String::from(mes);
                println!("{mes}");
                return;
            }
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

        if let Ok(list) = self.online_users.lock() {
            let mut ind = -5;
            for i in list.iter().enumerate() {
                if i.1 .1 == self.user_id {
                    ind = i.0 as i32;
                }
            }
            if ind < 0 {
                let mes = "You are not logged in !";
                self.answear = String::from(mes);
                println!("{mes}");
                return;
            }
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
        if self.arguments.len() != 2 {
            let mes = "invalid syntax! Use: <reply> <\"message\"> <message_id>!";
            println!("{}", mes);
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

        let message = self.arguments[0].clone();
        let answeared_message_id_string = self.arguments[1].clone();

        let answeared_id: i32 = match answeared_message_id_string.parse::<i32>() {
            Ok(val) => val,
            Err(_) => -5,
        };
        let mut sender = None;
        if let Ok(list) = self.online_users.lock() {
            for i in list.iter() {
                if i.1 == self.user_id {
                    sender = Some(i.0);
                    break;
                }
            }
        }
        if let Some(sender_id) = sender {
            let mut receiver = None;
            if let Ok(conn) = self.data_base.lock() {
                let mut stmt = conn
                    .prepare("SELECT sender_id FROM message WHERE message_id = ?1;")
                    .unwrap();
                let mut rows = stmt
                    .query_map([answeared_id], |row| row.get::<usize, i32>(0))
                    .unwrap();

                receiver = match rows.next() {
                    Some(row) => match row {
                        Ok(id) => Some(id),
                        Err(_) => None,
                    },
                    None => None,
                };
            }
            if let Some(receiver_id) = receiver {

                if let Ok(m) = &mut self.unprocessed_messages.lock() {
                    let index = (sender_id, receiver_id);
                    m.entry(index)
                        .or_insert_with(Vec::new)
                        .push((message, Some(answeared_id)));
                }
                self.answear = "Reply sent!".to_string();
            } else {
                let mes = "Message you want to reply does not exist !";
                println!("{mes}");
                self.answear = String::from(mes);
                return;
            }
        } else {
            let mes = "You have to sign in !";
            println!("{mes}");
            self.answear = String::from(mes);
        }
    }
    fn send(&mut self) {
        if self.arguments.len() != 2 {
            let mes = "invalid syntax !<send> <\"message\"> <receiver> !";
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
            m.entry(index)
                .or_insert_with(Vec::new)
                .push((message, None));
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
            if let Ok(conn) = self.data_base.lock() {
                let mut rm = Vec::new();
                for i in m.iter() {
                    if i.0 .1 == id {
                        self.answear += "from ";
                        self.answear += format!("{}", i.0 .0).as_str();
                        self.answear.push('\n');

                        for j in i.1 {
                            self.answear += j.0.as_str();
                            if let Some(id) = j.1 {
                                conn.execute("INSERT INTO message (sender_id, receiver_id, content, reply_to) VALUES (?1,?2,?3,?4)", (i.0.0,i.0.1,j.0.clone(),id)).unwrap();
                            } else {
                                conn.execute("INSERT INTO message (sender_id, receiver_id, content) VALUES (?1,?2,?3)", (i.0.0,i.0.1,j.0.clone())).unwrap();
                            }
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
    }
    fn show_past_chat(&mut self) {
        if self.arguments.len() != 1 {
            let mes = "invalid syntax !<show_past_chat> <user> !";
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

        let username = self.arguments[0].clone();

        if let Ok(conn) = self.data_base.lock() {
            let mut stmt = conn
                .prepare("SELECT user_id FROM users WHERE username = ?1")
                .unwrap();
            let mut rows = stmt
                .query_map([username.clone()], |row| row.get::<usize, i64>(0))
                .unwrap();

            let sender_id = match rows.next() {
                Some(s) => match s {
                    Ok(val) => val,
                    Err(_) => -5,
                },
                None => -5,
            };

            let mut receiver_id = -5;
            if let Ok(list) = self.online_users.lock() {
                for i in list.iter() {
                    if i.1 == self.user_id {
                        receiver_id = i.0 as i64;
                        break;
                    }
                }
            }

            self.answear = format!("{}'s chat:\n", username.as_str());

            let mut stmt = conn.prepare("SELECT message_id, content, reply_to FROM message WHERE (sender_id =?1 AND receiver_id =?2) OR (sender_id = ?3 AND receiver_id = ?4);").unwrap();
            let rows = stmt
                .query_map([sender_id, receiver_id, receiver_id, sender_id], |row| {
                    let id = row.get::<usize, i64>(0).unwrap();
                    let name = row.get::<usize, String>(1).unwrap();
                    let reply_to = row.get::<usize, Option<i64>>(2).unwrap();
                    Ok((id, name, reply_to))
                })
                .unwrap();

            for content in rows {
                match content {
                    Ok((id, s, reply_to)) => {
                        self.answear += format!("Message-id({id}").as_str();
                        if let Some(reply) = reply_to {
                            self.answear += format!(" replied to {reply}): ").as_str();
                        } else {
                            self.answear += format!("): ").as_str();
                        }
                        self.answear += s.as_str();
                        self.answear.push('\n');
                    }
                    Err(_) => {}
                }
            }
        }
    }
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
