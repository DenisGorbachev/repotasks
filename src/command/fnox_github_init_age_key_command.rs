use crate::{AGENTS_MD, FNOX_AGE_KEY, FNOX_CONFIG, FnoxConfig, FnoxConfigSecretsToReencryptError, GithubRepo, GithubRepoOriginError, GithubRepoSecretNamesError, GithubRepoSetSecretError, OpItemAgeIdentityError, OpVault, OpVaultCreatePasswordItemError, OpVaultFindItemError, TEST_PROFILE, get_toml_item_mut, push_unique_toml_string};
use age::x25519::Identity;
use clap::Parser;
use errgonomic::{ErrVec, handle, handle_bool, handle_iter, map_err};
use std::process::ExitCode;
use std::{fs, io};
use thiserror::Error;
use toml_edit::{DocumentMut, Item};
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
        handle!(require_clean_repo(&sh), RequireCleanRepoFailed);
        let repo = handle!(GithubRepo::origin(&sh), OriginFailed);
        let identity = handle!(get_or_create_ci_age_identity(&sh, &vault, &repo), GetOrCreateCiAgeIdentityFailed, vault, repo);
        handle!(configure_test_age_encryption(&sh, &identity), ConfigureTestAgeEncryptionFailed);
        handle!(commit_fnox_config_if_changed(&sh), CommitFnoxConfigIfChangedFailed);
        handle!(repo.set_secret(&sh, FNOX_AGE_KEY, &identity.to_string()), SetSecretFailed, repo);
        Ok(ExitCode::SUCCESS)
    }
}

pub fn require_clean_repo(sh: &Shell) -> Result<(), RequireCleanRepoError> {
    use RequireCleanRepoError::*;
    let status = handle!(cmd!(sh, "git status --porcelain=v1 --untracked-files=normal --ignore-submodules=none").read(), GitStatusFailed);
    handle_bool!(!status.is_empty(), WorktreeInvalid, status);
    let tracked = handle!(cmd!(sh, "git ls-files -v --error-unmatch -- {FNOX_CONFIG} {AGENTS_MD} .").read(), GitLsFilesFailed);
    handle_bool!(
        tracked
            .lines()
            .any(|line| line.starts_with(|tag: char| tag == 'S' || tag.is_ascii_lowercase())),
        IndexFlagsInvalid,
        tracked
    );
    Ok(())
}

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

pub fn configure_test_age_encryption(sh: &Shell, identity: &Identity) -> Result<(), ConfigureTestAgeEncryptionError> {
    use ConfigureTestAgeEncryptionError::*;
    handle!(cmd!(sh, "git diff --exit-code --name-only HEAD -- {FNOX_CONFIG} {AGENTS_MD}").run(), GitDiffFailed);
    if let Err(error) = apply_test_age_encryption(sh, identity) {
        handle!(cmd!(sh, "git restore --worktree -- {FNOX_CONFIG}").run(), GitRestoreFailed, apply_error: error);
        return Err(ApplyTestAgeEncryptionFailed {
            source: Box::new(error),
        });
    }
    Ok(())
}

pub fn apply_test_age_encryption(sh: &Shell, identity: &Identity) -> Result<(), ApplyTestAgeEncryptionError> {
    use ApplyTestAgeEncryptionError::*;
    handle!(update_recipients(identity), UpdateRecipientsFailed);
    map_err!(reencrypt_test_secrets(sh, identity), ReencryptTestSecretsFailed)
}

pub fn update_recipients(identity: &Identity) -> Result<(), UpdateRecipientsError> {
    use UpdateRecipientsError::*;
    let recipient = identity.to_public().to_string();
    let contents = handle!(fs::read_to_string(FNOX_CONFIG), ReadToStringFailed);
    let mut doc: DocumentMut = handle!(contents.parse(), ParseFailed, contents);
    let has_test_age = get_toml_item_mut(doc.as_item_mut(), ["profiles", TEST_PROFILE, "providers", "age"]).is_some();
    let path: &[&str] = if has_test_age {
        &["profiles", TEST_PROFILE, "providers", "age", "recipients"]
    } else {
        &["providers", "age", "recipients"]
    };
    let added = get_toml_item_mut(doc.as_item_mut(), path)
        .and_then(Item::as_array_mut)
        .is_some_and(|recipients| push_unique_toml_string(recipients, &recipient));
    if added {
        handle!(fs::write(FNOX_CONFIG, doc.to_string()), WriteFailed);
    }
    Ok(())
}

pub fn reencrypt_test_secrets(sh: &Shell, identity: &Identity) -> Result<(), ReencryptTestSecretsError> {
    use ReencryptTestSecretsError::*;
    let keys = handle!(pending_keys(identity), PendingKeysFailed);
    if keys.is_empty() {
        return Ok(());
    }
    handle!(cmd!(sh, "fnox --config {FNOX_CONFIG} --profile {TEST_PROFILE} reencrypt --force --provider age -- {keys...}").run(), FnoxReencryptFailed);
    let unreadable = handle!(pending_keys(identity), PendingKeysAfterReencryptFailed);
    handle_bool!(!unreadable.is_empty(), RecipientsInvalid, unreadable);
    Ok(())
}

pub fn pending_keys(identity: &Identity) -> Result<Vec<String>, PendingKeysError> {
    use PendingKeysError::*;
    let contents = handle!(fs::read_to_string(FNOX_CONFIG), ReadToStringFailed);
    let config: FnoxConfig = handle!(toml::from_str(&contents), FromStrFailed, contents);
    let keys = handle_iter!(config.secrets_to_reencrypt(TEST_PROFILE, identity), SecretsToReencryptFailed);
    Ok(keys.into_iter().map(String::from).collect())
}

pub fn commit_fnox_config_if_changed(sh: &Shell) -> Result<(), CommitFnoxConfigIfChangedError> {
    use CommitFnoxConfigIfChangedError::*;
    let changed = handle!(cmd!(sh, "git diff --name-only -- {FNOX_CONFIG}").read(), GitDiffFnoxConfigFailed);
    if changed.is_empty() {
        return Ok(());
    }
    let files = [FNOX_CONFIG, AGENTS_MD];
    handle!(cmd!(sh, "git diff --exit-code --name-only HEAD -- {AGENTS_MD}").run(), GitDiffAgentsMdFailed);
    if let Err(error) = cmd!(sh, "mise run gen:agents").run() {
        handle!(cmd!(sh, "git restore --worktree -- {files...}").run(), GitRestoreFailed, gen_agents_error: error);
        return Err(MiseRunGenAgentsFailed {
            source: error,
        });
    }
    map_err!(commit_generated(sh, &files), CommitGeneratedFailed)
}

pub fn commit_generated(sh: &Shell, files: &[&str]) -> Result<(), CommitGeneratedError> {
    use CommitGeneratedError::*;
    let generated = handle!(cmd!(sh, "git hash-object -- {files...}").read(), GitHashObjectFailed);
    if let Err(error) = cmd!(sh, "git commit --only --message 'chore: configure CI age key' -- {files...}").run() {
        let current = handle!(cmd!(sh, "git hash-object -- {files...}").read(), GitHashObjectAfterCommitFailed, commit_error: error);
        if current == generated {
            handle!(cmd!(sh, "git restore --worktree -- {files...}").run(), GitRestoreFailed, commit_error: error);
        }
        return Err(GitCommitFailed {
            source: error,
        });
    }
    Ok(())
}

#[derive(Error, Debug)]
pub enum FnoxGithubInitAgeKeyCommandRunError {
    #[error("failed to create a shell")]
    ShellNewFailed { source: xshell::Error },
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

#[derive(Error, Debug)]
pub enum RequireCleanRepoError {
    #[error("failed to read the Git status")]
    GitStatusFailed { source: xshell::Error },
    #[error("worktree has {count} uncommitted changes; commit or stash them before running this task", count = status.lines().count())]
    WorktreeInvalid { status: String },
    #[error("failed to list tracked files")]
    GitLsFilesFailed { source: xshell::Error },
    #[error("clear assume-unchanged and skip-worktree flags before running this task")]
    IndexFlagsInvalid { tracked: String },
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

#[derive(Error, Debug)]
pub enum ConfigureTestAgeEncryptionError {
    #[error("'{FNOX_CONFIG}' or '{AGENTS_MD}' differs from HEAD")]
    GitDiffFailed { source: xshell::Error },
    #[error("failed to apply the test age encryption")]
    ApplyTestAgeEncryptionFailed { source: Box<ApplyTestAgeEncryptionError> },
    #[error("failed to restore '{FNOX_CONFIG}' after a failed update: {apply_error}")]
    GitRestoreFailed { source: xshell::Error, apply_error: Box<ApplyTestAgeEncryptionError> },
}

#[derive(Error, Debug)]
pub enum ApplyTestAgeEncryptionError {
    #[error("failed to add the CI age key to the test age recipients")]
    UpdateRecipientsFailed { source: UpdateRecipientsError },
    #[error("failed to re-encrypt the test secrets")]
    ReencryptTestSecretsFailed { source: ReencryptTestSecretsError },
}

#[derive(Error, Debug)]
pub enum UpdateRecipientsError {
    #[error("failed to read '{FNOX_CONFIG}'")]
    ReadToStringFailed { source: io::Error },
    #[error("failed to parse '{FNOX_CONFIG}'")]
    ParseFailed { source: Box<toml_edit::TomlError>, contents: String },
    #[error("failed to write '{FNOX_CONFIG}'")]
    WriteFailed { source: io::Error },
}

#[derive(Error, Debug)]
pub enum ReencryptTestSecretsError {
    #[error("failed to find the test secrets that the CI age key cannot decrypt")]
    PendingKeysFailed { source: PendingKeysError },
    #[error("failed to run fnox reencrypt")]
    FnoxReencryptFailed { source: xshell::Error },
    #[error("failed to verify the re-encrypted test secrets")]
    PendingKeysAfterReencryptFailed { source: PendingKeysError },
    #[error("the CI age key cannot decrypt {keys}; check the test age recipients in '{FNOX_CONFIG}'", keys = unreadable.join(", "))]
    RecipientsInvalid { unreadable: Vec<String> },
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

#[derive(Error, Debug)]
pub enum CommitFnoxConfigIfChangedError {
    #[error("failed to check whether '{FNOX_CONFIG}' changed")]
    GitDiffFnoxConfigFailed { source: xshell::Error },
    #[error("'{AGENTS_MD}' differs from HEAD")]
    GitDiffAgentsMdFailed { source: xshell::Error },
    #[error("failed to regenerate '{AGENTS_MD}'")]
    MiseRunGenAgentsFailed { source: xshell::Error },
    #[error("failed to restore the generated files after a failed regeneration: {gen_agents_error}")]
    GitRestoreFailed { source: xshell::Error, gen_agents_error: xshell::Error },
    #[error("failed to commit the generated files")]
    CommitGeneratedFailed { source: CommitGeneratedError },
}

#[derive(Error, Debug)]
pub enum CommitGeneratedError {
    #[error("failed to hash the generated files")]
    GitHashObjectFailed { source: xshell::Error },
    #[error("failed to commit the generated files")]
    GitCommitFailed { source: xshell::Error },
    #[error("failed to hash the generated files after a failed commit: {commit_error}")]
    GitHashObjectAfterCommitFailed { source: xshell::Error, commit_error: xshell::Error },
    #[error("failed to restore the generated files after a failed commit: {commit_error}")]
    GitRestoreFailed { source: xshell::Error, commit_error: xshell::Error },
}
