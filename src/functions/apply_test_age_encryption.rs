use crate::{ReencryptTestSecretsError, UpdateRecipientsError, reencrypt_test_secrets, update_recipients};
use age::x25519::Identity;
use errgonomic::{handle, map_err};
use thiserror::Error;
use xshell::Shell;

pub fn apply_test_age_encryption(sh: &Shell, identity: &Identity) -> Result<(), ApplyTestAgeEncryptionError> {
    use ApplyTestAgeEncryptionError::*;
    handle!(update_recipients(identity), UpdateRecipientsFailed);
    map_err!(reencrypt_test_secrets(sh, identity), ReencryptTestSecretsFailed)
}

#[derive(Error, Debug)]
pub enum ApplyTestAgeEncryptionError {
    #[error("failed to add the CI age key to the test age recipients")]
    UpdateRecipientsFailed { source: UpdateRecipientsError },
    #[error("failed to re-encrypt the test secrets")]
    ReencryptTestSecretsFailed { source: ReencryptTestSecretsError },
}
