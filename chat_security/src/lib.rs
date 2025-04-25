pub(crate) 
use std::net::TcpStream;
use std::io::{Read, Write};


use bincode::serialize;
use chacha20poly1305::XNonce;
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

/*
    Message format
    SENDER_DISPLAYNAME -> TO_DISPLAYNAME
    TIMESTAMP
    MESSAGE_CONTENTS

 */
#[derive(Serialize, Deserialize)]
struct Message{
    sender_id: String,
    to_id: String,
    contents: String,
    timestamp: u64,
}
impl ToString for Message{
    fn to_string(&self) -> String{
        let mut message = String::new();
        message.push_str(&format!("{} -> {}", self.sender_id, self.to_id));
        message.push_str(&format!("\n{}", self.timestamp));
        message.push_str(&format!("\n{}", self.contents));
        return message;
    }
}
impl Message{
    fn from_string(message: String) -> Result<Self, std::io::Error>{
        let lines = message.lines().collect::<Vec<_>>();
        let head_line = lines[0];
        let head_parts = head_line.split("->").collect::<Vec<_>>();
        if head_parts.len() != 2{
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid message format (Header line invalid)"));
        }
        let sender_id = head_parts[0].trim().to_string();
        let to_id = head_parts[1].trim().to_string();
        let timestamp = lines[1].trim().parse::<u64>()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid message format (Timestamp invalid)"))?;
        let contents = lines[2..].join("\n").trim().to_string();

        return Ok(Message{
            sender_id,
            to_id,
            contents,
            timestamp,
        });
        
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
        let message = message.to_string();
        let msg_bytes = bincode::serialize(&message)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
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

    fn recieve_message(&self, stream: &mut TcpStream) -> Result<Message, std::io::Error>{
        let buf = HandshakeData::read_length_prefixed(stream)?;
        let encrypted_message: EncryptedMessage = bincode::deserialize(&buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let nonce = XNonce::from_slice(&encrypted_message.nonce);
        let decrypted = self.cipher.decrypt(nonce, encrypted_message.ciphertext.as_ref())
            .expect("Failed to decrypt message");
        let message: String = bincode::deserialize(&decrypted)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let message = Message::from_string(message)?;
        Ok(message)
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{TcpListener, TcpStream};
    use std::thread;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn setup_tcp_pair() -> (TcpStream, TcpStream) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        
        let client = thread::spawn(move || {
            TcpStream::connect(addr).unwrap()
        });
        
        let (server, _) = listener.accept().unwrap();
        let client = client.join().unwrap();
        
        (client, server)
    }

    #[test]
    fn test_handshake() {
        let (mut client, mut server) = setup_tcp_pair();
        
        let client_thread = thread::spawn(move || {
            let client_secret = EphemeralSecret::random_from_rng(OsRng);
            let client_public = PublicKey::from(&client_secret);
            HandshakeData::init_handshake(&mut client, client_public).unwrap()
        });

        let server_secret = EphemeralSecret::random_from_rng(OsRng);
        let server_public = PublicKey::from(&server_secret);
        let server_response = HandshakeData::recieve_handshake(&mut server, server_public).unwrap();
        
        let client_response = client_thread.join().unwrap();
        
        assert_eq!(server_response.public_key.len(), 32);
        assert_eq!(client_response.public_key.len(), 32);
    }

    #[test]
    fn test_session_establishment() {
        let (mut client, mut server) = setup_tcp_pair();
        
        let client_thread = thread::spawn(move || {
            SessionCryptData::start_session(&mut client).unwrap()
        });

        let _server_session = SessionCryptData::recieve_session(&mut server).unwrap();
        let _client_session = client_thread.join().unwrap();
    }

    #[test]
    fn test_message_exchange() {
        let (mut client, mut server) = setup_tcp_pair();
        
        let client_thread = thread::spawn(move || {
            let session = SessionCryptData::start_session(&mut client).unwrap();
            
            // Send a test message
            let msg = Message {
                sender_id: "client".to_string(),
                to_id: "server".to_string(),
                contents: "Hello server!".to_string(),
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            };
            
            session.send_message(&mut client, msg).unwrap();
            
            // Receive response
            let response = session.recieve_message(&mut client).unwrap();
            response
        });

        let server_session = SessionCryptData::recieve_session(&mut server).unwrap();
        
        // Receive client message
        let received = server_session.recieve_message(&mut server).unwrap();
        assert_eq!(received.sender_id, "client");
        assert_eq!(received.to_id, "server");
        assert_eq!(received.contents, "Hello server!");
        
        // Send response
        let response = Message {
            sender_id: "server".to_string(),
            to_id: "client".to_string(),
            contents: "Hello client!".to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        server_session.send_message(&mut server, response).unwrap();
        
        let client_received = client_thread.join().unwrap();
        assert_eq!(client_received.sender_id, "server");
        assert_eq!(client_received.to_id, "client");
        assert_eq!(client_received.contents, "Hello client!");
    }

    #[test]
    fn test_message_formatting() {
        let msg = Message {
            sender_id: "alice".to_string(),
            to_id: "bob".to_string(),
            contents: "Hello!".to_string(),
            timestamp: 1234567890,
        };

        let formatted = msg.to_string();
        let parsed = Message::from_string(formatted).unwrap();

        assert_eq!(parsed.sender_id, "alice");
        assert_eq!(parsed.to_id, "bob");
        assert_eq!(parsed.contents, "Hello!");
        assert_eq!(parsed.timestamp, 1234567890);
    }
}