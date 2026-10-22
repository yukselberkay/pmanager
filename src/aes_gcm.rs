/**
 * aes_gcm.rs
 * Handles the encryption and the decryption of data using aes-gcm256.
*/
use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use zeroize::Zeroizing;

pub const KEY_LEN: usize = 32;
// 96 bits, must be unique for every encryption under the same key.
pub const NONCE_LEN: usize = 12;
pub const TAG_LEN: usize = 16;

pub struct AesGcm256;

impl AesGcm256 {
    /// Encrypts `data`, authenticating `aad` along with it.
    pub fn encrypt_bytes(
        key: &[u8; KEY_LEN],
        nonce: &[u8; NONCE_LEN],
        aad: &[u8],
        data: &[u8],
    ) -> Result<Vec<u8>, aes_gcm::Error> {
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));

        cipher.encrypt(Nonce::from_slice(nonce), Payload { msg: data, aad })
    }

    /// Decrypts `ciphertext`, failing if it or `aad` were tampered with.
    pub fn decrypt_bytes(
        key: &[u8; KEY_LEN],
        nonce: &[u8; NONCE_LEN],
        aad: &[u8],
        ciphertext: &[u8],
    ) -> Result<Zeroizing<Vec<u8>>, aes_gcm::Error> {
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));

        let plaintext = cipher.decrypt(
            Nonce::from_slice(nonce),
            Payload {
                msg: ciphertext,
                aad,
            },
        )?;

        Ok(Zeroizing::new(plaintext))
    }
}
