use std::{io, net::{SocketAddr, TcpListener, TcpStream}, sync::mpsc::{channel, Receiver, Sender, TryRecvError}, thread, time::{SystemTime, UNIX_EPOCH}};

use clap::{Parser, ArgGroup};

use chat_security::{Message, SessionCryptData};

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




fn message(text: &str, should_quit: bool) -> Message{
    let time: u64;
    if should_quit{
        time = 0;
    }
    else{
        time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap().as_secs();
    } 
    let (usr1, usr2) = ("Sender", "Reciever");
    
    return Message{
        sender_id: usr1.to_string(),
        to_id: usr2.to_string(),
        contents: text.to_string(),
        timestamp: time,
    }

}

fn output_loop(output_recv: Receiver<String>) {
    while let Ok(message) = output_recv.recv() {
        println!("{}", message);
    }
}

fn input_loop(sender: Sender<Message>){
    let ioin = std::io::stdin();
    let mut input_str = String::new();
    let mut msg: Message;
    let mut should_quit: bool = false;
    while !should_quit{
        ioin.read_line(&mut input_str).unwrap();
        if input_str.trim() == "/quit"{
            should_quit = true;
        }
        msg = message(&input_str, should_quit);
        sender.send(msg).unwrap();
        input_str.clear();
    }
}

fn run_event_loop(mut session: SessionCryptData) -> Result<(), io::Error>{
    // Variable declarations before loop begins, saves memory allocation overhead
    let (input_sender, input_recv) = channel::<Message>();
    let (output_sender, output_recv) = channel::<String>();
    let mut should_quit: bool = false;
    let mut input_msg:  Result<Message, TryRecvError>;
    let mut return_err: Result<(), io::Error> = Ok(());


    thread::spawn(move || {
        input_loop(input_sender);
    });
    thread::spawn(move || {
        output_loop(output_recv);
    });
    let mut all_msgs: Vec<Message> = Vec::with_capacity(10);
    while !should_quit{
        //First section, checks for user input
        input_msg = input_recv.try_recv();
        match input_msg{
            Ok(m) => {
                if m.timestamp == 0{
                    should_quit = true;
                }
                match session.send_message(m.clone()){
                    Ok(_) => {},
                    Err(_) => {
                        println!("Error: message could not be sent");
                    }
                };
                all_msgs.push(m);
            },
            Err(e) => {
                if e == TryRecvError::Empty { continue; }
                //Only other possibility is disconnection
                else {should_quit = true;}
            }
        }
        

        //Second section, checks if peer messages are available
        match session.check_data_available(){
            Ok(avail) => {
                if avail{
                    match session.recieve_message(){
                        Ok(m) => {
                            if m.timestamp == 0{
                                should_quit = true;
                            }
                            all_msgs.push(m)
                        },
                        Err(e) => {
                            return_err = Err(e);
                            should_quit = true;
                        }
                    }
                }
            }
            Err(e) => {
                should_quit = true;
                return_err = Err(e);
            }
        }

        //Third section, display all messages
        for msg in &all_msgs{
            output_sender.send(msg.displayable()).unwrap();
        }

        all_msgs.clear();
    }
    return_err
}

fn run_server(addr: SocketAddr) -> Result<SessionCryptData, io::Error>{
    let listener = TcpListener::bind(addr)?;
    println!("Listening on {}", listener.local_addr().unwrap());
    let stream = listener.accept()?.0;
    SessionCryptData::recieve_session(stream)
}

fn connect_to_server(addr: SocketAddr) -> Result<SessionCryptData, io::Error>{
    let stream = TcpStream::connect(addr)?;
    SessionCryptData::start_session(stream)
}

fn main() -> Result<(), io::Error>{
    let args = Args::parse();
    let session: SessionCryptData;
    if args.recieve{
        let addr: SocketAddr = format!("127.0.0.1:{}", args.port).parse().unwrap();
        session = run_server(addr).unwrap();
    }
    else{
        let addr: SocketAddr = args.address.unwrap().parse().unwrap();
        session = connect_to_server(addr).unwrap();
    }
    run_event_loop(session)

}



