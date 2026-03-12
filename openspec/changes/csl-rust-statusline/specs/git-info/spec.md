## ADDED Requirements

### Requirement: Detect git branch name
The system SHALL detect the current git branch name using the `gix` crate by reading the HEAD reference of the repository containing the working directory.

#### Scenario: On a named branch
- **WHEN** the working directory is in a git repo checked out to branch "main"
- **THEN** the system returns "main" as the branch name

#### Scenario: Detached HEAD
- **WHEN** HEAD is detached (not on a branch)
- **THEN** the system returns the short commit hash as the identifier

#### Scenario: Not a git repository
- **WHEN** the working directory is not inside a git repository
- **THEN** the system returns no branch info (no error)

### Requirement: Detect working tree dirty status
The system SHALL detect whether the git working tree has uncommitted changes (modified, added, or deleted files).

#### Scenario: Clean working tree
- **WHEN** there are no uncommitted changes
- **THEN** the system reports the tree as clean and the branch name renders in default color

#### Scenario: Dirty working tree
- **WHEN** there are modified, staged, or untracked files
- **THEN** the system reports the tree as dirty and the branch name renders in yellow

### Requirement: Handle gix failures gracefully
The system SHALL catch any errors from the `gix` crate and degrade gracefully by omitting git information from the statusline.

#### Scenario: gix initialization fails
- **WHEN** `gix` fails to open or discover a repository (e.g., permissions error)
- **THEN** the system shows the directory without branch info and does not crash
