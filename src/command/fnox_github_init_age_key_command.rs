use crate::{CommitFnoxConfigIfChangedError, ConfigureTestAgeEncryptionError, FNOX_AGE_KEY, FNOX_CONFIG, GetOrCreateCiAgeIdentityError, GithubRepo, GithubRepoOriginError, GithubRepoSetSecretError, OpVault, RequireCleanRepoError, commit_fnox_config_if_changed, configure_test_age_encryption, get_or_create_ci_age_identity, require_clean_repo};
use clap::Parser;
use errgonomic::handle;
use std::process::ExitCode;
use thiserror::Error;
use xshell::{Shell, cmd};

#[derive(Parser, Clone, Debug)]
#[command(flatten_help = true)]
pub struct FnoxGithubInitAgeKeyCommand {
    /// 1Password vault for the repository key.
    #[arg(long, default_value = "Repos")]
    vault: OpVault,
}

impl FnoxGithubInitAgeKeyCommand {
    pub async fn run(self) -> Result<ExitCode, FnoxGithubInitAgeKeyCommandRunError> {
        use FnoxGithubInitAgeKeyCommandRunError::*;
        let Self {
            vault,
        } = self;
        let sh = handle!(Shell::new(), ShellNewFailed);
        let root = handle!(cmd!(sh, "git rev-parse --show-toplevel").read(), GitRevParseFailed);
        sh.change_dir(root);
        handle!(require_clean_repo(&sh), RequireCleanRepoFailed);
        let repo = handle!(GithubRepo::origin(&sh), OriginFailed);
        let identity = handle!(get_or_create_ci_age_identity(&sh, &vault, &repo), GetOrCreateCiAgeIdentityFailed, vault, repo);
        handle!(configure_test_age_encryption(&sh, &identity), ConfigureTestAgeEncryptionFailed);
        handle!(commit_fnox_config_if_changed(&sh), CommitFnoxConfigIfChangedFailed);
        handle!(repo.set_secret(&sh, FNOX_AGE_KEY, &identity.to_string()), SetSecretFailed, repo);
        Ok(ExitCode::SUCCESS)
    }
}

#[derive(Error, Debug)]
pub enum FnoxGithubInitAgeKeyCommandRunError {
    #[error("failed to create a shell")]
    ShellNewFailed { source: xshell::Error },
    #[error("failed to find the root of the Git repository")]
    GitRevParseFailed { source: xshell::Error },
    #[error("repository is not ready for this task")]
    RequireCleanRepoFailed { source: RequireCleanRepoError },
    #[error("failed to resolve the GitHub repository of the origin remote")]
    OriginFailed { source: GithubRepoOriginError },
    #[error("failed to get or create the CI age key for '{}' in 1Password vault '{}'", repo.0, vault.0)]
    GetOrCreateCiAgeIdentityFailed { source: Box<GetOrCreateCiAgeIdentityError>, vault: OpVault, repo: Box<GithubRepo> },
    #[error("failed to configure the test age encryption in '{FNOX_CONFIG}'")]
    ConfigureTestAgeEncryptionFailed { source: Box<ConfigureTestAgeEncryptionError> },
    #[error("failed to commit '{FNOX_CONFIG}'")]
    CommitFnoxConfigIfChangedFailed { source: CommitFnoxConfigIfChangedError },
    #[error("failed to store the CI age key in '{}'", repo.0)]
    SetSecretFailed { source: GithubRepoSetSecretError, repo: Box<GithubRepo> },
}
