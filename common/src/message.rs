


pub struct Message {
    Text:String,
    Title:String,
    date_time:String
}

impl Message {
    pub fn default() -> Self {
        Self{
            Text: String::default(),
            Title: String::default(),
            date_time: String::default(),
        }
    }
    pub fn new(text: &str, title: &str) -> Self {
        let txt = String::from(text);
        let tt = String::from(title); 
        Self{
            Text: txt,
            Title: tt,
            date_time: String::default(),
        }
    }
}