use aes_gcm::aead::{Aead, KeyInit, OsRng, rand_core::RngCore};
use aes_gcm::{Aes256Gcm, Nonce};
use argon2::Argon2;
use base64::Engine;
use std::path::PathBuf;

const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;
const B64: base64::engine::general_purpose::GeneralPurpose = base64::engine::general_purpose::STANDARD;

pub struct CryptoContext {
    cipher: Aes256Gcm,
    salt: Vec<u8>,
}

impl CryptoContext {
    pub fn new(master_password: &str, salt: Vec<u8>) -> Result<Self, String> {
        if salt.len() < 8 {
            return Err("salt 长度不足（至少 8 字节）".to_string());
        }
        let mut key = vec![0u8; KEY_LEN];
        Argon2::default()
            .hash_password_into(master_password.as_bytes(), &salt, &mut key)
            .map_err(|e| format!("密钥派生失败: {}", e))?;
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| format!("加密初始化失败: {}", e))?;
        Ok(Self { cipher, salt })
    }

    pub fn encrypt(&self, plaintext: &str) -> Result<String, String> {
        let mut nonce_bytes = [0u8; NONCE_LEN];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self.cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| format!("加密失败: {}", e))?;

        Ok(format!(
            "{}:{}:{}",
            B64.encode(&self.salt),
            B64.encode(nonce_bytes),
            B64.encode(&ciphertext)
        ))
    }

    pub fn decrypt(&self, encrypted: &str) -> Result<String, String> {
        let parts: Vec<&str> = encrypted.splitn(3, ':').collect();
        if parts.len() != 3 {
            return Err("密文格式错误".to_string());
        }

        // 验证 salt 一致性
        let embedded_salt = B64.decode(parts[0]).map_err(|_| "salt 解码失败")?;
        if embedded_salt != self.salt {
            return Err("salt 不匹配：数据可能来自其他设备".to_string());
        }

        let nonce_bytes = B64.decode(parts[1]).map_err(|_| "nonce 解码失败")?;
        let ciphertext = B64.decode(parts[2]).map_err(|_| "密文解码失败")?;

        if nonce_bytes.len() != NONCE_LEN {
            return Err("nonce 长度错误".to_string());
        }

        let nonce = Nonce::from_slice(&nonce_bytes);
        let plaintext = self.cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|_| "解密失败：主密码错误或数据已损坏")?;

        String::from_utf8(plaintext).map_err(|_| "解密结果非有效 UTF-8".to_string())
    }
}

pub fn generate_salt() -> Vec<u8> {
    let mut salt = vec![0u8; SALT_LEN];
    OsRng.fill_bytes(&mut salt);
    salt
}

fn salt_path() -> PathBuf {
    crate::paths::local_dir().join("crypto_salt")
}

pub fn load_salt() -> Option<Vec<u8>> {
    let content = std::fs::read_to_string(salt_path()).ok()?;
    let salt = B64.decode(content.trim()).ok()?;
    if salt.len() < 8 { None } else { Some(salt) }
}

pub fn save_salt(salt: &[u8]) -> Result<(), String> {
    let encoded = B64.encode(salt);
    std::fs::write(salt_path(), encoded).map_err(|e| format!("保存 salt 失败: {}", e))
}

pub fn has_salt() -> bool {
    salt_path().exists()
}

pub fn restore_salt_from_sync(salt_b64: &str) -> Result<(), String> {
    if has_salt() {
        return Ok(());
    }
    let salt = B64.decode(salt_b64).map_err(|_| "同步 salt 解码失败")?;
    if salt.len() < 8 {
        return Err("同步 salt 长度不足".to_string());
    }
    save_salt(&salt)
}
