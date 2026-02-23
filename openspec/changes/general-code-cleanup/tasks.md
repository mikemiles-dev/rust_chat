## 1. Dead Code Removal

- [x] 1.1 Remove `UserInputError::InvalidUser` variant and its `#[allow(dead_code)]` from `shared/src/input.rs`
- [x] 1.2 Verify whether `PendingOutgoingTransfer::file_size` is read anywhere; if unused, remove the field and its `#[allow(dead_code)]` from `client/src/client.rs`
- [x] 1.3 Verify whether `PendingIncomingTransfer::sender` and `PendingIncomingTransfer::file_size` are read anywhere; if unused, remove the fields and their `#[allow(dead_code)]` annotations from `client/src/client.rs`
- [x] 1.4 Remove the unused `_prompt: String` parameter from the function in `client/src/readline_helper.rs` and update all call sites
- [x] 1.5 Compile and verify no new warnings are introduced

## 2. Naming Standardization

- [x] 2.1 Rename the `chat_name` field to `username` in the `UserConnection` struct in `server/src/user_connection/mod.rs`
- [x] 2.2 Rename all `chat_name` variables and parameters to `username` in `server/src/user_connection/mod.rs`
- [x] 2.3 Rename all `chat_name` parameters to `username` in `server/src/user_connection/handlers.rs`
- [x] 2.4 Update any comments referencing `chat_name` to say `username`
- [x] 2.5 Compile and verify all references are updated (compiler will catch misses)

## 3. Binary Parsing Helpers

- [x] 3.1 Add `extract_length_prefixed_string(data: &[u8], offset: usize) -> Result<(&str, usize), &'static str>` to `shared/src/message.rs`
- [x] 3.2 Add `validate_binary_length(data: &[u8], offset: usize, required: usize) -> Result<(), &'static str>` to `shared/src/message.rs`
- [x] 3.3 Add unit tests for both helpers covering success, insufficient data, and invalid UTF-8 cases
- [x] 3.4 Compile shared crate and verify tests pass

## 4. Lock Safety

- [x] 4.1 Replace `.unwrap()` with `.expect("descriptive message")` on the `RwLock` read in `client/src/completer.rs`
- [x] 4.2 Replace `.unwrap()` with `.expect("descriptive message")` on the `RwLock` write in `client/src/client.rs`
- [x] 4.3 Audit for any other bare `.unwrap()` on lock acquisitions across all crates and fix them

## 5. Scoped Blocks

- [x] 5.1 Replace eligible `drop()` calls with scoped blocks in `server/src/main.rs` (7 occurrences)
- [x] 5.2 Replace eligible `drop()` calls with scoped blocks in `server/src/user_connection/mod.rs` (5 occurrences)
- [x] 5.3 Replace eligible `drop()` calls with scoped blocks in `server/src/user_connection/handlers.rs` (18 occurrences) — skip any where nesting would exceed 4 levels
- [x] 5.4 Compile and verify behavior is unchanged

## 6. Server File Transfer Module Extraction

- [x] 6.1 Create `server/src/user_connection/file_transfer_handlers.rs` and move `process_file_transfer`, `process_file_transfer_request`, and `process_file_transfer_response` from `handlers.rs`
- [x] 6.2 Add `mod file_transfer_handlers;` to `server/src/user_connection/mod.rs` (or the appropriate parent module)
- [x] 6.3 Update `handlers.rs` to delegate file transfer message types to the new module
- [x] 6.4 Adjust visibility (`pub(crate)`) on any structs/fields needed by the new module
- [x] 6.5 Replace inline binary parsing in the moved methods with calls to `extract_length_prefixed_string` and `validate_binary_length` from `shared`
- [x] 6.6 Compile server crate and verify no errors

## 7. Client File Transfer Module Extraction

- [x] 7.1 Create `client/src/file_transfer.rs` and move `handle_file_transfer`, `handle_file_transfer_request`, `handle_file_transfer_response`, `accept_file_transfer`, and `reject_file_transfer` from `client.rs`
- [x] 7.2 Add `mod file_transfer;` to `client/src/main.rs` or `client/src/client.rs` as appropriate
- [x] 7.3 Update `client.rs` to delegate file transfer handling to the new module
- [x] 7.4 Adjust visibility (`pub(crate)`) on any structs/fields needed by the new module
- [x] 7.5 Replace inline binary parsing in the moved methods with calls to `extract_length_prefixed_string` and `validate_binary_length` from `shared`
- [x] 7.6 Compile client crate and verify no errors

## 8. Final Verification

- [x] 8.1 Run `cargo build` for the entire workspace — zero errors, zero warnings
- [x] 8.2 Run `cargo test` for the entire workspace — all tests pass
- [x] 8.3 Verify no `#[allow(dead_code)]` annotations remain (except `#[allow(async_fn_in_trait)]`)
- [x] 8.4 Verify no `chat_name` references remain in the server crate
- [x] 8.5 Verify no bare `.unwrap()` on lock acquisitions remain
