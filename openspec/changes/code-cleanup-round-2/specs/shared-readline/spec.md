## ADDED Requirements

### Requirement: Generic completer in shared crate
The `shared` crate SHALL provide a `CommandCompleter` struct that implements rustyline's `Completer`, `Hinter`, `Highlighter`, `Validator`, and `Helper` traits. It SHALL accept a list of command strings and an optional `Arc<RwLock<HashSet<String>>>` for user-based completion.

#### Scenario: Command completion without user list
- **WHEN** a `CommandCompleter` is created with commands only (no user list)
- **THEN** it completes `/`-prefixed input against the command list, matching current server completer behavior

#### Scenario: Command completion with user list
- **WHEN** a `CommandCompleter` is created with commands and a user list
- **THEN** it completes `/dm ` and `/send ` prefixes against the user list, and `/`-prefixed input against the command list, matching current client completer behavior

#### Scenario: Hinting behavior
- **WHEN** exactly one candidate matches the current input
- **THEN** the completer provides the remaining suffix as a hint

### Requirement: Shared readline spawner
The `shared` crate SHALL provide a `spawn_readline_handler` function that spawns a blocking thread with a rustyline `Editor`, sends input lines via `tokio::sync::mpsc::UnboundedSender`, and returns the receiver. It SHALL accept a `CommandCompleter` and a boolean flag indicating whether to require a TTY.

#### Scenario: TTY available
- **WHEN** the readline handler is spawned and a TTY is available
- **THEN** it returns `Some(receiver)` and sends each input line through the channel

#### Scenario: TTY unavailable with require_tty=false
- **WHEN** the readline handler is spawned with `require_tty=false` and no TTY is available
- **THEN** it returns `None` without panicking (matching current server behavior)

#### Scenario: TTY unavailable with require_tty=true
- **WHEN** the readline handler is spawned with `require_tty=true` and no TTY is available
- **THEN** it panics with a descriptive message (matching current client behavior)

### Requirement: Client uses shared completer
The client SHALL replace `client/src/completer.rs` with an import of `shared::completer::CommandCompleter`, constructing it with client commands and the connected users list.

#### Scenario: Client tab completion works
- **WHEN** the user presses tab after typing `/dm ` followed by a username prefix
- **THEN** the completer suggests matching usernames, identical to current behavior

### Requirement: Server uses shared completer
The server SHALL replace `server/src/completer.rs` with an import of `shared::completer::CommandCompleter`, constructing it with server commands and no user list.

#### Scenario: Server tab completion works
- **WHEN** the server operator presses tab after typing `/`
- **THEN** the completer suggests matching server commands, identical to current behavior

### Requirement: Client uses shared readline spawner
The client SHALL replace `client/src/readline_helper.rs` with a call to `shared::readline::spawn_readline_handler`, passing the constructed `CommandCompleter` and `require_tty=true`.

#### Scenario: Client readline works
- **WHEN** the client starts
- **THEN** it reads input lines via the shared readline handler with auto-history enabled

### Requirement: Server uses shared readline spawner
The server SHALL replace `server/src/readline_helper.rs` with a call to `shared::readline::spawn_readline_handler`, passing the constructed `CommandCompleter` and `require_tty=false`.

#### Scenario: Server readline works with TTY
- **WHEN** the server starts with a TTY available
- **THEN** it reads input lines via the shared readline handler

#### Scenario: Server readline works without TTY
- **WHEN** the server starts without a TTY (Docker/systemd)
- **THEN** it receives `None` and runs in non-interactive mode
