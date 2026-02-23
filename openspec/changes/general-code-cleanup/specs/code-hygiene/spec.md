## ADDED Requirements

### Requirement: No suppressed dead code warnings
The codebase SHALL NOT contain `#[allow(dead_code)]` annotations to suppress warnings on genuinely unused items. Unused struct fields, enum variants, and function parameters SHALL be removed rather than annotated. The `#[allow(async_fn_in_trait)]` annotation is exempt as it suppresses a different lint category.

#### Scenario: Unused enum variant removed
- **WHEN** an enum variant (e.g., `UserInputError::InvalidUser`) is not referenced anywhere in the codebase
- **THEN** the variant SHALL be deleted entirely rather than annotated with `#[allow(dead_code)]`

#### Scenario: Unused struct fields removed
- **WHEN** a struct field (e.g., `PendingOutgoingTransfer::file_size`) is annotated with `#[allow(dead_code)]` and is not read or written outside of construction
- **THEN** the field and its `#[allow(dead_code)]` annotation SHALL be removed

#### Scenario: Async trait annotation preserved
- **WHEN** a trait uses async methods and has `#[allow(async_fn_in_trait)]`
- **THEN** the annotation SHALL be kept because it suppresses a different, valid lint

### Requirement: Consistent user identifier naming
The server crate SHALL use `username` as the canonical variable and field name for user identifiers. All occurrences of `chat_name` in `user_connection/mod.rs` and `user_connection/handlers.rs` SHALL be renamed to `username`.

#### Scenario: Field renamed in UserConnection
- **WHEN** the `UserConnection` struct has a field tracking the connected user's name
- **THEN** that field SHALL be named `username`, not `chat_name`

#### Scenario: Handler parameters renamed
- **WHEN** handler methods in `MessageHandlers` accept a parameter for the current user's name
- **THEN** that parameter SHALL be named `username`, not `chat_name`

#### Scenario: Behavior unchanged after rename
- **WHEN** `chat_name` is renamed to `username` throughout the server crate
- **THEN** all runtime behavior, wire protocol messages, and log output SHALL remain identical

### Requirement: Lock acquisition safety
Lock acquisitions on `RwLock` and `Mutex` SHALL use `.expect()` with a descriptive message instead of bare `.unwrap()`. This applies to both `std::sync` and `tokio::sync` lock types.

#### Scenario: Std RwLock with expect
- **WHEN** a `std::sync::RwLock` is acquired (e.g., `self.users.read()`)
- **THEN** the call SHALL use `.expect("descriptive message")` instead of `.unwrap()`

#### Scenario: Descriptive panic message
- **WHEN** a lock acquisition uses `.expect()`
- **THEN** the message SHALL identify which lock failed (e.g., `"connected users lock poisoned"`)

### Requirement: Scoped blocks for lock guard lifetime
Explicit `drop()` calls on lock guards SHALL be replaced with scoped blocks `{ ... }` where the guard's lifetime is the sole reason for the `drop()`. Explicit `drop()` SHALL be retained where scoping would increase nesting beyond 4 levels or where the variable is referenced after the drop point.

#### Scenario: Simple drop replaced with scope
- **WHEN** a lock guard is acquired, used, and then explicitly dropped before subsequent code
- **THEN** the guard and its usage SHALL be wrapped in a `{ ... }` block and the `drop()` call removed

#### Scenario: Complex nesting keeps drop
- **WHEN** replacing `drop()` with a scoped block would create nesting deeper than 4 levels
- **THEN** the explicit `drop()` SHALL be retained

### Requirement: File transfer module extraction
File transfer logic SHALL be extracted into dedicated modules to reduce file size. `client/src/client.rs` SHALL have file transfer methods moved to `client/src/file_transfer.rs`. `server/src/user_connection/handlers.rs` SHALL have file transfer handlers moved to `server/src/user_connection/file_transfer_handlers.rs`.

#### Scenario: Client file transfer extraction
- **WHEN** the client crate is organized
- **THEN** `handle_file_transfer`, `handle_file_transfer_request`, `handle_file_transfer_response`, `accept_file_transfer`, and `reject_file_transfer` SHALL reside in `client/src/file_transfer.rs`

#### Scenario: Server file transfer extraction
- **WHEN** the server crate is organized
- **THEN** `process_file_transfer`, `process_file_transfer_request`, and `process_file_transfer_response` SHALL reside in `server/src/user_connection/file_transfer_handlers.rs`

#### Scenario: No behavior change from extraction
- **WHEN** methods are moved to new modules
- **THEN** all functionality, error handling, and message flow SHALL remain identical

### Requirement: Unused function parameters removed
Function parameters that are accepted but never used (excluding trait-required parameters) SHALL be removed from the function signature and all call sites.

#### Scenario: Unused prompt parameter
- **WHEN** a function accepts a parameter (e.g., `_prompt: String`) that is never read
- **THEN** the parameter SHALL be removed from the function signature and all call sites updated
