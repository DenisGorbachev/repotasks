use errgonomic::{handle, map_err};
use secrecy::{ExposeSecret, SecretString};
use thiserror::Error;
use url::Url;
use xshell::{Shell, cmd};

#[derive(Clone, Debug)]
pub struct GithubRepo(pub Url);

impl GithubRepo {
    pub fn origin(sh: &Shell) -> Result<Self, GithubRepoOriginError> {
        use GithubRepoOriginError::*;
        let remote = handle!(cmd!(sh, "git config --get remote.origin.url").read(), GitConfigFailed);
        let url = handle!(cmd!(sh, "gh repo view {remote} --json url --jq .url").read(), GhRepoViewFailed, remote);
        let url = handle!(Url::parse(&url), ParseFailed, url);
        Ok(url.into())
    }

    pub fn secret_names(&self, sh: &Shell) -> Result<Vec<String>, GithubRepoSecretNamesError> {
        use GithubRepoSecretNamesError::*;
        let url = self.0.as_str();
        let names = handle!(cmd!(sh, "gh secret list --repo {url} --json name --jq .[].name").read(), GhSecretListFailed);
        Ok(names.lines().map(String::from).collect())
    }

    pub fn set_secret(&self, sh: &Shell, name: &str, value: &SecretString) -> Result<(), GithubRepoSetSecretError> {
        use GithubRepoSetSecretError::*;
        let url = self.0.as_str();
        map_err!(
            cmd!(sh, "gh secret set {name} --repo {url}")
                .stdin(value.expose_secret())
                .run(),
            GhSecretSetFailed,
            name
        )
    }
}

impl From<Url> for GithubRepo {
    fn from(url: Url) -> Self {
        Self(url)
    }
}

#[derive(Error, Debug)]
pub enum GithubRepoOriginError {
    #[error("failed to read the origin remote URL")]
    GitConfigFailed { source: xshell::Error },
    #[error("failed to resolve the GitHub repository of remote '{remote}'")]
    GhRepoViewFailed { source: xshell::Error, remote: String },
    #[error("failed to parse the GitHub repository URL '{url}'")]
    ParseFailed { source: url::ParseError, url: String },
}

#[derive(Error, Debug)]
pub enum GithubRepoSecretNamesError {
    #[error("failed to list GitHub secrets")]
    GhSecretListFailed { source: xshell::Error },
}

#[derive(Error, Debug)]
pub enum GithubRepoSetSecretError {
    #[error("failed to set GitHub secret '{name}'")]
    GhSecretSetFailed { source: xshell::Error, name: String },
}
