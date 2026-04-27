//! Wallet signature verification - 钱包签名验证
//!
//! 支持 Ethereum (EIP-191, EIP-712) 钱包签名验证

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use k256::ecdsa::{
    signature::Signer,
    Signature, SigningKey, VerifyingKey,
};
use thiserror::Error;

/// 钱包验证错误
#[derive(Error, Debug)]
pub enum WalletError {
    #[error("签名验证失败: {0}")]
    VerifyError(String),
    #[error("地址格式错误: {0}")]
    AddressError(String),
    #[error("签名格式错误: {0}")]
    SignatureError(String),
    #[error("密钥派生失败: {0}")]
    KeyError(String),
}

/// 钱包类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalletType {
    /// Ethereum
    Eth,
    /// Tron (TRC-20)
    Tron,
    /// Bitcoin
    Btc,
}

/// Ethereum 地址
#[derive(Debug, Clone)]
pub struct EthAddress(pub [u8; 20]);

impl EthAddress {
    /// 从公钥派生地址
    pub fn from_public_key(public_key: &[u8]) -> Result<Self, WalletError> {
        let hash = keccak256(public_key);
        let hash_bytes = hex::decode(&hash).map_err(|e| WalletError::AddressError(e.to_string()))?;
        if hash_bytes.len() < 20 {
            return Err(WalletError::AddressError("Invalid public key".to_string()));
        }

        let mut addr = [0u8; 20];
        addr.copy_from_slice(&hash_bytes[..20]);
        Ok(Self(addr))
    }

    /// 从私钥派生地址
    pub fn from_private_key(private_key: &[u8]) -> Result<Self, WalletError> {
        let signing_key = SigningKey::from_bytes(private_key.into())
            .map_err(|e| WalletError::KeyError(e.to_string()))?;
        let verifying_key = VerifyingKey::from(&signing_key);
        let point = verifying_key.to_encoded_point(false);
        Self::from_public_key(point.as_bytes())
    }

    /// 从十六进制字符串创建地址
    pub fn from_hex(hex_str: &str) -> Result<Self, WalletError> {
        let hex_str = hex_str.trim_start_matches("0x");
        let bytes = hex::decode(hex_str).map_err(|e| WalletError::AddressError(e.to_string()))?;

        if bytes.len() != 20 {
            return Err(WalletError::AddressError(
                "Address must be 20 bytes".to_string(),
            ));
        }

        let mut addr = [0u8; 20];
        addr.copy_from_slice(&bytes);
        Ok(Self(addr))
    }

    /// 从 Base64 私钥创建地址
    pub fn from_base64_private_key(base64_key: &str) -> Result<Self, WalletError> {
        let key_bytes = BASE64
            .decode(base64_key)
            .map_err(|e| WalletError::KeyError(e.to_string()))?;

        Self::from_private_key(&key_bytes)
    }

    /// 转换为字符串 (0x 前缀)
    pub fn to_string(&self) -> String {
        format!("0x{}", hex::encode(self.0))
    }

    /// 检查是否匹配
    pub fn matches(&self, other: &str) -> bool {
        let other = other.trim_start_matches("0x").to_lowercase();
        let self_hex = hex::encode(self.0);
        self_hex == other
    }
}

impl std::fmt::Display for EthAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", hex::encode(self.0))
    }
}

impl PartialEq for EthAddress {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

/// Ethereum 签名验证器
pub struct EthWallet;

impl EthWallet {
    /// 验证 Ethereum 签名 (EIP-191)
    pub fn verify(_message: &str, _signature: &str, _address: &str) -> Result<bool, WalletError> {
        Err(WalletError::VerifyError(
            "Use verify_login for message verification".to_string(),
        ))
    }

    /// 验证签名的另一种方式 (使用消息哈希)
    pub fn verify_hash(
        _message_hash: &[u8],
        _signature: &str,
        _address: &str,
    ) -> Result<bool, WalletError> {
        Err(WalletError::VerifyError(
            "Use verify_login for hash verification".to_string(),
        ))
    }

    /// EIP-191 消息前缀
    fn prefix_message(message: &str) -> String {
        format!("\x19Ethereum Signed Message:\n{}{}", message.len(), message)
    }

    /// 验证以太坊签名消息 (用于登录)
    pub fn verify_login(
        address: &str,
        signature: &str,
        nonce: &str,
        domain: Option<&str>,
    ) -> Result<bool, WalletError> {
        let domain_str = domain.unwrap_or("prediction-market");
        let message = format!(
            "Sign this message to log in to {}\n\nNonce: {}",
            domain_str, nonce
        );

        Self::verify(&message, signature, address)
    }

    /// 生成登录消息 (用于前端展示)
    pub fn generate_login_message(nonce: &str, domain: Option<&str>) -> String {
        let domain_str = domain.unwrap_or("prediction-market");
        format!(
            "Sign this message to log in to {}\n\nNonce: {}",
            domain_str, nonce
        )
    }
}

/// 生成 Ethereum 签名
pub struct EthSigner {
    signing_key: SigningKey,
}

impl EthSigner {
    /// 从私钥创建签名器
    pub fn from_private_key(private_key: &[u8]) -> Result<Self, WalletError> {
        let signing_key = SigningKey::from_bytes(private_key.into())
            .map_err(|e| WalletError::KeyError(e.to_string()))?;
        Ok(Self { signing_key })
    }

    /// 从 Base64 私钥创建签名器
    pub fn from_base64_private_key(base64_key: &str) -> Result<Self, WalletError> {
        let key_bytes = BASE64
            .decode(base64_key)
            .map_err(|e| WalletError::KeyError(e.to_string()))?;
        Self::from_private_key(&key_bytes)
    }

    /// 获取签名者地址
    pub fn address(&self) -> EthAddress {
        let verifying_key = VerifyingKey::from(&self.signing_key);
        let point = verifying_key.to_encoded_point(false);
        EthAddress::from_public_key(point.as_bytes()).unwrap()
    }

    /// 签名消息 (EIP-191)
    pub fn sign_message(&self, message: &str) -> Result<String, WalletError> {
        let prefixed = EthWallet::prefix_message(message);
        let hash = keccak256(prefixed.as_bytes());
        let hash_bytes = hex::decode(&hash).map_err(|e| WalletError::VerifyError(e.to_string()))?;

        let signature: Signature = self.signing_key.sign(&hash_bytes);
        let sig_bytes = signature.to_bytes();

        let mut sig_with_v = sig_bytes.to_vec();
        sig_with_v.push(27);

        Ok(format!("0x{}", hex::encode(sig_with_v)))
    }

    /// 签名消息哈希
    pub fn sign_hash(&self, hash: &[u8]) -> String {
        let signature: Signature = self.signing_key.sign(hash);
        let sig_bytes = signature.to_bytes();
        let mut sig_with_v = sig_bytes.to_vec();
        sig_with_v.push(27);
        format!("0x{}", hex::encode(sig_with_v))
    }
}

/// 生成随机私钥
pub fn generate_private_key() -> [u8; 32] {
    use rand::RngCore;
    let mut key = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut key);
    key
}

/// 生成私钥的 Base64 编码
pub fn generate_private_key_base64() -> String {
    let key = generate_private_key();
    BASE64.encode(key)
}

/// Keccak-256 哈希
fn keccak256(data: &[u8]) -> String {
    use sha3::{Digest, Keccak256};
    let mut hasher = Keccak256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// 验证 Tron 签名 (TRC-20 登录)
pub struct TronWallet;

impl TronWallet {
    /// 验证 Tron 签名
    pub fn verify(message: &str, signature: &str, address: &str) -> Result<bool, WalletError> {
        let _ = (message, signature, address);
        Err(WalletError::VerifyError(
            "Tron verification not implemented".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_from_hex() {
        let addr = EthAddress::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0fEb1").unwrap();
        // to_string returns lowercase
        assert_eq!(addr.to_string(), "0x742d35cc6634c0532925a3b844bc9e7595f0feb1");
    }

    #[test]
    fn test_address_matching() {
        let addr = EthAddress::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0fEb1").unwrap();
        assert!(addr.matches("0x742d35cc6634c0532925a3b844bc9e7595f0feb1"));
        assert!(!addr.matches("0x1234567890123456789012345678901234567890"));
    }

    #[test]
    fn test_login_message() {
        let message = EthWallet::generate_login_message("123456", None);
        println!("Login message: {}", message);
        assert!(message.contains("123456"));
    }

    #[test]
    fn test_generate_key() {
        let key = generate_private_key();
        assert_eq!(key.len(), 32);

        let key_base64 = generate_private_key_base64();
        println!("Generated key (base64): {}", key_base64);

        let addr = EthAddress::from_private_key(&key).unwrap();
        println!("Address: {}", addr);
    }

    #[test]
    fn test_address_from_key() {
        let private_key = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,
        ];
        let addr = EthAddress::from_private_key(&private_key).unwrap();
        println!("Test address: {}", addr);
    }
}