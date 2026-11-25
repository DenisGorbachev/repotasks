use crate::{AGENTS_MD, FNOX_CONFIG};
use errgonomic::{handle, handle_bool};
use thiserror::Error;
use xshell::{Shell, cmd};

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
