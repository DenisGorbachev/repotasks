use crate::{FnoxGithubInitAgeKeyCommand, FnoxGithubInitAgeKeyCommandRunError};
use FnoxGithubSubcommand::*;
use clap::{Parser, Subcommand};
use errgonomic::map_err;
use std::process::ExitCode;
use thiserror::Error;

#[derive(Parser, Clone, Debug)]
#[command(flatten_help = true)]
pub struct FnoxGithubCommand {
    #[command(subcommand)]
    subcommand: FnoxGithubSubcommand,
}

#[derive(Subcommand, Clone, Debug)]
pub enum FnoxGithubSubcommand {
    InitAgeKey(FnoxGithubInitAgeKeyCommand),
}

impl FnoxGithubCommand {
    pub async fn run(self) -> Result<ExitCode, FnoxGithubCommandRunError> {
        use FnoxGithubCommandRunError::*;
        let Self {
            subcommand,
        } = self;
        match subcommand {
            InitAgeKey(command) => map_err!(command.run().await, FnoxGithubInitAgeKeyCommandRunFailed),
        }
    }
}

#[derive(Error, Debug)]
pub enum FnoxGithubCommandRunError {
    #[error("failed to run fnox github init-age-key command")]
    FnoxGithubInitAgeKeyCommandRunFailed { source: FnoxGithubInitAgeKeyCommandRunError },
}
