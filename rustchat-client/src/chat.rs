use std::net::TcpStream;
use std::io::{Read, Write};

use chat_security;


/*
Chat Handshake protocol:

Each message contains shared secret, timestamp, contents, and 
By rule, the one connecting sends their handshake first. 
Messages are currently limited to 300 characters per message, but may add functionality to allow users to set personal limits later on

SELF_ID > PEER_ID
TIMESTAMP
SHARED_SECRET
MSG_DATA

*/

struct Message{
    timestamp: u64,
    sender_id: String,
    to_id: String,
    contents: String,
}


