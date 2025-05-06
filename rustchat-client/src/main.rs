use std::{io, net::{SocketAddr, TcpListener, TcpStream},
     time::{SystemTime, UNIX_EPOCH}};

use clap::{Parser, ArgGroup};

use chat_security::{Message, SessionCryptData};

mod terminal;
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
    address: Option<String>,

    #[arg(short, long)]
    /// Specifies your own display name
    name: String,

}

fn message(text: &str, name: &str) -> Message{
    let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap().as_secs();
    
    let (usr1, usr2) = (name, "Reciever");
    
    return Message{
        sender_id: usr1.to_string(),
        to_id: usr2.to_string(),
        contents: text.to_string(),
        timestamp: time,
    }

}



fn main() -> Result<(), io::Error>{
    let args = Args::parse();
    let session: SessionCryptData;
    if args.recieve{
        let addr = SocketAddr::from(([127, 0, 0, 1], args.port));
        let listener = TcpListener::bind(addr)?;
        println!("Listening on {}", listener.local_addr()?);
        session = SessionCryptData::recieve_session(listener.accept()?.0)?;
        match terminal::ChatWindow::run_main(session, &args.name){
            Err(e) => {
                if e.kind() == io::ErrorKind::UnexpectedEof {
                    println!("Session ended");
                    return Ok(());
                } else {
                    return Err(e);
                }
                
            }
            _ => {
                println!("Session ended");
                return Ok(());
            }
        };
    }
    else{
        let addr: SocketAddr = args.address.unwrap().parse().unwrap();
        let stream = TcpStream::connect(addr)?;
        println!("Connected to {}", addr);
        session = SessionCryptData::start_session(stream)?;
        match terminal::ChatWindow::run_main(session, &args.name){
            Err(e) => {
                if e.kind() == io::ErrorKind::UnexpectedEof {
                    println!("Session ended");
                    return Ok(());
                } else {
                    return Err(e);
                }
                
            }
            _ => {
                println!("Session ended");
                return Ok(());
            }
        };
    }
    
}



