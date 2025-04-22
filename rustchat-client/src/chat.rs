use std::net::TcpStream;
use std::io::{Read, Write};

use chat_security;


/*
Chat Handshake protocol:





*/

struct ChatSessionData{
    peer_id: String,
    self_id: String,
    
}


impl ChatSessionData {
    pub fn start_session_as_server(mut socket: TcpStream) -> Self{
        let mut buf = [0u8; 1024];
        socket.read(&mut buf).expect("Failed to read from socket");

    }
}