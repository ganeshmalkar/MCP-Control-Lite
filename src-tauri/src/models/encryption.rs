use serde::{Deserialize, Serialize, Deserializer, Serializer};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use base64::{Engine as _, engine::general_purpose};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce, Key
};
use ring::{pbkdf2, rand::SecureRandom};
use std::num::NonZeroU32;

/// Encrypted field wrapper that handles automatic encryption/decryption
#[derive(Debug, Clone)]
pub struct EncryptedField<T> {
    /// The encrypted data (base64 encoded)
    encrypted_data: Option<String>,
    /// The plaintext data (only available in memory)
    plaintext_data: Option<T>,
    /// Encryption metadata
    metadata: EncryptionMetadata,
    /// Whether the field is currently decrypted
    is_decrypted: bool,
}

/// Metadata about encryption for a field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionMetadata {
    /// Encryption algorithm used
    pub algorithm: EncryptionAlgorithm,
    /// Key identifier used for encryption
    pub key_id: String,
    /// Initialization vector (base64 encoded)
    pub iv: String,
    /// When the field was encrypted
    pub encrypted_at: DateTime<Utc>,
    /// Salt used for key derivation (if applicable)
    pub salt: Option<String>,
    /// Additional authenticated data
    pub aad: Option<String>,
}

/// Supported encryption algorithms
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EncryptionAlgorithm {
    /// AES-256-GCM encryption
    Aes256Gcm,
    /// ChaCha20-Poly1305 encryption
    ChaCha20Poly1305,
    /// No encryption (for non-sensitive data)
    None,
}

/// Encryption key information
#[derive(Debug, Clone)]
pub struct EncryptionKey {
    /// Unique identifier for this key
    pub id: String,
    /// The actual key material (32 bytes for AES-256)
    pub key_material: Vec<u8>,
    /// Algorithm this key is used for
    pub algorithm: EncryptionAlgorithm,
    /// When this key was created
    pub created_at: DateTime<Utc>,
    /// When this key expires (if applicable)
    pub expires_at: Option<DateTime<Utc>>,
    /// Whether this key is active
    pub is_active: bool,
}

/// Key derivation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyDerivationConfig {
    /// PBKDF2 iteration count
    pub iterations: u32,
    /// Salt length in bytes
    pub salt_length: usize,
    /// Derived key length in bytes
    pub key_length: usize,
}

/// Encryption service for managing field-level encryption
pub struct EncryptionService {
    /// Available encryption keys
    keys: HashMap<String, EncryptionKey>,
    /// Default key ID to use for new encryptions
    default_key_id: Option<String>,
    /// Key derivation configuration
    key_derivation: KeyDerivationConfig,
}

/// Error types for encryption operations
#[derive(Debug, thiserror::Error)]
pub enum EncryptionError {
    #[error("Key not found: {key_id}")]
    KeyNotFound { key_id: String },
    
    #[error("Invalid key material: {reason}")]
    InvalidKey { reason: String },
    
    #[error("Encryption failed: {reason}")]
    EncryptionFailed { reason: String },
    
    #[error("Decryption failed: {reason}")]
    DecryptionFailed { reason: String },
    
    #[error("Serialization error: {reason}")]
    SerializationError { reason: String },
    
    #[error("Key derivation failed: {reason}")]
    KeyDerivationFailed { reason: String },
    
    #[error("Algorithm not supported: {algorithm:?}")]
    UnsupportedAlgorithm { algorithm: EncryptionAlgorithm },
}

impl Default for KeyDerivationConfig {
    fn default() -> Self {
        Self {
            iterations: 100_000, // OWASP recommended minimum
            salt_length: 32,     // 256 bits
            key_length: 32,      // 256 bits for AES-256
        }
    }
}

impl<T> EncryptedField<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Clone,
{
    /// Create a new encrypted field with plaintext data
    pub fn new(data: T) -> Self {
        Self {
            encrypted_data: None,
            plaintext_data: Some(data),
            metadata: EncryptionMetadata {
                algorithm: EncryptionAlgorithm::None,
                key_id: String::new(),
                iv: String::new(),
                encrypted_at: Utc::now(),
                salt: None,
                aad: None,
            },
            is_decrypted: true,
        }
    }
    
    /// Create an encrypted field from encrypted data
    pub fn from_encrypted(encrypted_data: String, metadata: EncryptionMetadata) -> Self {
        Self {
            encrypted_data: Some(encrypted_data),
            plaintext_data: None,
            metadata,
            is_decrypted: false,
        }
    }
    
    /// Encrypt the field data
    pub fn encrypt(&mut self, service: &EncryptionService) -> Result<(), EncryptionError> {
        if let Some(plaintext) = &self.plaintext_data {
            let serialized = serde_json::to_vec(plaintext)
                .map_err(|e| EncryptionError::SerializationError { 
                    reason: e.to_string() 
                })?;
            
            let (encrypted_data, metadata) = service.encrypt_bytes(&serialized)?;
            
            self.encrypted_data = Some(encrypted_data);
            self.metadata = metadata;
            self.plaintext_data = None; // Clear plaintext for security
            self.is_decrypted = false;
        }
        
        Ok(())
    }
    
    /// Decrypt the field data
    pub fn decrypt(&mut self, service: &EncryptionService) -> Result<(), EncryptionError> {
        if let Some(encrypted_data) = &self.encrypted_data {
            let decrypted_bytes = service.decrypt_bytes(encrypted_data, &self.metadata)?;
            
            let plaintext: T = serde_json::from_slice(&decrypted_bytes)
                .map_err(|e| EncryptionError::SerializationError { 
                    reason: e.to_string() 
                })?;
            
            self.plaintext_data = Some(plaintext);
            self.is_decrypted = true;
        }
        
        Ok(())
    }
    
    /// Get the plaintext data (requires decryption first)
    pub fn get(&self) -> Option<&T> {
        self.plaintext_data.as_ref()
    }
    
    /// Get mutable reference to plaintext data
    pub fn get_mut(&mut self) -> Option<&mut T> {
        if self.plaintext_data.is_some() {
            self.encrypted_data = None; // Mark as needing re-encryption
        }
        self.plaintext_data.as_mut()
    }
    
    /// Set new plaintext data
    pub fn set(&mut self, data: T) {
        self.plaintext_data = Some(data);
        self.encrypted_data = None; // Mark as needing re-encryption
        self.is_decrypted = true;
    }
    
    /// Check if the field is currently decrypted
    pub fn is_decrypted(&self) -> bool {
        self.is_decrypted
    }
    
    /// Check if the field has encrypted data
    pub fn is_encrypted(&self) -> bool {
        self.encrypted_data.is_some()
    }
    
    /// Get encryption metadata
    pub fn metadata(&self) -> &EncryptionMetadata {
        &self.metadata
    }
}

impl Default for EncryptionService {
    fn default() -> Self {
        Self::new()
    }
}

impl EncryptionService {
    /// Create a new encryption service
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
            default_key_id: None,
            key_derivation: KeyDerivationConfig::default(),
        }
    }
    
    /// Add an encryption key
    pub fn add_key(&mut self, key: EncryptionKey) {
        let key_id = key.id.clone();
        self.keys.insert(key_id.clone(), key);
        
        // Set as default if no default is set
        if self.default_key_id.is_none() {
            self.default_key_id = Some(key_id);
        }
    }
    
    /// Generate a new encryption key
    pub fn generate_key(&mut self, algorithm: EncryptionAlgorithm) -> Result<String, EncryptionError> {
        let key_id = Uuid::new_v4().to_string();
        
        let key_material = match algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                let mut key_bytes = vec![0u8; 32]; // 256 bits
                ring::rand::SystemRandom::new()
                    .fill(&mut key_bytes)
                    .map_err(|_| EncryptionError::KeyDerivationFailed { 
                        reason: "Failed to generate random key".to_string() 
                    })?;
                key_bytes
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                let mut key_bytes = vec![0u8; 32]; // 256 bits
                ring::rand::SystemRandom::new()
                    .fill(&mut key_bytes)
                    .map_err(|_| EncryptionError::KeyDerivationFailed { 
                        reason: "Failed to generate random key".to_string() 
                    })?;
                key_bytes
            }
            EncryptionAlgorithm::None => {
                return Err(EncryptionError::UnsupportedAlgorithm { algorithm });
            }
        };
        
        let key = EncryptionKey {
            id: key_id.clone(),
            key_material,
            algorithm,
            created_at: Utc::now(),
            expires_at: None,
            is_active: true,
        };
        
        self.add_key(key);
        Ok(key_id)
    }
    
    /// Derive a key from a password
    pub fn derive_key_from_password(
        &self,
        password: &str,
        salt: &[u8],
    ) -> Result<Vec<u8>, EncryptionError> {
        let mut key = vec![0u8; self.key_derivation.key_length];
        
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            NonZeroU32::new(self.key_derivation.iterations).unwrap(),
            salt,
            password.as_bytes(),
            &mut key,
        );
        
        Ok(key)
    }
    
    /// Encrypt bytes using the default key
    pub fn encrypt_bytes(&self, data: &[u8]) -> Result<(String, EncryptionMetadata), EncryptionError> {
        let key_id = self.default_key_id.as_ref()
            .ok_or_else(|| EncryptionError::KeyNotFound { 
                key_id: "default".to_string() 
            })?;
        
        self.encrypt_bytes_with_key(data, key_id)
    }
    
    /// Encrypt bytes using a specific key
    pub fn encrypt_bytes_with_key(
        &self,
        data: &[u8],
        key_id: &str,
    ) -> Result<(String, EncryptionMetadata), EncryptionError> {
        let key = self.keys.get(key_id)
            .ok_or_else(|| EncryptionError::KeyNotFound { 
                key_id: key_id.to_string() 
            })?;
        
        match key.algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                let cipher_key = Key::<Aes256Gcm>::from_slice(&key.key_material);
                let cipher = Aes256Gcm::new(cipher_key);
                
                // Generate random nonce
                let mut nonce_bytes = [0u8; 12]; // 96 bits for GCM
                ring::rand::SystemRandom::new()
                    .fill(&mut nonce_bytes)
                    .map_err(|_| EncryptionError::EncryptionFailed { 
                        reason: "Failed to generate nonce".to_string() 
                    })?;
                
                let nonce = Nonce::from_slice(&nonce_bytes);
                
                let ciphertext = cipher.encrypt(nonce, data)
                    .map_err(|e| EncryptionError::EncryptionFailed { 
                        reason: e.to_string() 
                    })?;
                
                let encrypted_data = general_purpose::STANDARD.encode(&ciphertext);
                let iv = general_purpose::STANDARD.encode(nonce_bytes);
                
                let metadata = EncryptionMetadata {
                    algorithm: EncryptionAlgorithm::Aes256Gcm,
                    key_id: key_id.to_string(),
                    iv,
                    encrypted_at: Utc::now(),
                    salt: None,
                    aad: None,
                };
                
                Ok((encrypted_data, metadata))
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                // TODO: Implement ChaCha20-Poly1305
                Err(EncryptionError::UnsupportedAlgorithm { 
                    algorithm: EncryptionAlgorithm::ChaCha20Poly1305 
                })
            }
            EncryptionAlgorithm::None => {
                Err(EncryptionError::UnsupportedAlgorithm { 
                    algorithm: EncryptionAlgorithm::None 
                })
            }
        }
    }
    
    /// Decrypt bytes
    pub fn decrypt_bytes(
        &self,
        encrypted_data: &str,
        metadata: &EncryptionMetadata,
    ) -> Result<Vec<u8>, EncryptionError> {
        let key = self.keys.get(&metadata.key_id)
            .ok_or_else(|| EncryptionError::KeyNotFound { 
                key_id: metadata.key_id.clone() 
            })?;
        
        match metadata.algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                let cipher_key = Key::<Aes256Gcm>::from_slice(&key.key_material);
                let cipher = Aes256Gcm::new(cipher_key);
                
                let ciphertext = general_purpose::STANDARD.decode(encrypted_data)
                    .map_err(|e| EncryptionError::DecryptionFailed { 
                        reason: format!("Invalid base64: {}", e) 
                    })?;
                
                let nonce_bytes = general_purpose::STANDARD.decode(&metadata.iv)
                    .map_err(|e| EncryptionError::DecryptionFailed { 
                        reason: format!("Invalid IV: {}", e) 
                    })?;
                
                let nonce = Nonce::from_slice(&nonce_bytes);
                
                let plaintext = cipher.decrypt(nonce, ciphertext.as_ref())
                    .map_err(|e| EncryptionError::DecryptionFailed { 
                        reason: e.to_string() 
                    })?;
                
                Ok(plaintext)
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                Err(EncryptionError::UnsupportedAlgorithm { 
                    algorithm: EncryptionAlgorithm::ChaCha20Poly1305 
                })
            }
            EncryptionAlgorithm::None => {
                Err(EncryptionError::UnsupportedAlgorithm { 
                    algorithm: EncryptionAlgorithm::None 
                })
            }
        }
    }
    
    /// Rotate encryption keys
    pub fn rotate_key(&mut self, old_key_id: &str) -> Result<String, EncryptionError> {
        let old_key = self.keys.get(old_key_id)
            .ok_or_else(|| EncryptionError::KeyNotFound { 
                key_id: old_key_id.to_string() 
            })?;
        
        let algorithm = old_key.algorithm.clone();
        
        // Generate new key
        let new_key_id = self.generate_key(algorithm)?;
        
        // Deactivate old key
        if let Some(old_key) = self.keys.get_mut(old_key_id) {
            old_key.is_active = false;
        }
        
        // Update default key if necessary
        if self.default_key_id.as_deref() == Some(old_key_id) {
            self.default_key_id = Some(new_key_id.clone());
        }
        
        Ok(new_key_id)
    }
}

// Custom serialization for EncryptedField
impl<T> Serialize for EncryptedField<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Clone,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        
        let mut state = serializer.serialize_struct("EncryptedField", 2)?;
        state.serialize_field("encrypted_data", &self.encrypted_data)?;
        state.serialize_field("metadata", &self.metadata)?;
        state.end()
    }
}

// Custom deserialization for EncryptedField
impl<'de, T> Deserialize<'de> for EncryptedField<T>
where
    T: Serialize + for<'d> Deserialize<'d> + Clone,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct EncryptedFieldData {
            encrypted_data: Option<String>,
            metadata: EncryptionMetadata,
        }
        
        let data = EncryptedFieldData::deserialize(deserializer)?;
        
        Ok(EncryptedField {
            encrypted_data: data.encrypted_data,
            plaintext_data: None,
            metadata: data.metadata,
            is_decrypted: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encryption_service_key_generation() {
        let mut service = EncryptionService::new();
        let key_id = service.generate_key(EncryptionAlgorithm::Aes256Gcm).unwrap();
        
        assert!(service.keys.contains_key(&key_id));
        assert_eq!(service.default_key_id, Some(key_id));
    }
    
    #[test]
    fn test_encrypted_field_basic_operations() {
        let mut service = EncryptionService::new();
        service.generate_key(EncryptionAlgorithm::Aes256Gcm).unwrap();
        
        let test_data = "sensitive information".to_string();
        let mut field = EncryptedField::new(test_data.clone());
        
        // Encrypt the field
        field.encrypt(&service).unwrap();
        assert!(field.is_encrypted());
        assert!(!field.is_decrypted());
        
        // Decrypt the field
        field.decrypt(&service).unwrap();
        assert!(field.is_decrypted());
        assert_eq!(field.get(), Some(&test_data));
    }
    
    #[test]
    fn test_key_rotation() {
        let mut service = EncryptionService::new();
        let old_key_id = service.generate_key(EncryptionAlgorithm::Aes256Gcm).unwrap();
        
        let new_key_id = service.rotate_key(&old_key_id).unwrap();
        
        assert_ne!(old_key_id, new_key_id);
        assert_eq!(service.default_key_id, Some(new_key_id));
        
        let old_key = service.keys.get(&old_key_id).unwrap();
        assert!(!old_key.is_active);
    }
    
    #[test]
    fn test_password_key_derivation() {
        let service = EncryptionService::new();
        let password = "strong_password_123!";
        let salt = b"random_salt_data_here_32_bytes__";
        
        let key1 = service.derive_key_from_password(password, salt).unwrap();
        let key2 = service.derive_key_from_password(password, salt).unwrap();
        
        assert_eq!(key1, key2); // Same password and salt should produce same key
        assert_eq!(key1.len(), 32); // Should be 256 bits
    }
}
