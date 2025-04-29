use std::net::{TcpStream, SocketAddr, TcpListener};
use std::io::Error;

use clap::{Parser, ArgGroup};

use chat_security::SessionCryptData;


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


fn run_server(socket_addr: SocketAddr) -> Result<SessionCryptData, Error>{
    
    let listener = TcpListener::bind(socket_addr)?;
    
    println!("Listening on {} for connections", listener.local_addr()?.to_string());
    
    let stream = listener.accept()?;
    println!("Recieving data from {:?}", stream.1);
    let mut stream = stream.0;
    
    println!("Attempting to start connection with peer...");
    SessionCryptData::recieve_session(stream)


}

fn connect_to_peer(ip: SocketAddr) -> Result<SessionCryptData, Error>{

    let mut stream = TcpStream::connect(ip)?;

    SessionCryptData::start_session(stream)
}

fn run_event_loop(session: SessionCryptData){
    //Check incoming message buffer, outgoing message buffer, compare times, and display both
}

fn main(){
    let args = Args::parse();
    let session: SessionCryptData;

    if args.recieve{
        let addr = SocketAddr::from(([127, 0, 0, 1], args.port));
        session = run_server(addr).expect("Error: Failed to start server");
        
    }
    else if args.send{
        let addr = match args.address{
            Some(ip) => ip,
            None => panic!("Error: No IP address specified")
        };
        let addr = addr.parse::<SocketAddr>().expect("Error: Invalid IP address");
        session = connect_to_peer(addr).expect("Error: Failed to connect to peer");
    }
    else{
        panic!("Error: No mode specified");
    }

    run_event_loop(session);


}



