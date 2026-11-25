# repotasks

## `fnox github init-age-key`

- Must resolve GitHub repo from stored origin URL through `gh repo view`.
- May treat write races as acceptable because command is run by a single user who guarantees that other scripts do not run concurrently.
