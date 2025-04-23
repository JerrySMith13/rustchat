use std::net::TcpStream;
use std::io::{Read, Write};

use bincode::serialize;
use rand::{rngs::OsRng, CryptoRng};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use x25519_dalek::{EphemeralSecret, PublicKey};


#[derive(Serialize, Deserialize)]
struct HandshakeData{
    public_key: [u8; 32]
}
impl HandshakeData{
    fn init_handshake(stream: &mut TcpStream, pub_key: PublicKey) -> Result<HandshakeData, std::io::Error>{
        let handshake = HandshakeData{
            public_key: pub_key.to_bytes(),
        };
        let serialized = serialize(&handshake).expect("Failed to serialize handshake data");
        stream.write_all(&serialized)?;
        stream.flush()?;
        let mut buffer = vec![0; 1024];
        stream.read(&mut buffer)?;
    }
}



struct SessionCryptData{
    shared_key: [u8; 32],
    self_secret: EphemeralSecret,
    peer_public: PublicKey,
    
}



impl SessionCryptData{
    /// This function is called by the peer who initiated the connection (i.e. the one sending the initial handshake
    /// message)
    fn start_session(stream: &mut TcpStream) -> Result<Self, std::io::Error>{
        let self_secret = EphemeralSecret::random_from_rng(OsRng);
        let self_public = PublicKey::from(&self_secret);

        HandshakeData::init_handshake(stream, self_public)?;

        
    
    }
}


