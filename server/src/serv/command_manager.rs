use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;
use tokio_rusqlite::Connection;

pub struct CommandManager {
    command: String,
    arguments: Vec<String>,
    user_id: u16,
    answer: String,
    data_base: Arc<Mutex<Connection>>,
    online_users: Arc<Mutex<Vec<(i32, u16, String)>>>,
    unprocessed_messages: Arc<Mutex<HashMap<(i32, i32), Vec<(String, Option<i32>)>>>>,
}

impl CommandManager {
    pub fn new(
        data_base: Arc<Mutex<Connection>>,
        online_users: Arc<Mutex<Vec<(i32, u16, String)>>>,
        unprocessed_messages: Arc<Mutex<HashMap<(i32, i32), Vec<(String, Option<i32>)>>>>,
    ) -> Self {
        Self {
            command: String::default(),
            arguments: Vec::new(),
            answer: String::default(),
            data_base: data_base.clone(),
            online_users,
            unprocessed_messages,
            user_id: 0,
        }
    }

    pub fn parse_command(&mut self, input: &str, id: u16) {
        self.arguments.clear();
        self.command.clear();
        self.answer.clear();
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
        if words.is_empty() {
            return;
        }
        self.command = words[0].clone();
        self.arguments = words[1..].to_vec();
    }

    pub fn print(&self) {
        println!("{}", self.command);
        for arg in self.arguments.iter() {
            print!("{}|", arg);
        }
        println!();
    }

    pub async fn identify_command(&mut self) {
        match self.command.as_str() {
            "sign_in" => {
                self.sign_in().await;
            }
            "sign_up" => {
                self.sign_up().await;
            }
            "sign_out" => {
                self.sign_out().await;
            }
            "reply" => {
                self.reply().await;
            }
            "send" => {
                self.send().await;
            }
            "show_past_chat" => {
                self.show_past_chat().await;
            }
            "show_users" => {
                self.show_users().await;
            }
            "check_inbox" => {
                self.check_inbox().await;
            }
            "exit" => {
                self.exit().await;
            }
            _ => {
                self.invalid();
            }
        }
    }

    async fn sign_in(&mut self) {
        if self.arguments.len() != 2 {
            self.answer = "invalid syntax !<sign_in> <username> <password> !".to_string();
            println!("{}", self.answer);
            return;
        }

        {
            let users = self.online_users.lock().await;
            if users.iter().any(|(_, client, _)| *client == self.user_id) {
                self.answer = "You are logged in !".to_string();
                println!("{}", self.answer);
                return;
            }
        }
        
        let username = self.arguments[0].clone();
        let password = self.arguments[1].clone();
        let conn_clone = {
            let db_guard = self.data_base.lock().await;
            db_guard.clone()
        };
        let query_result: Result<(i32, String, String), _> = conn_clone
            .call(move |conn| {
                let mut stmt = conn
                    .prepare("SELECT user_id, password, username FROM users WHERE username = ?1")
                    .unwrap();
                let mut rows = stmt
                    .query_map([username.clone()], |row| {
                        let id: i32 = row.get(0).unwrap();
                        let pass: String = row.get(1).unwrap();
                        let usr: String = row.get(2).unwrap();
                        Ok((id, pass, usr))
                    })
                    .unwrap();
                if let Some(Ok((id, pass, usr))) = rows.next() {
                    Ok((id, pass, usr))
                } else {
                    Ok((0, "Couldn't find user !".to_string(), "idk".to_string()))
                }
            })
            .await;
        match query_result {
            Ok((id, pass, name)) if pass == password => {
                {
                    let mut users = self.online_users.lock().await;
                    users.push((id, self.user_id, name));
                }
                self.answer = format!("Hello {}", self.arguments[0]);
                println!("{}", self.answer);
            }
            Ok(_) => {
                self.answer = "Mismatched password !".to_string();
                println!("{}", self.answer);
            }
            Err(_) => {
                self.answer = "Couldn't execute command".to_string();
                println!("{}", self.answer);
            }
        }
    }

    async fn sign_out(&mut self) {
        if self.arguments.len() != 0 {
            self.answer = "invalid syntax !<sign_out> !".to_string();
            println!("{}", self.answer);
            return;
        }
        {
            let users = self.online_users.lock().await;
            if !users.iter().any(|(_, client, _)| *client == self.user_id) {
                self.answer = "You are not logged in !".to_string();
                println!("{}", self.answer);
                return;
            }
        }
        {
            let mut users = self.online_users.lock().await;
            if let Some(pos) = users
                .iter()
                .position(|(_, client, _)| *client == self.user_id)
            {
                users.swap_remove(pos);
            } else {
                self.answer = "User is not online !".to_string();
                println!("{}", self.answer);
                return;
            }
        }
        self.answer = "Logged out !".to_string();
        println!("{}", self.answer);
    }

    async fn sign_up(&mut self) {
        if self.arguments.len() != 4 {
            self.answer =
                "invalid syntax !<sign_up> <first_name> <last_name> <username> <password> !"
                    .to_string();
            println!("{}", self.answer);
            return;
        }
        let first_name = self.arguments[0].clone();
        let last_name = self.arguments[1].clone();
        let username = self.arguments[2].clone();
        let password = self.arguments[3].clone();
        let db_conn = self.data_base.clone();
        let conn = db_conn.lock().await;
        let result = conn.call(move |conn| {
            let mut stmt = conn.prepare("SELECT user_id FROM users WHERE username = ?1").unwrap();
            let mut rows = stmt.query_map([username.clone()], |row| row.get::<usize, i64>(0)).unwrap();
            let user_exists = match rows.next() {
                Some(Ok(_)) => true,
                None => false,
                Some(Err(_)) => false,
            };
            if user_exists {
                Ok("User already exists !".to_string())
            } else {
                conn.execute("INSERT INTO users (first_name,last_name,username,password) VALUES (?1,?2,?3,?4)", [first_name, last_name, username, password]).unwrap();
                Ok("User created !".to_string())
            }
        }).await;
        match result {
            Ok(mes) => {
                self.answer = mes;
                println!("{}", self.answer);
            }
            Err(_) => {
                self.answer = "Couldn't execute command !".to_string();
                println!("{}", self.answer);
            }
        }
    }

    async fn reply(&mut self) {
        if self.arguments.len() != 2 {
            self.answer = "invalid syntax! Use: <reply> <\"message\"> <message_id>!".to_string();
            println!("{}", self.answer);
            return;
        }
        {
            let users = self.online_users.lock().await;
            if !users.iter().any(|(_, client, _)| *client == self.user_id) {
                self.answer = "Sign in first !".to_string();
                println!("{}", self.answer);
                return;
            }
        }
        let message = self.arguments[0].clone();
        let answered_message_id_string = self.arguments[1].clone();
        let answered_id = answered_message_id_string.parse::<i32>().ok();
        if answered_id.is_none() {
            self.answer = "Invalid number format for message id !".to_string();
            println!("{}", self.answer);
            return;
        }
        let mut sender = None;
        {
            let users = self.online_users.lock().await;
            for (id, client, _) in users.iter() {
                if *client == self.user_id {
                    sender = Some(*id);
                    break;
                }
            }
        }
        if let Some(sender_id) = sender {
            let db_conn = self.data_base.clone();
            let conn = db_conn.lock().await;
            let receiver: Option<i32> = conn
                .call(move |conn| {
                    let mut stmt = conn
                        .prepare("SELECT sender_id FROM message WHERE message_id = ?1;")
                        .unwrap();
                    let mut rows = stmt
                        .query_map([answered_id.unwrap()], |row| row.get::<usize, i32>(0))
                        .unwrap();
                    Ok(match rows.next() {
                        Some(Ok(id)) => Some(id),
                        _ => None,
                    })
                })
                .await
                .unwrap();
            if let Some(receiver_id) = receiver {
                let mut um_guard = self.unprocessed_messages.lock().await;
                let index = (sender_id, receiver_id);
                um_guard
                    .entry(index)
                    .or_insert_with(Vec::new)
                    .push((message, answered_id));
                self.answer = "Reply sent!".to_string();
                println!("{}", self.answer);
            } else {
                self.answer = "Message you want to reply does not exist !".to_string();
                println!("{}", self.answer);
                return;
            }
        } else {
            self.answer = "You have to sign in !".to_string();
            println!("{}", self.answer);
        }
    }

    async fn send(&mut self) {
        if self.arguments.len() != 2 {
            self.answer = "invalid syntax !<send> <\"message\"> <receiver> !".to_string();
            println!("{}", self.answer);
            return;
        }
        {
            let users = self.online_users.lock().await;
            if !users.iter().any(|(_, client, _)| *client == self.user_id) {
                self.answer = "Sign in first !".to_string();
                println!("{}", self.answer);
                return;
            }
        }
        let mut sender_id = None;
        {
            let users = self.online_users.lock().await;
            for (id, client, _) in users.iter() {
                if *client == self.user_id {
                    sender_id = Some(*id);
                    break;
                }
            }
        }
        if sender_id.is_none() {
            self.answer = "User not online !".to_string();
            println!("{}", self.answer);
            return;
        }
        let message = self.arguments[0].clone();
        let username = self.arguments[1].clone();
        let db_conn = self.data_base.clone();
        let conn = db_conn.lock().await;
        let receiver_id: i32 = conn
            .call(move |conn| {
                let mut stmt = conn
                    .prepare("SELECT user_id FROM users WHERE username = ?1;")
                    .unwrap();
                let res = stmt
                    .query_row([username.clone()], |row| row.get::<usize, i32>(0))
                    .unwrap_or(-5);
                Ok(res)
            })
            .await
            .unwrap();
        if receiver_id == -5 {
            self.answer = "User does not exist !".to_string();
            println!("{}", self.answer);
            return;
        }
        {
            let mut um_guard = self.unprocessed_messages.lock().await;
            let index = (sender_id.unwrap(), receiver_id);
            um_guard
                .entry(index)
                .or_insert_with(Vec::new)
                .push((message, None));
        }
        self.answer = "Message sent !".to_string();
        println!("{}", self.answer);
    }

    fn invalid(&mut self) {
        self.answer = "invalid".to_string();
    }

    async fn check_inbox(&mut self) {
        if self.arguments.len() != 0 {
            self.answer = "invalid syntax !<check_inbox> !".to_string();
            println!("{}", self.answer);
            return;
        }
        {
            let users = self.online_users.lock().await;
            if !users.iter().any(|(_, client, _)| *client == self.user_id) {
                self.answer = "Sign in first !".to_string();
                println!("{}", self.answer);
                return;
            }
        }
        let mut id = None;
        {
            let users = self.online_users.lock().await;
            for (uid, client, _) in users.iter() {
                if *client == self.user_id {
                    id = Some(*uid);
                    break;
                }
            }
        }
        if id.is_none() {
            self.answer = "User not found !".to_string();
            println!("{}", self.answer);
            return;
        }
        self.answer = "Unread messages:\n".to_string();
        let db_conn = self.data_base.clone();
        let conn = db_conn.lock().await;
        let unprocessed = {
            let guard = self.unprocessed_messages.lock().await;
            guard.clone()
        };
        let answer_str: String = conn.call(move |conn| {
            let mut m = unprocessed;
            let mut answer = String::new();
            let mut rm = Vec::new();
            for ((sender, receiver), messages) in m.iter() {
                if let Some(user_id) = id {
                    if *receiver == user_id {
                        answer.push_str("from ");
                        answer.push_str(&format!("{}", sender));
                        answer.push('\n');
                        for (content, rep) in messages.iter() {
                            answer.push_str(content);
                            answer.push('\n');
                            if let Some(reply_id) = rep {
                                conn.execute("INSERT INTO message (sender_id, receiver_id, content, reply_to) VALUES (?1, ?2, ?3, ?4)", (sender, receiver, content.clone(), reply_id)).unwrap();
                            } else {
                                conn.execute("INSERT INTO message (sender_id, receiver_id, content) VALUES (?1, ?2, ?3)", (sender, receiver, content.clone())).unwrap();
                            }
                        }
                        rm.push((*sender, *receiver));
                    }
                }
            }
            for key in &rm {
                m.remove_entry(key);
            }
            Ok(answer)
        }).await.unwrap();
        self.answer += &answer_str;
    }

    async fn show_past_chat(&mut self) {
        if self.arguments.len() != 1 {
            self.answer = "invalid syntax !<show_past_chat> <user> !".to_string();
            println!("{}", self.answer);
            return;
        }
        {
            let users = self.online_users.lock().await;
            if !users.iter().any(|(_, client, _)| *client == self.user_id) {
                self.answer = "Sign in first !".to_string();
                println!("{}", self.answer);
                return;
            }
        }
        let username = self.arguments[0].clone();
        self.answer = format!("{}'s chat:\n", username);
        let db_conn = self.data_base.clone();
        let conn = db_conn.lock().await;
        let users_copy = {
            let guard = self.online_users.lock().await;
            guard.clone()
        };
        let user_id = self.user_id;
        let chat: String = conn.call(move |conn| {
            let mut answer = String::new();
            let mut stmt = conn.prepare("SELECT user_id FROM users WHERE username = ?1").unwrap();
            let mut rows = stmt.query_map([username.clone()], |row| row.get::<usize, i64>(0)).unwrap();
            let sender_id = match rows.next() {
                Some(Ok(val)) => val,
                _ => -5,
            };
            let mut receiver_id = -5;
            for (uid, client, _) in users_copy.iter() {
                if *client == user_id {
                    receiver_id = *uid as i64;
                    break;
                }
            }
            let mut stmt = conn.prepare("SELECT message_id, content, reply_to FROM message WHERE (sender_id = ?1 AND receiver_id = ?2) OR (sender_id = ?3 AND receiver_id = ?4)").unwrap();
            let rows = stmt.query_map([sender_id, receiver_id, receiver_id, sender_id], |row| {
                let id = row.get::<usize, i64>(0).unwrap();
                let content = row.get::<usize, String>(1).unwrap();
                let reply_to = row.get::<usize, Option<i64>>(2).unwrap();
                Ok((id, content, reply_to))
            }).unwrap();
            for entry in rows {
                if let Ok((id, content, reply_to)) = entry {
                    answer += &format!("Message-id({}", id);
                    if let Some(reply) = reply_to {
                        answer += &format!(" replied to {}): ", reply);
                    } else {
                        answer += "): ";
                    }
                    answer += &content;
                    answer.push('\n');
                }
            }
            Ok(answer)
        }).await.unwrap();
        self.answer += &chat;
    }

    async fn show_users(&mut self) {
        if self.arguments.len() != 0 {
            self.answer = "invalid syntax !<show_users> !".to_string();
            println!("{}", self.answer);
            return;
        }
        {
            let users = self.online_users.lock().await;
            if !users.iter().any(|(_, client, _)| *client == self.user_id) {
                self.answer = "Sign in first !".to_string();
                println!("{}", self.answer);
                return;
            }
        }
        self.answer = "Online users:\n".to_string();
        let users = self.online_users.lock().await;
        for (_, _, username) in users.iter() {
            self.answer.push_str(username);
            self.answer.push('\n');
        }
    }

    async fn exit(&mut self) {
        if self.arguments.len() != 0 {
            self.answer = "This command shouldn't have arguments !".to_string();
            println!("{}", self.answer);
        }
        {
            let mut users = self.online_users.lock().await;
            if let Some(pos) = users
                .iter()
                .position(|(_, client, _)| *client == self.user_id)
            {
                users.swap_remove(pos);
            } else {
                self.answer = "User is not online !".to_string();
                println!("{}", self.answer);
                return;
            }
        }
        self.answer = "exit!".to_string();
        println!("{}", self.answer);
    }

    pub fn get_answer(&self) -> &str {
        self.answer.as_str()
    }
}
