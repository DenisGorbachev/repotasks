use crate::{ApplyTestAgeEncryptionError, FNOX_CONFIG, RunInTerminalError, apply_test_age_encryption, run_in_terminal};
use age::x25519::Identity;
use errgonomic::handle;
use thiserror::Error;
use xshell::{Shell, cmd};

pub fn configure_test_age_encryption(sh: &Shell, identity: &Identity) -> Result<(), ConfigureTestAgeEncryptionError> {
    use ConfigureTestAgeEncryptionError::*;
    if let Err(error) = apply_test_age_encryption(sh, identity) {
        handle!(run_in_terminal(cmd!(sh, "git restore --worktree -- {FNOX_CONFIG}")), GitRestoreFailed, apply_error: error);
        return Err(ApplyTestAgeEncryptionFailed {
            source: Box::new(error),
        });
    }
    Ok(())
}

#[derive(Error, Debug)]
pub enum ConfigureTestAgeEncryptionError {
    #[error("failed to apply the test age encryption")]
    ApplyTestAgeEncryptionFailed { source: Box<ApplyTestAgeEncryptionError> },
    #[error("failed to restore '{FNOX_CONFIG}' after a failed update: {apply_error}")]
    GitRestoreFailed { source: RunInTerminalError, apply_error: Box<ApplyTestAgeEncryptionError> },
}
