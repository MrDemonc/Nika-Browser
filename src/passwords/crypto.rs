use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use pbkdf2::pbkdf2_hmac;
use rand::Rng;
use sha2::Sha256;

const PBKDF2_ITERATIONS: u32 = 100_000;
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;

/// Deriva una clave de 256 bits a partir de la contraseña maestra y una sal criptográfica
pub fn derive_master_key(master_password: &str, salt: &[u8]) -> [u8; 32] {
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(master_password.as_bytes(), salt, PBKDF2_ITERATIONS, &mut key);
    key
}

/// Genera una sal criptográfica aleatoria de 16 bytes
pub fn generate_salt() -> [u8; SALT_LEN] {
    let mut salt = [0u8; SALT_LEN];
    rand::rng().fill_bytes(&mut salt);
    salt
}

/// Genera un nonce único de 12 bytes para AES-256-GCM
pub fn generate_nonce() -> [u8; NONCE_LEN] {
    let mut nonce = [0u8; NONCE_LEN];
    rand::rng().fill_bytes(&mut nonce);
    nonce
}

/// Cifra datos con AES-256-GCM usando la clave derivada
pub fn encrypt_bytes(key: &[u8; 32], plaintext: &[u8]) -> Result<(Vec<u8>, [u8; NONCE_LEN]), String> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| format!("Error inicializando cifrador: {}", e))?;
    let nonce_bytes = generate_nonce();
    let nonce = Nonce::from(nonce_bytes);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| format!("Fallo al cifrar datos: {}", e))?;
    Ok((ciphertext, nonce_bytes))
}

/// Descifra datos con AES-256-GCM comprobando la autenticidad
pub fn decrypt_bytes(key: &[u8; 32], ciphertext: &[u8], nonce_bytes: &[u8; NONCE_LEN]) -> Result<Vec<u8>, String> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| format!("Error inicializando cifrador: {}", e))?;
    let nonce = Nonce::from(*nonce_bytes);
    let decrypted = cipher
        .decrypt(&nonce, ciphertext)
        .map_err(|_| "Fallo al descifrar: autenticación inválida o clave incorrecta".to_string())?;
    Ok(decrypted)
}

/// Utilidad para codificar en Base64
pub fn to_base64(data: &[u8]) -> String {
    glib::base64_encode(data).to_string()
}

/// Utilidad para decodificar desde Base64
pub fn from_base64(encoded: &str) -> Result<Vec<u8>, String> {
    let bytes = glib::base64_decode(encoded);
    if bytes.is_empty() && !encoded.trim().is_empty() {
        Err("Cadena Base64 inválida".to_string())
    } else {
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pbkdf2_derivation() {
        let salt = generate_salt();
        let key1 = derive_master_key("my-secret-pass", &salt);
        let key2 = derive_master_key("my-secret-pass", &salt);
        assert_eq!(key1, key2);

        let other_salt = generate_salt();
        let key3 = derive_master_key("my-secret-pass", &other_salt);
        assert_ne!(key1, key3);

        let key_wrong_pass = derive_master_key("different-pass", &salt);
        assert_ne!(key1, key_wrong_pass);
    }

    #[test]
    fn test_aes_gcm_encrypt_decrypt() {
        let salt = generate_salt();
        let key = derive_master_key("super-secret-key-123", &salt);
        let plaintext = b"Hello, Nika Browser AES-256-GCM!";

        let (ciphertext, nonce) = encrypt_bytes(&key, plaintext).expect("Encryption failed");
        assert_ne!(ciphertext, plaintext);

        let decrypted = decrypt_bytes(&key, &ciphertext, &nonce).expect("Decryption failed");
        assert_eq!(decrypted, plaintext);

        // Intento con clave incorrecta
        let wrong_key = derive_master_key("wrong-password", &salt);
        let failed_decrypt = decrypt_bytes(&wrong_key, &ciphertext, &nonce);
        assert!(failed_decrypt.is_err());

        // Intento con texto alterado
        let mut tampered = ciphertext.clone();
        tampered[0] ^= 0xFF;
        let tampered_decrypt = decrypt_bytes(&key, &tampered, &nonce);
        assert!(tampered_decrypt.is_err());
    }

    #[test]
    fn test_base64_roundtrip() {
        let data = b"Nika Browser 2026";
        let enc = to_base64(data);
        let dec = from_base64(&enc).expect("Base64 decode failed");
        assert_eq!(dec, data);
    }
}
