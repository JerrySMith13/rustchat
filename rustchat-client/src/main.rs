use tokio::net::{TcpStream, TcpListener};
use std::net::SocketAddr;

use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{channel, Sender, Receiver};
use std::io::Error;
use std::time::{SystemTime, UNIX_EPOCH};

use clap::{Parser, ArgGroup};

use chat_security::{SessionCryptData, Message};

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


async fn run_server(socket_addr: SocketAddr) -> Result<SessionCryptData, Error>{
    
    let listener = TcpListener::bind(socket_addr).await?;
    
    println!("Listening on {} for connections", listener.local_addr()?.to_string());
    
    let stream = listener.accept().await?;
    println!("Recieving data from {:?}", stream.1);
    let stream = stream.0;
    
    println!("Attempting to start connection with peer...");
    SessionCryptData::recieve_session(stream).await


}

async fn connect_to_peer(ip: SocketAddr) -> Result<SessionCryptData, Error>{

    let stream = TcpStream::connect(ip).await?;

    SessionCryptData::start_session(stream).await
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
#[allow(unused_assignments)]
async fn write_event_loop(cond: Arc<Mutex<bool>>, send: Sender<Message>) {
    let ioin = std::io::stdin();
    let mut input = String::new();
    loop {
        input.clear();
        ioin.read_line(&mut input).expect("Error: Failed to read line");
        let msg = message(&input);
        let mut cond = *cond.lock().unwrap();
        if cond == false{
            println!("Exiting event loop");
            break;
        }
        if input.trim() == "/quit"{
            println!("Exiting event loop");
            cond = false;
            break;
        }
        send.send(msg).await.expect("Error: Failed to send message");

    }
}



async fn run_event_loop(mut session: SessionCryptData) {
    let (input_send, mut input_recv): (Sender<Message>, Receiver<Message>) = channel(50);
    let cond = Arc::new(Mutex::new(true));
    let cond_clone = Arc::clone(&cond);
    tokio::spawn(async move {
        write_event_loop(cond_clone, input_send).await;
    });
    let mut all_msgs: Vec<Message> = Vec::new();

    loop{
        if input_recv.len() > 0{
            let msg = input_recv.recv().await.expect("Error: Failed to receive message");
            all_msgs.push(msg);
        }
        if let Err(e) = session.poll_recieve_ready(){
            match e.kind(){
                std::io::ErrorKind::WouldBlock => {
                    
                },
                _ => {
                    println!("Error: Failed to poll for messages: {}", e);
                    return;
                }
            }
        }
        else{
            let msg = session.recieve_message().await.expect("Error: Failed to receive message");
            all_msgs.push(msg);
        }
        if all_msgs.len() > 0{
            for msg in all_msgs.iter(){
                println!("{}: {}", msg.sender_id, msg.contents);
            }
            all_msgs.clear();
        }
        let cond = cond.lock().unwrap();   
        if *cond == false{
            println!("Exiting event loop");
            break;
        }
    }


}

#[tokio::main]
async fn main(){
    let args = Args::parse();
    let session: SessionCryptData;

    if args.recieve{
        let addr = SocketAddr::from(([127, 0, 0, 1], args.port));
        session = run_server(addr).await.expect("Error: Failed to start server");
        
    }
    else if args.send{
        let addr = match args.address{
            Some(ip) => ip,
            None => panic!("Error: No IP address specified")
        };
        let addr = addr.parse::<SocketAddr>().expect("Error: Invalid IP address");
        session = connect_to_peer(addr).await.expect("Error: Failed to connect to peer");
    }
    else{
        panic!("Error: No mode specified");
    }

    run_event_loop(session).await;


}



