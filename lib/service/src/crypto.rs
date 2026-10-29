use std::fs;
use std::io::Write;
use std::path::Path;

use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::{Aes256Gcm, Nonce};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use once_cell::sync::Lazy;
use sha2::{Digest, Sha256};

use api::error::ErrorInfo;
use api::ResponseResult;

const PREFIX: &str = "enc:v1:";
const KEY_SIZE: usize = 32;
const NONCE_SIZE: usize = 12;

static CIPHER: Lazy<Result<Aes256Gcm, ErrorInfo>> = Lazy::new(|| {
    load_key()
        .map_err(|e| ErrorInfo::new(0, &format!("load data encryption key failed: {}", e)))
        .map(|key| Aes256Gcm::new_from_slice(&key).expect("invalid data encryption key"))
});

fn cipher() -> ResponseResult<&'static Aes256Gcm> {
    CIPHER.as_ref().map_err(|e| e.clone())
}

fn key_path() -> String {
    std::env::var("DATA_KEY_PATH").unwrap_or_else(|_| "./data/encryption.key".to_string())
}

fn derive_key(value: &[u8]) -> [u8; KEY_SIZE] {
    let hash = Sha256::digest(value);
    let mut key = [0u8; KEY_SIZE];
    key.copy_from_slice(&hash);
    key
}

fn load_key() -> ResponseResult<[u8; KEY_SIZE]> {
    if let Ok(value) = std::env::var("DATA_ENCRYPTION_KEY") {
        let value = value.trim();
        if !value.is_empty() {
            return Ok(derive_key(value.as_bytes()));
        }
    }

    let path = key_path();
    if let Some(parent) = Path::new(&path).parent() {
        fs::create_dir_all(parent).map_err(|e| {
            ErrorInfo::new(0, &format!("create data key directory failed: {}", e))
        })?;
    }

    if !Path::new(&path).exists() {
        let mut key = [0u8; KEY_SIZE];
        OsRng.fill_bytes(&mut key);
        let encoded = STANDARD.encode(key);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            let mut file = fs::OpenOptions::new()
                .mode(0o600)
                .write(true)
                .create_new(true)
                .open(&path)
                .map_err(|e| {
                    ErrorInfo::new(0, &format!("create data key file failed: {}", e))
                })?;
            file.write_all(encoded.as_bytes()).map_err(|e| {
                ErrorInfo::new(0, &format!("write data key file failed: {}", e))
            })?;
        }
        #[cfg(not(unix))]
        fs::write(&path, encoded)
            .map_err(|e| ErrorInfo::new(0, &format!("write data key file failed: {}", e)))?;
        return Ok(key);
    }

    let content = fs::read_to_string(&path).map_err(|e| {
        ErrorInfo::new(0, &format!("read data key file failed: {}", e))
    })?;
    let content = content.trim();
    if let Ok(key) = STANDARD.decode(content) {
        if key.len() == KEY_SIZE {
            let mut result = [0u8; KEY_SIZE];
            result.copy_from_slice(&key);
            return Ok(result);
        }
    }
    Ok(derive_key(content.as_bytes()))
}

pub fn is_encrypted(value: &str) -> bool {
    value.starts_with(PREFIX)
}

pub fn encrypt_str(value: &str) -> ResponseResult<String> {
    if is_encrypted(value) {
        return Ok(value.to_string());
    }
    let cipher = cipher()?;
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let encrypted = cipher
        .encrypt(nonce, value.as_bytes())
        .map_err(|e| ErrorInfo::new(0, &format!("encrypt data failed: {}", e)))?;
    Ok(format!(
        "{}{}:{}",
        PREFIX,
        STANDARD.encode(nonce_bytes),
        STANDARD.encode(encrypted)
    ))
}

pub fn decrypt_str(value: &str) -> ResponseResult<String> {
    if !is_encrypted(value) {
        return Ok(value.to_string());
    }
    let encoded = value
        .strip_prefix(PREFIX)
        .ok_or_else(|| ErrorInfo::new(0, "invalid encrypted data prefix"))?;
    let mut parts = encoded.split(':');
    let nonce = parts
        .next()
        .ok_or_else(|| ErrorInfo::new(0, "invalid encrypted data nonce"))?;
    let encrypted = parts
        .next()
        .ok_or_else(|| ErrorInfo::new(0, "invalid encrypted data payload"))?;
    let nonce = STANDARD
        .decode(nonce)
        .map_err(|e| ErrorInfo::new(0, &format!("decode encrypted nonce failed: {}", e)))?;
    let encrypted = STANDARD
        .decode(encrypted)
        .map_err(|e| ErrorInfo::new(0, &format!("decode encrypted payload failed: {}", e)))?;
    let cipher = cipher()?;
    let decrypted = cipher
        .decrypt(Nonce::from_slice(&nonce), encrypted.as_ref())
        .map_err(|e| ErrorInfo::new(0, &format!("decrypt data failed: {}", e)))?;
    String::from_utf8(decrypted).map_err(|e| {
        ErrorInfo::new(0, &format!("decode decrypted data failed: {}", e))
    })
}

pub fn encrypt(value: Option<&str>) -> ResponseResult<Option<String>> {
    match value {
        Some(value) => Ok(Some(encrypt_str(value)?)),
        None => Ok(None),
    }
}

pub fn decrypt(value: Option<&str>) -> ResponseResult<Option<String>> {
    match value {
        Some(value) => Ok(Some(decrypt_str(value)?)),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_test_key() {
        std::env::set_var("DATA_ENCRYPTION_KEY", "test-key");
    }

    #[test]
    fn encrypt_and_decrypt_roundtrip() {
        init_test_key();
        let encrypted = encrypt_str("secret").unwrap();
        assert!(is_encrypted(&encrypted));
        assert_ne!(encrypted, "secret");
        assert_eq!(decrypt_str(&encrypted).unwrap(), "secret");
    }

    #[test]
    fn decrypt_plain_text() {
        init_test_key();
        assert_eq!(decrypt_str("plain").unwrap(), "plain");
    }

    #[test]
    fn encrypt_none() {
        init_test_key();
        assert_eq!(encrypt(None).unwrap(), None);
        assert_eq!(decrypt(None).unwrap(), None);
    }
}
