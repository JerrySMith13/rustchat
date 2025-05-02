use std::time::{SystemTime, UNIX_EPOCH};

use clap::{Parser, ArgGroup};

use chat_security::Message;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(group(
    ArgGroup::new("mode")
        .required(true)
        .args(["send", "recieve"]),
))]
pub struct Args{
    
    #[arg(short, long, required_if_eq("send", "false"))]
    /// Starts rustchat in recieving mode 
    recieve: bool,
    
    #[arg(short, long, required_if_eq("recieve", "false"))]
    /// Starts rustchat in send mode, with specified IP to chat with
    send: bool,
    
    #[arg(short, long, default_value_t = 0, requires = "recieve")]
    /// Specifies port to listen on (if left blank, system will choose for you)
    port: u16,

    #[arg(short, long, requires = "send")]
    /// IP to begin chatting with
    address: Option<String>

}



fn message(text: &str) -> Message{
    let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let (usr1, usr2) = ("Todo", "todo");
    return Message{
        sender_id: usr1.to_string(),
        to_id: usr2.to_string(),
        contents: text.to_string(),
        timestamp: time,
    }

}


fn main(){
    


}



