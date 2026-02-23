## ADDED Requirements

### Requirement: User lifecycle handlers in dedicated module
The `process_join`, `process_rename_request`, and `process_version_check` methods SHALL be moved from `handlers.rs` to a new `server/src/user_connection/user_handlers.rs` module.

#### Scenario: Join processed via user_handlers module
- **WHEN** the server receives a `Join` message
- **THEN** it is routed to `user_handlers::process_join` with identical behavior

#### Scenario: Rename processed via user_handlers module
- **WHEN** the server receives a `RenameRequest` message
- **THEN** it is routed to `user_handlers::process_rename_request` with identical behavior

#### Scenario: Version check processed via user_handlers module
- **WHEN** the server receives a `VersionCheck` message
- **THEN** it is routed to `user_handlers::process_version_check` with identical behavior

### Requirement: Messaging handlers in dedicated module
The `process_chat_message`, `process_direct_message`, `process_list_users`, and `process_set_status` methods SHALL be moved from `handlers.rs` to a new `server/src/user_connection/message_handlers.rs` module.

#### Scenario: Chat message processed via message_handlers module
- **WHEN** the server receives a `ChatMessage` message
- **THEN** it is routed to `message_handlers::process_chat_message` with identical behavior

#### Scenario: Direct message processed via message_handlers module
- **WHEN** the server receives a `DirectMessage` message
- **THEN** it is routed to `message_handlers::process_direct_message` with identical behavior

#### Scenario: List users processed via message_handlers module
- **WHEN** the server receives a `ListUsers` message
- **THEN** it is routed to `message_handlers::process_list_users` with identical behavior

#### Scenario: Set status processed via message_handlers module
- **WHEN** the server receives a `SetStatus` message
- **THEN** it is routed to `message_handlers::process_set_status` with identical behavior

### Requirement: Router remains in handlers.rs
The `process_message` method and the `MessageHandlers` struct definition SHALL remain in `handlers.rs`, which acts as the central router importing from submodules.

#### Scenario: Message routing unchanged
- **WHEN** the server receives any message type
- **THEN** `process_message` in `handlers.rs` dispatches to the correct submodule handler

### Requirement: Shared types and constants remain in handlers.rs
The `StreamWrapper` struct, `MAX_USERNAME_LENGTH`, `MAX_MESSAGE_LENGTH`, `MAX_STATUS_LENGTH` constants, and the `randomize_username` helper SHALL remain in `handlers.rs` so all submodules can access them via `super::`.

#### Scenario: Submodules reference shared types
- **WHEN** `user_handlers.rs` or `message_handlers.rs` needs `StreamWrapper` or validation constants
- **THEN** they import from `super::handlers` without circular dependencies

### Requirement: Existing tests remain and pass
All existing tests in `handlers.rs` SHALL remain in that file (or be moved alongside the code they test) and continue to pass without modification.

#### Scenario: Tests pass after split
- **WHEN** `cargo test` is run on the server crate
- **THEN** all handler tests pass with no failures
