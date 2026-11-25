use crate::{FNOX_AGE_KEY, GithubRepo, GithubRepoSecretNamesError, OpItemAgeIdentityError, OpVault, OpVaultCreatePasswordItemError, OpVaultFindItemError};
use age::x25519::Identity;
use errgonomic::{handle, handle_bool, map_err};
use thiserror::Error;
use xshell::Shell;

pub fn get_or_create_ci_age_identity(sh: &Shell, vault: &OpVault, repo: &GithubRepo) -> Result<Identity, GetOrCreateCiAgeIdentityError> {
    use GetOrCreateCiAgeIdentityError::*;
    let url = repo.0.as_str();
    let title = url.strip_prefix("https://").unwrap_or(url);
    if let Some(item) = handle!(vault.find_item(sh, title), FindItemFailed) {
        return map_err!(item.age_identity(), AgeIdentityFailed);
    }
    let names = handle!(repo.secret_names(sh), SecretNamesFailed);
    handle_bool!(names.iter().any(|name| name == FNOX_AGE_KEY), ItemNotFound, title);
    let identity = Identity::generate();
    handle!(vault.create_password_item(sh, title, &identity.to_string()), CreatePasswordItemFailed);
    Ok(identity)
}

#[derive(Error, Debug)]
pub enum GetOrCreateCiAgeIdentityError {
    #[error("failed to find the 1Password item")]
    FindItemFailed { source: OpVaultFindItemError },
    #[error("failed to read the CI age key from the 1Password item")]
    AgeIdentityFailed { source: OpItemAgeIdentityError },
    #[error("failed to read the GitHub secret names")]
    SecretNamesFailed { source: GithubRepoSecretNamesError },
    #[error("GitHub already has {FNOX_AGE_KEY}, but 1Password has no item '{title}'; restore the item before retrying, or explicitly rotate the key")]
    ItemNotFound { title: String },
    #[error("failed to store a new CI age key in 1Password")]
    CreatePasswordItemFailed { source: OpVaultCreatePasswordItemError },
}
