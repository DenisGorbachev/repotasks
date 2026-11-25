use crate::{FNOX_CONFIG, FnoxConfig, FnoxConfigSecretsToReencryptError, TEST_PROFILE};
use age::x25519::Identity;
use errgonomic::{ErrVec, handle, handle_iter};
use std::{fs, io};
use thiserror::Error;

pub fn pending_keys(identity: &Identity) -> Result<Vec<String>, PendingKeysError> {
    use PendingKeysError::*;
    let contents = handle!(fs::read_to_string(FNOX_CONFIG), ReadToStringFailed);
    let config: FnoxConfig = handle!(toml::from_str(&contents), FromStrFailed, contents);
    let keys = handle_iter!(config.secrets_to_reencrypt(TEST_PROFILE, identity), SecretsToReencryptFailed);
    Ok(keys.into_iter().map(String::from).collect())
}

#[derive(Error, Debug)]
pub enum PendingKeysError {
    #[error("failed to read '{FNOX_CONFIG}'")]
    ReadToStringFailed { source: io::Error },
    #[error("failed to parse '{FNOX_CONFIG}'")]
    FromStrFailed { source: Box<toml::de::Error>, contents: String },
    #[error("failed to check {count} test secrets", count = source.len())]
    SecretsToReencryptFailed { source: ErrVec<FnoxConfigSecretsToReencryptError> },
}
