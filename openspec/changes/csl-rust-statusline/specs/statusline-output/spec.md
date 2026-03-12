## ADDED Requirements

### Requirement: Parse stdin JSON input
The system SHALL read a single JSON object from stdin containing the fields: `model` (string), `contextWindow` (integer), `tokensUsed` (integer), `workingDirectory` (string), and `sessionStartTime` (ISO 8601 string). Unknown fields SHALL be ignored.

#### Scenario: Valid JSON input
- **WHEN** Claude Code pipes valid JSON with all expected fields to stdin
- **THEN** the system parses all fields and uses them for statusline rendering

#### Scenario: Missing optional fields
- **WHEN** JSON input is missing `sessionStartTime` or `workingDirectory`
- **THEN** the system uses sensible defaults (empty string for directory, omits duration)

#### Scenario: Malformed JSON input
- **WHEN** stdin contains invalid JSON or is empty
- **THEN** the system prints an empty string to stdout and exits with code 0

### Requirement: Render session info line
The system SHALL output a single line containing: model name, context usage progress bar, shortened working directory with git branch, and session duration.

#### Scenario: Full session line with git
- **WHEN** all data is available including git branch "main" and working directory "~/projects/myapp"
- **THEN** the output line matches the format: `<model>  <bar> <pct>%  <dir> (<branch>)  <duration>`

#### Scenario: Session line without git
- **WHEN** the working directory is not a git repository
- **THEN** the output line omits the branch portion and shows only the directory

### Requirement: Render context usage progress bar
The system SHALL render context usage as a 10-segment block bar using `▰` (filled) and `▱` (empty) characters, with a percentage.

#### Scenario: Context at 48%
- **WHEN** tokensUsed is 48% of contextWindow
- **THEN** the bar renders as `▰▰▰▰▰▱▱▱▱▱ 48%`

#### Scenario: Context at 0%
- **WHEN** tokensUsed is 0
- **THEN** the bar renders as `▱▱▱▱▱▱▱▱▱▱ 0%`

### Requirement: Color-code progress bars by threshold
The system SHALL color progress bars based on percentage: green (<50%), yellow (50-70%), orange (70-90%), red (>=90%).

#### Scenario: Low usage coloring
- **WHEN** usage percentage is 35%
- **THEN** the progress bar is rendered in green

#### Scenario: High usage coloring
- **WHEN** usage percentage is 92%
- **THEN** the progress bar is rendered in red

### Requirement: Render rate limit lines
The system SHALL render one line per active rate limit window showing: window label, usage bar, percentage, and reset time.

#### Scenario: Two rate limit windows
- **WHEN** usage data contains a 5-hour and 7-day window
- **THEN** two lines are rendered below the session line, each with window label, bar, percentage, and reset info

#### Scenario: No usage data available
- **WHEN** no OAuth token is found or API fetch fails
- **THEN** no rate limit lines are rendered (only the session line is shown)

### Requirement: Format session duration
The system SHALL format session duration as compact human-readable strings (e.g., `2h15m`, `45m`, `3m`).

#### Scenario: Multi-hour session
- **WHEN** session has been running for 2 hours and 15 minutes
- **THEN** duration displays as `2h15m`

#### Scenario: Short session
- **WHEN** session has been running for 3 minutes
- **THEN** duration displays as `3m`

### Requirement: Shorten working directory
The system SHALL replace the user's home directory prefix with `~` in the displayed path.

#### Scenario: Home directory path
- **WHEN** working directory is `/Users/alice/projects/myapp`
- **THEN** it displays as `~/projects/myapp`

### Requirement: Always exit 0 in statusline mode
The system SHALL always exit with code 0 when running in statusline mode, regardless of any errors encountered.

#### Scenario: Any error condition
- **WHEN** any error occurs during statusline rendering (parse failure, API error, git error)
- **THEN** the process exits with code 0
