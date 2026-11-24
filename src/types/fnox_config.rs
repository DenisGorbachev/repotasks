use crate::{FnoxProfile, FnoxSecretNeedsReencryptionError};
use age::x25519::Identity;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FnoxConfig {
    #[serde(default)]
    pub profiles: BTreeMap<String, FnoxProfile>,
}

impl FnoxConfig {
    pub fn secrets_to_reencrypt<'a>(&'a self, profile: &str, identity: &'a Identity) -> impl Iterator<Item = Result<&'a str, FnoxConfigSecretsToReencryptError>> + use<'a> {
        use FnoxConfigSecretsToReencryptError::*;
        self.profiles
            .get(profile)
            .into_iter()
            .flat_map(|profile| &profile.secrets)
            .filter_map(move |(name, secret)| match secret.needs_reencryption(identity) {
                Ok(needed) => needed.then_some(Ok(name.as_str())),
                Err(source) => Some(Err(NeedsReencryptionFailed {
                    source,
                    name: name.clone(),
                })),
            })
    }
}

#[derive(Error, Debug)]
pub enum FnoxConfigSecretsToReencryptError {
    #[error("failed to check whether secret '{name}' needs re-encryption")]
    NeedsReencryptionFailed { source: FnoxSecretNeedsReencryptionError, name: String },
}
