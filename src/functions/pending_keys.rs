use crate::{FNOX_CONFIG, FnoxConfig, FnoxConfigSecretsToReencryptError, TEST_PROFILE};
use age::x25519::Identity;
use errgonomic::{ErrVec, handle, handle_iter};
use thiserror::Error;
use xshell::Shell;

pub fn pending_keys(sh: &Shell, identity: &Identity) -> Result<Vec<String>, PendingKeysError> {
    use PendingKeysError::*;
    let contents = handle!(sh.read_file(FNOX_CONFIG), ReadFileFailed);
    let config: FnoxConfig = handle!(toml::from_str(&contents), FromStrFailed, contents);
    let keys = handle_iter!(config.secrets_to_reencrypt(TEST_PROFILE, identity), SecretsToReencryptFailed);
    Ok(keys.into_iter().map(String::from).collect())
}

#[derive(Error, Debug)]
pub enum PendingKeysError {
    #[error("failed to read '{FNOX_CONFIG}'")]
    ReadFileFailed { source: xshell::Error },
    #[error("failed to parse '{FNOX_CONFIG}'")]
    FromStrFailed { source: Box<toml::de::Error>, contents: String },
    #[error("failed to check {count} test secrets", count = source.len())]
    SecretsToReencryptFailed { source: ErrVec<FnoxConfigSecretsToReencryptError> },
}
