use crate::{FNOX_CONFIG, PendingKeysError, RunInTerminalError, TEST_PROFILE, pending_keys, run_in_terminal};
use age::x25519::Identity;
use errgonomic::{handle, handle_bool};
use thiserror::Error;
use xshell::{Shell, cmd};

pub fn reencrypt_test_secrets(sh: &Shell, identity: &Identity) -> Result<(), ReencryptTestSecretsError> {
    use ReencryptTestSecretsError::*;
    let keys = handle!(pending_keys(identity), PendingKeysFailed);
    if keys.is_empty() {
        return Ok(());
    }
    handle!(run_in_terminal(cmd!(sh, "fnox --config {FNOX_CONFIG} --profile {TEST_PROFILE} reencrypt --force --provider age -- {keys...}")), FnoxReencryptFailed);
    let unreadable = handle!(pending_keys(identity), PendingKeysAfterReencryptFailed);
    handle_bool!(!unreadable.is_empty(), RecipientsInvalid, unreadable);
    Ok(())
}

#[derive(Error, Debug)]
pub enum ReencryptTestSecretsError {
    #[error("failed to find the test secrets that the CI age key cannot decrypt")]
    PendingKeysFailed { source: PendingKeysError },
    #[error("failed to run fnox reencrypt")]
    FnoxReencryptFailed { source: RunInTerminalError },
    #[error("failed to verify the re-encrypted test secrets")]
    PendingKeysAfterReencryptFailed { source: PendingKeysError },
    #[error("the CI age key cannot decrypt {keys}; check the test age recipients in '{FNOX_CONFIG}'", keys = unreadable.join(", "))]
    RecipientsInvalid { unreadable: Vec<String> },
}
