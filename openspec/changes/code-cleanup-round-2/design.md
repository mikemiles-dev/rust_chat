## Context

Round 1 of cleanup addressed surface-level issues: dead code, naming inconsistencies, lock safety, and initial module extraction for file transfer handlers. The codebase still has magic numbers scattered across binary parsing and network setup, unchecked integer casts that risk panics or silent truncation, duplicated error handling patterns, a 724-line handler file, and near-identical completer/readline code in both client and server.

The workspace uses Rust 2024 edition with tokio async runtime, rustyline for interactive input (already a workspace dependency), and a custom binary protocol with length-prefixed strings.

## Goals / Non-Goals

**Goals:**
- Replace all magic numbers with named constants for readability and maintainability
- Eliminate unchecked `as` casts that could panic or silently truncate
- Consolidate repeated error handling patterns into helpers
- Split the large `handlers.rs` into focused submodules
- Deduplicate completer and readline logic by moving shared code to the `shared` crate

**Non-Goals:**
- Changing the wire protocol format or adding new message types
- Introducing a logging framework (e.g., `tracing`) to replace the custom logger
- Optimizing tokio feature flags (keep `"full"` for simplicity)
- Adding new tests beyond what's needed to verify refactored code works
- Changing any user-facing behavior

## Decisions

### 1. Constants location strategy

**Decision**: Define constants as close to their usage as possible — module-level `const` items in the file that uses them. Message type byte constants go on the `MessageTypes` enum as associated constants. Protocol header size goes on the `ChatMessage` module.

**Alternative considered**: A centralized `constants.rs` in `shared`. Rejected because most constants are local to a single file (e.g., `DEFAULT_PORT` is only used in `client.rs`, `TLS_HANDSHAKE_TIMEOUT` only in `server/main.rs`). Centralizing would create a grab-bag module with no cohesion.

### 2. Message type byte constants as associated constants

**Decision**: Add `impl MessageTypes { pub const CHAT_MESSAGE_BYTE: u8 = 1; ... }` and use these in both `From<u8>` and `Into<Vec<u8>>` implementations.

**Alternative considered**: A separate `const` block or a function mapping. Associated constants keep the values co-located with the enum definition, are accessible as `MessageTypes::CHAT_MESSAGE_BYTE`, and don't require a separate module.

### 3. Integer cast validation approach

**Decision**: Use `u8::try_from(len).map_err(...)` for length-to-byte casts and `usize::try_from(file_size).map_err(...)` for u64-to-usize casts. Replace `unwrap_or([0u8; 8])` with `.map_err(...)` that returns an error.

**Alternative considered**: Runtime assertions or `.expect()`. Rejected because these panic in release builds. Proper `Result` propagation is more Rust-idiomatic and lets callers handle the error gracefully.

### 4. Client file transfer error helper: macro vs function

**Decision**: Use a macro `extract_or_return!(content, offset, error_msg)` in `client/src/file_transfer.rs` that calls `extract_length_prefixed_string`, logs on failure, and returns from the enclosing function.

**Alternative considered**: A helper function returning `Option<(&str, usize)>`. This doesn't allow early return from the caller — you'd still need `match` or `?` at each call site. A macro eliminates the boilerplate while keeping the early-return behavior the existing code relies on.

### 5. Server file transfer error helper: closure vs function

**Decision**: Use a private helper function `fn parse_field(content, offset, addr) -> Result<(&str, usize), UserConnectionError>` in `file_transfer_handlers.rs` that wraps `extract_length_prefixed_string` with logging and error conversion.

**Alternative considered**: A macro (like the client). On the server side, all handlers already use `Result` return types and `?` propagation, so a function that returns `Result` composes naturally without macros.

### 6. Server require-username helper

**Decision**: Add a `fn require_username(username: &Option<String>, action: &str, addr: SocketAddr) -> Result<String, UserConnectionError>` method on `MessageHandlers` that logs and returns `Err` when `None`.

**Alternative considered**: Checking at the `process_message` level before dispatching. Rejected because `Join` and `VersionCheck` don't require a username, so the check needs to stay per-handler. A shared method reduces the 7 duplicated match blocks to single-line `let username = self.require_username(username, "send chat message", self.addr)?;` calls.

### 7. Handler module split structure

**Decision**: Split into three files under `server/src/user_connection/`:
- `handlers.rs` — `MessageHandlers` struct, shared constants (`MAX_USERNAME_LENGTH`, etc.), `StreamWrapper`, `randomize_username`, `process_message` router, and tests
- `user_handlers.rs` — `process_join`, `process_rename_request`, `process_version_check`
- `message_handlers.rs` — `process_chat_message`, `process_direct_message`, `process_list_users`, `process_set_status`
- `file_transfer_handlers.rs` — already exists, unchanged

All submodules implement methods on `MessageHandlers` via `impl<'a> MessageHandlers<'a> { ... }` blocks, imported into scope by `mod` declarations in `handlers.rs` or `mod.rs`.

**Alternative considered**: Making each handler a standalone function that takes `MessageHandlers` as a parameter. Rejected because the current `impl` pattern works well, and the submodule `impl` blocks are idiomatic Rust for splitting large implementations across files.

### 8. Shared completer and readline

**Decision**: Add two new modules to the `shared` crate:
- `shared/src/completer.rs` — `CommandCompleter` struct parameterized with a command list and an optional user set
- `shared/src/readline.rs` — `spawn_readline_handler` function that takes a `CommandCompleter` and a `require_tty` flag

The `shared` crate already depends on `tokio`. It will need `rustyline` added as a dependency (already a workspace dependency).

Client and server will delete their local `completer.rs` and `readline_helper.rs` files and import from `shared` instead.

**Alternative considered**: A separate `readline` crate in the workspace. Overkill for two small files — `shared` is the natural home for cross-crate utilities.

## Risks / Trade-offs

**[Risk] Macro in client file transfer may be harder to debug** → The macro is small (3 lines expanded) and only used in one file. If it causes confusion, it can be replaced with explicit match blocks later.

**[Risk] Adding rustyline to shared increases its dependency footprint** → Both client and server already depend on rustyline, so no new transitive dependencies are introduced. The shared crate just gains a direct dependency on what it transitively already has.

**[Risk] Handler module split increases file count** → Adding 2 files (`user_handlers.rs`, `message_handlers.rs`) in exchange for reducing `handlers.rs` from 724 lines to ~200 lines (router + constants + tests). Net improvement in navigability.

**[Trade-off] require_username helper adds one function call per handler** → Negligible runtime cost; significant readability improvement by eliminating 7 identical match blocks.
