use std::net;
use std::io::{Write, Read};

/*
Chat protocol:
Upon accepting a connection, the server will first send a handshake message to the client.

Handshake message:
HELLO <client_id> 
param: val
param2: val2
param3: val3


*/


struct ChatMetadata {
    peer_id: String,
    self_id: String,
    self_random: String,
    peer_random: String,
}


fn server_handshake(socket: &mut net::TcpStream) {
    let mut buffer = [0; 1024];
    socket.read(&mut buffer).expect("Failed to read from socket");
    
}

fn run_server() {
    let listener = net::TcpListener::bind("127.0.0.1:0").expect("Failed to bind to address");
    let port = listener.local_addr().unwrap().port();
    println!("Server is running on port: {}", port);
    match listener.accept(){
        Ok((mut socket, addr)) => {
            println!("Recieving connection from: {}", addr);
            server_handshake(&mut socket);

        },
        Err(e) => {
            println!("Failed to accept connection: {}", e);
        }
    }

}

fn chat(mut socket: net::TcpStream) {
    println!("Connected to the server!");

    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).expect("Failed to read line");
        socket.write_all(input.as_bytes()).expect("Failed to send message");
    }
}


fn main() {
    println!("Welcome to RustChat!\n
    This is a simple chat application written in Rust.\n
    Options:\n
    1. Start a server\n
    2. Connect to a server\n
    3. Exit\n");


    let mut input = String::new();
    std::io::stdin().read_line(&mut input).expect("Failed to read line");
    let choice: u32 = input.trim().parse().expect("Please enter a number");

    match choice {
        1 => run_server(),
        2 => connect_to_server(),
        3 => println!("Exiting..."),
        _ => println!("Invalid choice. Please try again."),
    }
}    
