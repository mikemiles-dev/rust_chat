## 1. Named Constants — Client

- [x] 1.1 Add `DEFAULT_PORT: u16 = 8080` constant in `client/src/client.rs` and use it in `parse_server_addr()`
- [x] 1.2 Add `RECONNECT_DELAY: Duration = Duration::from_millis(100)` constant in `client/src/client.rs` and use it in `reconnect()`
- [x] 1.3 Add `FILE_HEADER_OVERHEAD: usize = 1024` constant in `client/src/client.rs` and use it in the file size check
- [x] 1.4 Compile client crate and verify no warnings

## 2. Named Constants — Server

- [x] 2.1 Add `TLS_HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(30)` constant in `server/src/main.rs` and use it in the TLS accept
- [x] 2.2 Add `BROADCAST_BUFFER_MULTIPLIER: usize = 16` and `SERVER_CMD_CHANNEL_SIZE: usize = 100` constants in `server/src/main.rs` and use them in channel creation
- [x] 2.3 Compile server crate and verify no warnings

## 3. Named Constants — Shared Protocol

- [x] 3.1 Add `HEADER_SIZE: usize = 5` constant in `shared/src/message.rs` and use it in `try_new()` and `From<Vec<u8>>`
- [x] 3.2 Add associated byte constants on `MessageTypes` (e.g., `const CHAT_MESSAGE_BYTE: u8 = 1` through `const VERSION_MISMATCH_BYTE: u8 = 17`)
- [x] 3.3 Update `From<u8> for MessageTypes` to use the named byte constants
- [x] 3.4 Update `From<ChatMessage> for Vec<u8>` to use the named byte constants
- [x] 3.5 Compile shared crate and run existing tests

## 4. Safe Integer Casting — File Size

- [x] 4.1 Replace `metadata.len() as usize` with `usize::try_from(metadata.len())` in `client/src/client.rs`, returning an error on overflow
- [x] 4.2 Replace `u64::from_be_bytes(...) as usize` with `usize::try_from(...)` in `client/src/file_transfer.rs`, logging an error and returning on overflow
- [x] 4.3 Replace `unwrap_or([0u8; 8])` with `.map_err(...)` that logs and returns in `client/src/file_transfer.rs`
- [x] 4.4 Compile client crate and verify no warnings

## 5. Safe Integer Casting — Length Prefixed Strings

- [x] 5.1 Add validation that string length fits in `u8` before `as u8` casts in `client/src/client.rs` (file send message building)
- [x] 5.2 Add validation that string length fits in `u8` before `as u8` casts in `client/src/file_transfer.rs` (accept/reject response building)
- [x] 5.3 Add validation that string length fits in `u8` before `as u8` casts in `server/src/user_connection/file_transfer_handlers.rs` (outgoing message building)
- [x] 5.4 Compile all crates and verify no warnings

## 6. Error Handling — Client File Transfer Helper

- [x] 6.1 Create an `extract_or_return!` macro in `client/src/file_transfer.rs` that calls `extract_length_prefixed_string`, logs the error context, and returns from the enclosing function on failure
- [x] 6.2 Replace all `match extract_length_prefixed_string { Ok => ..., Err => { log; return } }` blocks in `handle_file_transfer` with the macro
- [x] 6.3 Replace the same pattern in `handle_file_transfer_request` with the macro
- [x] 6.4 Replace the same pattern in `handle_file_transfer_response` with the macro
- [x] 6.5 Compile client crate and verify no warnings

## 7. Error Handling — Server File Transfer Helper

- [x] 7.1 Add a private `parse_field(content, offset, addr) -> Result<(&str, usize), UserConnectionError>` function in `server/src/user_connection/file_transfer_handlers.rs`
- [x] 7.2 Replace `.map_err` blocks in `process_file_transfer` with calls to `parse_field`
- [x] 7.3 Replace `.map_err` blocks in `process_file_transfer_request` with calls to `parse_field`
- [x] 7.4 Replace `.map_err` blocks in `process_file_transfer_response` with calls to `parse_field`
- [x] 7.5 Compile server crate and verify no warnings

## 8. Error Handling — Require Username Helper

- [x] 8.1 Add `require_username(&self, username: &Option<String>, action: &str) -> Result<String, UserConnectionError>` method on `MessageHandlers` in `server/src/user_connection/handlers.rs`
- [x] 8.2 Replace the `match username` block in `process_chat_message` with `self.require_username()`
- [x] 8.3 Replace the `match username` block in `process_direct_message` with `self.require_username()`
- [x] 8.4 Replace the `match username` block in `process_set_status` with `self.require_username()`
- [x] 8.5 Replace the `match username` block in `process_rename_request` with `self.require_username()`
- [x] 8.6 Replace the `match username` blocks in `process_file_transfer`, `process_file_transfer_request`, and `process_file_transfer_response` with `self.require_username()`
- [x] 8.7 Compile server crate and verify no warnings

## 9. Error Handling — ChatClientError Display

- [x] 9.1 Implement `std::fmt::Display` for `ChatClientError` in `client/src/client.rs` with human-readable messages for each variant
- [x] 9.2 Compile client crate and verify no warnings

## 10. Handler Module Split

- [x] 10.1 Create `server/src/user_connection/user_handlers.rs` and move `process_join`, `process_rename_request`, `process_version_check` from `handlers.rs`
- [x] 10.2 Create `server/src/user_connection/message_handlers.rs` and move `process_chat_message`, `process_direct_message`, `process_list_users`, `process_set_status` from `handlers.rs`
- [x] 10.3 Add `mod user_handlers;` and `mod message_handlers;` declarations (in `handlers.rs` or `mod.rs` as appropriate)
- [x] 10.4 Verify `handlers.rs` retains `MessageHandlers` struct, `StreamWrapper`, constants, `randomize_username`, `process_message` router, and tests
- [x] 10.5 Compile server crate and run `cargo test` — all tests pass

## 11. Shared Completer

- [x] 11.1 Add `rustyline` dependency to `shared/Cargo.toml`
- [x] 11.2 Create `shared/src/completer.rs` with `CommandCompleter` struct that accepts a command list and optional `Arc<RwLock<HashSet<String>>>` for user completion
- [x] 11.3 Implement `Completer`, `Hinter`, `Highlighter`, `Validator`, and `Helper` traits on `CommandCompleter`
- [x] 11.4 Add `pub mod completer;` to `shared/src/lib.rs`
- [x] 11.5 Compile shared crate

## 12. Shared Readline Spawner

- [x] 12.1 Create `shared/src/readline.rs` with `spawn_readline_handler` function accepting a `CommandCompleter` and `require_tty: bool` flag
- [x] 12.2 Implement TTY-optional logic: return `None` when no TTY and `require_tty=false`, panic when no TTY and `require_tty=true`
- [x] 12.3 Add `pub mod readline;` to `shared/src/lib.rs`
- [x] 12.4 Compile shared crate

## 13. Client Adoption of Shared Readline

- [x] 13.1 Update `client/src/main.rs` to import `shared::completer::CommandCompleter` and `shared::readline::spawn_readline_handler`
- [x] 13.2 Construct `CommandCompleter` with client commands and connected users list
- [x] 13.3 Replace the call to the local `readline_helper::spawn_readline_handler` with the shared version
- [x] 13.4 Delete `client/src/completer.rs` and `client/src/readline_helper.rs`
- [x] 13.5 Remove `mod completer;` and `mod readline_helper;` from `client/src/main.rs`
- [x] 13.6 Compile client crate and verify no warnings

## 14. Server Adoption of Shared Readline

- [x] 14.1 Update `server/src/main.rs` to import `shared::completer::CommandCompleter` and `shared::readline::spawn_readline_handler`
- [x] 14.2 Construct `CommandCompleter` with server commands and no user list
- [x] 14.3 Replace the call to the local `readline_helper::spawn_readline_handler` with the shared version
- [x] 14.4 Delete `server/src/completer.rs` and `server/src/readline_helper.rs`
- [x] 14.5 Remove `mod completer;` and `mod readline_helper;` from `server/src/main.rs`
- [x] 14.6 Compile server crate and verify no warnings

## 15. Final Verification

- [x] 15.1 Run `cargo build` for the entire workspace — zero errors, zero warnings
- [x] 15.2 Run `cargo test` for the entire workspace — all tests pass
- [x] 15.3 Verify no `as u8` length casts remain without prior validation
- [x] 15.4 Verify no `as usize` casts from `u64` remain without `TryFrom`
- [x] 15.5 Verify `client/src/completer.rs`, `client/src/readline_helper.rs`, `server/src/completer.rs`, `server/src/readline_helper.rs` are deleted
- [x] 15.6 Verify `handlers.rs` is under 250 lines
