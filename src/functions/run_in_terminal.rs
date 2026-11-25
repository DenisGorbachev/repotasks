use errgonomic::{handle, handle_bool, render_command};
use std::io;
use std::process::{Command, ExitStatus};
use thiserror::Error;
use xshell::Cmd;

pub fn run_in_terminal(cmd: Cmd) -> Result<(), RunInTerminalError> {
    use RunInTerminalError::*;
    eprintln!("$ {cmd}");
    let mut command = Command::from(cmd);
    let status = handle!(command.status(), StatusFailed, command);
    handle_bool!(!status.success(), StatusInvalid, command, status);
    Ok(())
}

#[derive(Error, Debug)]
pub enum RunInTerminalError {
    #[error("failed to run `{}`", render_command(command))]
    StatusFailed { source: io::Error, command: Box<Command> },
    #[error("`{}` exited with {status}", render_command(command))]
    StatusInvalid { command: Box<Command>, status: ExitStatus },
}
