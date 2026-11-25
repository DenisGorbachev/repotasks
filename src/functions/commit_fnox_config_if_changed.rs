use crate::{AGENTS_MD, CommitGeneratedError, FNOX_CONFIG, RunInTerminalError, commit_generated, run_in_terminal};
use errgonomic::{handle, map_err};
use thiserror::Error;
use xshell::{Shell, cmd};

pub fn commit_fnox_config_if_changed(sh: &Shell) -> Result<(), CommitFnoxConfigIfChangedError> {
    use CommitFnoxConfigIfChangedError::*;
    let changed = handle!(cmd!(sh, "git diff --name-only -- {FNOX_CONFIG}").read(), GitDiffFnoxConfigFailed);
    if changed.is_empty() {
        return Ok(());
    }
    let files = [FNOX_CONFIG, AGENTS_MD];
    if let Err(error) = run_in_terminal(cmd!(sh, "mise run gen:agents")) {
        handle!(run_in_terminal(cmd!(sh, "git restore --worktree -- {files...}")), GitRestoreFailed, gen_agents_error: error);
        return Err(MiseRunGenAgentsFailed {
            source: error,
        });
    }
    map_err!(commit_generated(sh, &files), CommitGeneratedFailed)
}

#[derive(Error, Debug)]
pub enum CommitFnoxConfigIfChangedError {
    #[error("failed to check whether '{FNOX_CONFIG}' changed")]
    GitDiffFnoxConfigFailed { source: xshell::Error },
    #[error("failed to regenerate '{AGENTS_MD}'")]
    MiseRunGenAgentsFailed { source: RunInTerminalError },
    #[error("failed to restore the generated files after a failed regeneration: {gen_agents_error}")]
    GitRestoreFailed { source: RunInTerminalError, gen_agents_error: RunInTerminalError },
    #[error("failed to commit the generated files")]
    CommitGeneratedFailed { source: CommitGeneratedError },
}
