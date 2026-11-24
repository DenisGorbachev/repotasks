use crate::{FnoxGithubCommand, FnoxGithubCommandRunError};
use FnoxSubcommand::*;
use clap::{Parser, Subcommand};
use errgonomic::map_err;
use std::process::ExitCode;
use thiserror::Error;

#[derive(Parser, Clone, Debug)]
#[command(flatten_help = true)]
pub struct FnoxCommand {
    #[command(subcommand)]
    subcommand: FnoxSubcommand,
}

#[derive(Subcommand, Clone, Debug)]
pub enum FnoxSubcommand {
    Github(FnoxGithubCommand),
}

impl FnoxCommand {
    pub async fn run(self) -> Result<ExitCode, FnoxCommandRunError> {
        use FnoxCommandRunError::*;
        let Self {
            subcommand,
        } = self;
        match subcommand {
            Github(command) => map_err!(command.run().await, FnoxGithubCommandRunFailed),
        }
    }
}

#[derive(Error, Debug)]
pub enum FnoxCommandRunError {
    #[error("failed to run fnox github command")]
    FnoxGithubCommandRunFailed { source: FnoxGithubCommandRunError },
}
