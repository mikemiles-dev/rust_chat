## ADDED Requirements

### Requirement: Client file transfer parsing uses unified helper
The client file transfer module SHALL use a single helper function or macro to extract length-prefixed strings with consistent error logging, replacing the repeated `match extract_length_prefixed_string { Ok => ..., Err => { log; return; } }` pattern in `client/src/file_transfer.rs`.

#### Scenario: Successful extraction via helper
- **WHEN** binary content contains a valid length-prefixed string at the given offset
- **THEN** the helper returns the string and new offset

#### Scenario: Failed extraction via helper
- **WHEN** binary content has insufficient data or invalid UTF-8
- **THEN** the helper logs the provided context message and causes the calling function to return early

### Requirement: Server file transfer parsing uses unified helper
The server file transfer module SHALL use a single helper function or closure to extract length-prefixed strings with consistent error logging, replacing the repeated `.map_err` blocks in `server/src/user_connection/file_transfer_handlers.rs`.

#### Scenario: Successful extraction via helper
- **WHEN** binary content contains a valid length-prefixed string at the given offset
- **THEN** the helper returns the string and new offset

#### Scenario: Failed extraction via helper
- **WHEN** binary content has insufficient data or invalid UTF-8
- **THEN** the helper logs a warning with the client address and returns `UserConnectionError::InvalidMessage`

### Requirement: Server require-username pattern consolidated
The server handler methods that check `username.is_some()` before proceeding SHALL use a shared helper to extract the username, replacing the repeated `match username { Some(name) => name.clone(), None => { log; return Err(...) } }` pattern across `process_file_transfer`, `process_file_transfer_request`, `process_file_transfer_response`, `process_chat_message`, `process_direct_message`, `process_set_status`, and `process_rename_request`.

#### Scenario: User has joined
- **WHEN** the username is `Some`
- **THEN** the helper returns a clone of the username

#### Scenario: User has not joined
- **WHEN** the username is `None`
- **THEN** the helper logs a warning with the action description and client address, and returns `UserConnectionError::InvalidMessage`

### Requirement: ChatClientError implements Display
The `ChatClientError` enum in `client/src/client.rs` SHALL implement `std::fmt::Display` with human-readable messages for each variant.

#### Scenario: Error formatted for display
- **WHEN** a `ChatClientError` is formatted with `{}`
- **THEN** it produces a human-readable description (e.g., "I/O error: connection refused" rather than debug output)
