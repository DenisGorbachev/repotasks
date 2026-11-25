# repotasks

## struct FnoxGithubInitAgeKeyCommand

- May ignore write races.
  - Rationale: a single user runs it and guarantees that no other scripts run concurrently.

## struct GithubRepo

- Must have methods:
  - `origin`
    - Must resolve the repo by passing the stored `remote.origin.url` to `gh repo view`.
