


pub struct Message {
    Text:String,
    date_time:String
}

impl Message {
    pub fn default() -> Self {
        Self{
            Text: String::default(),
            date_time: String::default(),
        }
    }
    pub fn new(text: &str, date_time: &str) -> Self {
        let txt = String::from(text);
        let time = String::from(date_time); 
        Self{
            Text: txt,
            date_time: time,
        }
    }

    pub fn get_text(&self) -> Option<String> {
        if self.Text != String::default() {
            return Some(self.Text.clone());
        } else {
            return None;
        }
    }

    pub fn get_time(&self) -> Option<String> {
        if self.date_time != String::default() {
            return Some(self.date_time.clone());
        } else {
            return None;
        }
    }
}