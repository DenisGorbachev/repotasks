use crate::{RunInTerminalError, run_in_terminal};
use errgonomic::handle;
use thiserror::Error;
use xshell::{Shell, cmd};

pub fn commit_generated(sh: &Shell, files: &[&str]) -> Result<(), CommitGeneratedError> {
    use CommitGeneratedError::*;
    let generated = handle!(cmd!(sh, "git hash-object -- {files...}").read(), GitHashObjectFailed);
    if let Err(error) = run_in_terminal(cmd!(sh, "git commit --only --message 'chore: configure CI age key' -- {files...}")) {
        let current = handle!(cmd!(sh, "git hash-object -- {files...}").read(), GitHashObjectAfterCommitFailed, commit_error: error);
        if current == generated {
            handle!(run_in_terminal(cmd!(sh, "git restore --worktree -- {files...}")), GitRestoreFailed, commit_error: error);
        }
        return Err(GitCommitFailed {
            source: error,
        });
    }
    Ok(())
}

#[derive(Error, Debug)]
pub enum CommitGeneratedError {
    #[error("failed to hash the generated files")]
    GitHashObjectFailed { source: xshell::Error },
    #[error("failed to commit the generated files")]
    GitCommitFailed { source: RunInTerminalError },
    #[error("failed to hash the generated files after a failed commit: {commit_error}")]
    GitHashObjectAfterCommitFailed { source: xshell::Error, commit_error: RunInTerminalError },
    #[error("failed to restore the generated files after a failed commit: {commit_error}")]
    GitRestoreFailed { source: RunInTerminalError, commit_error: RunInTerminalError },
}
