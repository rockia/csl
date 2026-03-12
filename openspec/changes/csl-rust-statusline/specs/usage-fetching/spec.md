## ADDED Requirements

### Requirement: Resolve OAuth token from priority chain
The system SHALL resolve an OAuth token by checking in order: (1) `CLAUDE_OAUTH_TOKEN` environment variable, (2) OS keychain via `keyring` crate, (3) `~/.claude/.credentials.json` file. The first successful source wins.

#### Scenario: Token from environment variable
- **WHEN** `CLAUDE_OAUTH_TOKEN` is set
- **THEN** the system uses that token and does not check keychain or file

#### Scenario: Token from keychain
- **WHEN** env var is not set but keychain contains a valid token
- **THEN** the system uses the keychain token

#### Scenario: Token from credentials file
- **WHEN** env var is not set and keychain fails
- **THEN** the system reads the token from `~/.claude/.credentials.json`

#### Scenario: No token available
- **WHEN** all three sources fail or are unavailable
- **THEN** the system proceeds without usage data (no error, no crash)

### Requirement: Fetch usage data from API
The system SHALL fetch usage data from `https://api.anthropic.com/api/oauth/usage` using the resolved OAuth token with a 3-second HTTP timeout.

#### Scenario: Successful API fetch
- **WHEN** the API returns a valid JSON response within 3 seconds
- **THEN** the system parses rate limit windows and usage percentages from the response

#### Scenario: API timeout
- **WHEN** the API does not respond within 3 seconds
- **THEN** the system falls back to cached data or skips usage display

#### Scenario: API error response
- **WHEN** the API returns a non-200 status code
- **THEN** the system falls back to cached data or skips usage display

### Requirement: Cache usage data with 60-second TTL
The system SHALL cache API responses at `/tmp/csl/usage-cache.json` with a 60-second time-to-live.

#### Scenario: Fresh cache available
- **WHEN** a cache file exists and is less than 60 seconds old
- **THEN** the system uses cached data without making an API call

#### Scenario: Stale cache with API available
- **WHEN** cache is older than 60 seconds and API fetch succeeds
- **THEN** the system uses fresh API data and updates the cache

#### Scenario: Stale cache with API unavailable
- **WHEN** cache is older than 60 seconds and API fetch fails
- **THEN** the system uses the stale cached data

#### Scenario: Corrupted cache file
- **WHEN** the cache file exists but contains invalid JSON
- **THEN** the system deletes the corrupt file and attempts a fresh API fetch

#### Scenario: No cache and no API
- **WHEN** no cache file exists and the API fetch fails
- **THEN** the system skips the usage display entirely
