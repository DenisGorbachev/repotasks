use crate::OpItem;
use core::convert::Infallible;
use core::str::FromStr;
use errgonomic::{handle, handle_bool, map_err, render_command};
use secrecy::{ExposeSecret, SecretSlice, SecretString};
use serde_json::json;
use std::io;
use std::process::{Command, ExitStatus, Output, Stdio};
use thiserror::Error;
use xshell::{Shell, cmd};

#[derive(Clone, Debug)]
pub struct OpVault(pub String);

impl OpVault {
    pub fn find_item(&self, sh: &Shell, title: &str) -> Result<Option<OpItem>, OpVaultFindItemError> {
        use OpVaultFindItemError::*;
        let vault = &self.0;
        let mut command = Command::from(cmd!(sh, "op item get {title} --vault {vault} --format json --reveal"));
        command.stdin(Stdio::inherit());
        let Output {
            status,
            stdout,
            stderr,
        } = handle!(command.output(), OutputFailed, title, command);
        let stdout = SecretSlice::from(stdout);
        if !status.success() {
            let reason = String::from_utf8_lossy(&stderr).trim().to_owned();
            let missing = format!("{} isn't an item in the {} vault.", json!(title), json!(vault));
            handle_bool!(status.code() != Some(1) || !reason.contains(&missing), OpItemGetInvalid, title, vault, status, reason);
            return Ok(None);
        }
        let item: OpItem = handle!(serde_json::from_slice(stdout.expose_secret()), FromSliceFailed, title);
        handle_bool!(item.title != title, TitleInvalid, title, vault, returned: item.title);
        Ok(Some(item))
    }

    pub fn create_password_item(&self, sh: &Shell, title: &str, password: &SecretString) -> Result<(), OpVaultCreatePasswordItemError> {
        use OpVaultCreatePasswordItemError::*;
        let vault = &self.0;
        let template = SecretString::from(json!({"title": title, "category": "PASSWORD", "fields": [{"id": "password", "type": "CONCEALED", "purpose": "PASSWORD", "label": "password", "value": password.expose_secret()}]}).to_string());
        map_err!(
            cmd!(sh, "op item create - --vault {vault}")
                .stdin(template.expose_secret())
                .ignore_stdout()
                .run(),
            OpItemCreateFailed,
            title
        )
    }
}

impl From<String> for OpVault {
    fn from(name: String) -> Self {
        Self(name)
    }
}

impl FromStr for OpVault {
    type Err = Infallible;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(name.to_owned()))
    }
}

#[derive(Error, Debug)]
pub enum OpVaultFindItemError {
    #[error("failed to run `{}` to look up 1Password item '{title}'", render_command(command))]
    OutputFailed { source: io::Error, title: String, command: Box<Command> },
    #[error("1Password lookup for '{title}' in '{vault}' exited with {status}: {reason}")]
    OpItemGetInvalid { title: String, vault: String, status: ExitStatus, reason: String },
    #[error("failed to parse 1Password item '{title}'")]
    FromSliceFailed { source: serde_json::Error, title: String },
    #[error("1Password returned item '{returned}' instead of '{title}' in '{vault}'")]
    TitleInvalid { title: String, vault: String, returned: String },
}

#[derive(Error, Debug)]
pub enum OpVaultCreatePasswordItemError {
    #[error("failed to create 1Password item '{title}'")]
    OpItemCreateFailed { source: xshell::Error, title: String },
}
