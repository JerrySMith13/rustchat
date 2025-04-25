use std::net::TcpStream;
use std::io::{Read, Write};


use bincode::serialize;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use x25519_dalek::{EphemeralSecret, PublicKey};
use sha2::Sha256;
use hkdf::Hkdf;
use chacha20poly1305::{
    aead::{Aead, KeyInit, AeadCore},
    XChaCha20Poly1305
};



#[derive(Serialize, Deserialize)]
struct HandshakeData{
    public_key: [u8; 32]
}
#[allow(dead_code)]
impl HandshakeData{
    fn write_length_prefixed(stream: &mut TcpStream, data: &[u8]) -> Result<(), std::io::Error> {
        let length = (data.len() as u32).to_be_bytes();
        stream.write_all(&length)?;
        stream.write_all(data)?;
        stream.flush()?;
        Ok(())
    }
    
    fn read_length_prefixed(stream: &mut TcpStream) -> Result<Vec<u8>, std::io::Error> {
        let mut length_bytes = [0u8; 4];
        stream.read_exact(&mut length_bytes)?;
        let length = u32::from_be_bytes(length_bytes) as usize;
    
        let mut buffer = vec![0; length];
        stream.read_exact(&mut buffer)?;
        Ok(buffer)
    }
        
    fn init_handshake(stream: &mut TcpStream, pub_key: PublicKey) -> Result<HandshakeData, std::io::Error>{
        let handshake = HandshakeData{
            public_key: pub_key.to_bytes(),
        };
        let serialized = serialize(&handshake).expect("Failed to serialize handshake data");
        
        Self::write_length_prefixed(stream, &serialized)?;
    
        let buf = Self::read_length_prefixed(stream)?;
        let response: HandshakeData = bincode::deserialize(&buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        return Ok(response);
    }

    fn recieve_handshake(stream: &mut TcpStream, pub_key: PublicKey) -> Result<HandshakeData, std::io::Error>{
        let buf = Self::read_length_prefixed(stream)?;
        let response: HandshakeData = bincode::deserialize(&buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let buf = bincode::serialize(&HandshakeData{ public_key: pub_key.to_bytes() })
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Self::write_length_prefixed(stream, &buf)?;
        return Ok(response);
    }
}



#[derive(Serialize, Deserialize)]
struct EncryptedMessage {
    nonce: [u8; 24],
    ciphertext: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
struct Message{
    sender_id: String,
    to_id: String,
    contents: String,
    timestamp: u64,
}
impl Message{
    
    pub fn from_string(message: String) -> Self{
        todo!("Function not yet implemented, but will parse text data into a message object for easier reading");
    }

    pub fn to_string(&self) -> String{
        todo!("Function not yet implemented, but will parse a message object into a string for easier sending");
    }

}


struct SessionCryptData{
    cipher: XChaCha20Poly1305,

}

#[allow(dead_code)]
impl SessionCryptData{
    fn derive_key(shared_secret: &[u8]) -> [u8; 32] {
        let hk = Hkdf::<Sha256>::new(None, shared_secret);
        let mut encryption_key = [0u8; 32];
        hk.expand(b"chat encryption key", &mut encryption_key).unwrap();
        encryption_key
    }
    
    /// This function is called by the peer who initiated the connection (i.e. the one sending the initial handshake
    /// message)
    fn start_session(stream: &mut TcpStream) -> Result<Self, std::io::Error>{
        let self_secret = EphemeralSecret::random_from_rng(OsRng);
        let self_public = PublicKey::from(&self_secret);

        let peer_public = HandshakeData::init_handshake(stream, self_public)?;
        let peer_public = PublicKey::from(peer_public.public_key);

        let shared = self_secret.diffie_hellman(&peer_public);
        let shared = Self::derive_key(shared.as_bytes());

        let cipher = XChaCha20Poly1305::new_from_slice(&shared).unwrap();


        return Ok(SessionCryptData{
            cipher,
        });
    
    }

    fn recieve_session(stream: &mut TcpStream) -> Result<Self, std::io::Error>{
        let self_secret = EphemeralSecret::random_from_rng(OsRng);
        let self_public = PublicKey::from(&self_secret);

        let peer_public = HandshakeData::recieve_handshake(stream, self_public)?;
        let peer_public = PublicKey::from(peer_public.public_key);

        

        let shared = self_secret.diffie_hellman(&peer_public);
        let shared = Self::derive_key(shared.as_bytes());

        let cipher = XChaCha20Poly1305::new_from_slice(&shared).unwrap();

        return Ok(SessionCryptData{
            cipher,
        });
    }

    fn send_message(&self, stream: &mut TcpStream, message: Message) -> Result<(), std::io::Error>{

        let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
        let msg_bytes = serialize(&message).expect("Failed to serialize message");
        let ciphertext = self.cipher.encrypt(&nonce, msg_bytes.as_ref())
            .expect("Failed to encrypt message");
        let encrypted_message = EncryptedMessage {
            nonce: *nonce.as_ref(),
            ciphertext,
        };
        let serialized = serialize(&encrypted_message).expect("Failed to serialize encrypted message");
        HandshakeData::write_length_prefixed(stream, &serialized)?;
        Ok(())


    }

    
}


