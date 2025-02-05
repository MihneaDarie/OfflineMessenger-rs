use common::Message;

pub struct CommandManager {
    command: String,
    arguments: Vec<String>,

    answear: String,
}

impl CommandManager {
    pub fn new() -> Self {
        Self {
            command: String::default(),
            arguments: Vec::new(),
            answear: String::default(),
        }
    }

    pub fn parse_command(&mut self, input: &str) {
        self.arguments.clear();
        self.command.clear();

        let words = input.split_whitespace().collect::<Vec<&str>>();
        self.command = String::from(words[0]);
        for i in words[1..].iter() {
            self.arguments.push(String::from(*i));
        }
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
            "check_inbox" => {
                self.check_inbox();
            }
            _ => {
                self.invalid();
            }
        }
    }
    fn sign_in(&mut self) {
        self.answear = "sign_in".to_string();
    }
    fn sign_out(&mut self) {
        self.answear = "sign_out".to_string();
    }
    fn sign_up(&mut self) {
        self.answear = "sign_up".to_string();

    }
    fn reply(&mut self) {
        self.answear = "reply".to_string();

    }
    fn send(&mut self) {
        self.answear = "send".to_string();

    }
    fn invalid(&mut self) {
        self.answear = "invalid".to_string();

    }
    fn check_inbox(&mut self) {
        self.answear = "check_inbox".to_string();

    }

    pub fn get_answear(&self) -> &str {
        self.answear.as_str()      
    }
}
