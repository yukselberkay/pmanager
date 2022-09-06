use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};

pub struct AesGcm256 {
    pub key: String,
    pub nonce: String,
}

impl AesGcm256 {
    fn new(key: String, nonce: String) -> AesGcm256 {
        AesGcm256 {
            key: String::from(key),
            nonce: String::from(nonce),
        }
    }

    pub fn encrypt_bytes(
        keyval: &String,
        nonce: String,
        data: Vec<u8>
    ) -> Vec<u8> {
        //key must be 32 bytes.
        let key = Key::from_slice(keyval.as_bytes());        
        let cipher = Aes256Gcm::new(key);
        
        // 96 bits, unique per value
        let nonce = Nonce::from_slice(nonce.as_bytes());

        let ciphertext = cipher.encrypt(nonce, data.as_ref())
            .unwrap();

        ciphertext
    }


    pub fn encrypt(keyval: &String, nonce: String, plaintext: String) -> Vec<u8>{
        
        let key = Key::from_slice(keyval.as_bytes());
        let cipher = Aes256Gcm::new(key);
    
        let nonce = Nonce::from_slice(nonce.as_bytes());
    
        let ciphertext = cipher.encrypt(nonce, plaintext.as_bytes().as_ref())
            .expect("encryption failure");

        ciphertext
    }

    pub fn decrypt(keyval: &String, nonce_val: String, ciphertext: Vec<u8>) -> Vec<u8>{
        //key must be 32 bytes.
        
        let key = Key::from_slice(keyval.as_bytes());
        let cipher = Aes256Gcm::new(key);
    
        // 96 bits, unique per value
        let nonce = Nonce::from_slice(nonce_val.as_bytes());
    
        let plaintext = cipher.decrypt(nonce, ciphertext.as_ref())
            .expect("decryption failure");
   
        plaintext
    }
}

