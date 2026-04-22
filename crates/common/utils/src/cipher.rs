//! Encryption and Decryption - 加解密模块

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use hmac::{Hmac, Mac};
use rand::RngCore;
use sha2::{Sha256, Sha512};
use thiserror::Error;

/// Cipher 错误
#[derive(Error, Debug)]
pub enum CipherError {
    #[error("加密失败: {0}")]
    EncryptError(String),
    #[error("解密失败: {0}")]
    DecryptError(String),
    #[error("密钥派生失败: {0}")]
    KeyDerivationError(String),
    #[error("Base64 解码失败: {0}")]
    Base64Error(String),
    #[error("HMAC 验证失败")]
    HmacVerifyFailed,
}

/// 对称加密器 (AES-256-GCM)
pub struct AesCipher {
    cipher: Aes256Gcm,
}

impl AesCipher {
    /// 使用密钥创建加密器
    pub fn new(key: &[u8; 32]) -> Self {
        let cipher = Aes256Gcm::new_from_slice(key).expect("Valid key length");
        Self { cipher }
    }

    /// 从 Base64 密钥创建加密器
    pub fn from_base64_key(key_base64: &str) -> Result<Self, CipherError> {
        let key_bytes = BASE64
            .decode(key_base64)
            .map_err(|e| CipherError::Base64Error(e.to_string()))?;

        if key_bytes.len() != 32 {
            return Err(CipherError::KeyDerivationError(
                "Key must be 32 bytes".to_string(),
            ));
        }

        let mut key = [0u8; 32];
        key.copy_from_slice(&key_bytes);
        Ok(Self::new(&key))
    }

    /// 加密数据
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<String, CipherError> {
        // 生成随机 nonce (12 bytes)
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // 加密
        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| CipherError::EncryptError(e.to_string()))?;

        // 组合 nonce + ciphertext 并 Base64 编码
        let mut combined = nonce_bytes.to_vec();
        combined.extend(ciphertext);

        Ok(BASE64.encode(combined))
    }

    /// 解密数据
    pub fn decrypt(&self, encrypted: &str) -> Result<Vec<u8>, CipherError> {
        // Base64 解码
        let combined = BASE64
            .decode(encrypted)
            .map_err(|e| CipherError::Base64Error(e.to_string()))?;

        if combined.len() < 12 {
            return Err(CipherError::DecryptError("Invalid encrypted data".to_string()));
        }

        // 分离 nonce 和 ciphertext
        let (nonce_bytes, ciphertext) = combined.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        // 解密
        self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| CipherError::DecryptError(e.to_string()))
    }

    /// 加密字符串
    pub fn encrypt_string(&self, plaintext: &str) -> Result<String, CipherError> {
        self.encrypt(plaintext.as_bytes())
    }

    /// 解密为字符串
    pub fn decrypt_string(&self, encrypted: &str) -> Result<String, CipherError> {
        let bytes = self.decrypt(encrypted)?;
        String::from_utf8(bytes).map_err(|e| CipherError::DecryptError(e.to_string()))
    }
}

/// HMAC-SHA256
pub struct HmacSha256 {
    key: Vec<u8>,
}

impl HmacSha256 {
    /// 创建 HMAC-SHA256
    pub fn new(key: &[u8]) -> Self {
        Self {
            key: key.to_vec(),
        }
    }

    /// 生成 HMAC
    pub fn generate(&self, data: &[u8]) -> String {
        type HmacSha256 = Hmac<Sha256>;
        let mut mac = <HmacSha256 as Mac>::new_from_slice(&self.key)
            .expect("HMAC can take key of any size");
        mac.update(data);
        let result = mac.finalize();
        hex::encode(result.into_bytes())
    }

    /// 验证 HMAC
    pub fn verify(&self, data: &[u8], expected: &str) -> bool {
        let computed = self.generate(data);
        computed == expected
    }

    /// 生成 Base64 编码的 HMAC
    #[allow(dead_code)]
    pub fn generate_base64(&self, data: &[u8]) -> String {
        type HmacSha256 = Hmac<Sha256>;
        let mut mac = <HmacSha256 as Mac>::new_from_slice(&self.key)
            .expect("HMAC can take key of any size");
        mac.update(data);
        let result = mac.finalize();
        BASE64.encode(result.into_bytes())
    }
}

/// HMAC-SHA512
pub struct HmacSha512 {
    key: Vec<u8>,
}

impl HmacSha512 {
    /// 创建 HMAC-SHA512
    pub fn new(key: &[u8]) -> Self {
        Self {
            key: key.to_vec(),
        }
    }

    /// 生成 HMAC
    pub fn generate(&self, data: &[u8]) -> String {
        type HmacSha512 = Hmac<Sha512>;
        let mut mac = <HmacSha512 as Mac>::new_from_slice(&self.key)
            .expect("HMAC can take key of any size");
        mac.update(data);
        let result = mac.finalize();
        hex::encode(result.into_bytes())
    }

    /// 验证 HMAC
    pub fn verify(&self, data: &[u8], expected: &str) -> bool {
        let computed = self.generate(data);
        computed == expected
    }
}

/// SHA256 哈希
pub fn sha256(data: &[u8]) -> String {
    use sha2::Digest;
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// SHA512 哈希
pub fn sha512(data: &[u8]) -> String {
    use sha2::Digest;
    let mut hasher = Sha512::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Keccak-256 哈希 (用于 Ethereum)
pub fn keccak256(data: &[u8]) -> String {
    use sha3::{Digest, Keccak256};
    let mut hasher = Keccak256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// 密码派生 (PBKDF2-SHA256)
#[allow(dead_code)]
pub fn pbkdf2_sha256(password: &str, salt: &[u8], iterations: u32) -> Vec<u8> {
    use std::num::Wrapping;
    type HmacSha256 = Hmac<Sha256>;
    let mut output = vec![0u8; 32];
    let password_bytes = password.as_bytes();

    // 简化实现 - 使用 HMAC 作为 PRF
    for i in 0..iterations {
        let mut mac = <HmacSha256 as hmac::Mac>::new_from_slice(password_bytes).unwrap();
        mac.update(salt);
        mac.update(&(Wrapping(i) + Wrapping(1)).0.to_le_bytes());
        let result = mac.finalize();
        for (j, byte) in result.into_bytes().iter().enumerate() {
            output[j] ^= byte;
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_encrypt_decrypt() {
        let key = [0u8; 32];
        let cipher = AesCipher::new(&key);

        let plaintext = "Hello, World!";
        let encrypted = cipher.encrypt_string(plaintext).unwrap();
        let decrypted = cipher.decrypt_string(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_hmac_sha256() {
        let hmac = HmacSha256::new(b"secret_key");
        let data = b"test message";

        let signature = hmac.generate(data);
        assert!(hmac.verify(data, &signature));
        assert!(!hmac.verify(b"wrong data", &signature));
    }

    #[test]
    fn test_sha256() {
        let hash = sha256(b"hello");
        assert_eq!(
            hash,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn test_keccak256() {
        let hash = keccak256(b"hello");
        // Ethereum uses different test vector
        println!("Keccak256: {}", hash);
    }
}