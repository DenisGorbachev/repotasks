use age::decrypt;
use age::x25519::Identity;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use errgonomic::handle;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use toml::Value;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FnoxSecret {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync: Option<Value>,
}

impl FnoxSecret {
    pub fn needs_reencryption(&self, identity: &Identity) -> Result<bool, FnoxSecretNeedsReencryptionError> {
        use FnoxSecretNeedsReencryptionError::*;
        match self {
            Self {
                provider: Some(provider),
                value: Some(value),
                sync,
            } if provider == "age" => {
                let ciphertext = handle!(STANDARD.decode(value), DecodeFailed, value);
                Ok(sync.is_some() || decrypt(identity, &ciphertext).is_err())
            }
            _ => Ok(false),
        }
    }
}

#[derive(Error, Debug)]
pub enum FnoxSecretNeedsReencryptionError {
    #[error("failed to decode the base64 secret value")]
    DecodeFailed { source: base64::DecodeError, value: String },
}
