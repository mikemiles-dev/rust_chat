## Context

The rust_chat project is a workspace with three crates: `client`, `server`, and `shared`. Over time, certain patterns have drifted — dead code annotations suppress warnings instead of prompting removal, binary parsing is copy-pasted across handler methods, large files have grown without module extraction, and naming conventions vary between `chat_name` and `username` for the same concept.

The codebase currently compiles warning-free only because `#[allow(dead_code)]` masks unused items. The explicit `drop()` pattern for lock guards is functional but obscures intent compared to scoped blocks.

## Goals / Non-Goals

**Goals:**
- Remove all dead code and unnecessary `#[allow(dead_code)]` annotations
- Extract duplicated binary message parsing into reusable helpers in the `shared` crate
- Replace explicit `drop()` calls with scoped blocks where readability improves
- Standardize on `username` as the canonical term for user identifiers across the server crate (renaming `chat_name` field and variable)
- Split `client/src/client.rs` and `server/src/user_connection/handlers.rs` into smaller modules
- Replace bare `.unwrap()` on lock acquisitions with `.expect()` for debuggability

**Non-Goals:**
- Changing any external behavior or wire protocol
- Introducing new dependencies (e.g., `DashMap`)
- Adding documentation or comments beyond what's needed for clarity on new helpers
- Refactoring error types or introducing new error handling patterns
- Performance optimization

## Decisions

### 1. Standardize on `username` over `chat_name`

**Decision**: Rename the `chat_name` field in `UserConnection` and all handler method parameters to `username`.

**Rationale**: The server crate already uses `username` in `main.rs` (kick, ban, rename handlers) and in comments/docs. The `chat_name` usage is isolated to `user_connection/mod.rs` and `handlers.rs`. Standardizing on `username` aligns with the dominant convention and is clearer.

**Alternative considered**: Standardize on `chat_name` — rejected because `username` is more widely used and more intuitive.

### 2. Extract binary parsing helpers into `shared`

**Decision**: Add helper functions to `shared/src/message.rs` (or a new `shared/src/binary_parse.rs` if it grows large enough) for extracting length-prefixed string fields from binary content.

**Rationale**: The file transfer handlers in both client and server independently parse the same binary format: `[1-byte length][string bytes]`. This pattern appears 6+ times. A shared `extract_length_prefixed_string(data: &[u8], offset: usize) -> Result<(&str, usize), Error>` function eliminates the duplication.

**Alternative considered**: Macros — rejected because a plain function is simpler and equally efficient.

### 3. Module extraction strategy

**Decision**:
- **Client**: Extract file transfer logic from `client.rs` into a new `client/src/file_transfer.rs` module. The methods `handle_file_transfer`, `handle_file_transfer_request`, `handle_file_transfer_response`, `accept_file_transfer`, and `reject_file_transfer` move there. They will be implemented as free functions or an extension trait that takes `&mut ChatClient` or the relevant fields.
- **Server**: Extract file transfer handlers from `handlers.rs` into a new `server/src/user_connection/file_transfer_handlers.rs`. The methods `process_file_transfer`, `process_file_transfer_request`, and `process_file_transfer_response` move there as methods on `MessageHandlers`.

**Rationale**: File transfer is the largest self-contained feature in both files. Extracting it reduces each file by ~300-400 lines while keeping the logical grouping clear.

**Alternative considered**: Splitting by concern (user management vs messaging vs file transfer) — viable but more invasive. File transfer alone provides sufficient decomposition for now.

### 4. Scoped blocks vs explicit `drop()`

**Decision**: Replace explicit `drop()` calls with scoped blocks `{ ... }` where the lock guard is the only reason for the `drop()`. Keep explicit `drop()` where the guard is used again after the drop or where scoping would create deeply nested code.

**Rationale**: Scoped blocks make the lock lifetime visually obvious. However, some handler methods acquire and release multiple locks at different points — forcing everything into scopes could increase nesting. Use judgment per call site.

### 5. Dead code removal approach

**Decision**:
- Remove `UserInputError::InvalidUser` variant entirely (unused everywhere)
- Remove `#[allow(dead_code)]` from `PendingOutgoingTransfer::file_size` and `PendingIncomingTransfer::{sender, file_size}` — if the fields are read nowhere, remove them; if they're used only in debug output, keep them without the annotation
- Keep `#[allow(async_fn_in_trait)]` annotations (these suppress a valid Rust lint for async trait methods without `Send` bounds)

## Risks / Trade-offs

- **Renaming `chat_name` → `username`**: Touches many lines in `mod.rs` and `handlers.rs`. Risk of missed references. → Mitigation: Use find-and-replace with compilation verification; the compiler will catch any misses.
- **Module extraction may break `pub` visibility**: Moving methods to a new file may require adjusting field visibility on structs. → Mitigation: Use `pub(crate)` where needed, keep the public API surface unchanged.
- **Scoped blocks can increase nesting**: Some handler methods already have 3-4 levels of nesting. → Mitigation: Only apply scoped blocks where they reduce complexity, not increase it. Keep `drop()` where scoping hurts readability.
- **Binary parsing helper changes the error path**: The shared helper will return a generic error that must be converted to the crate-specific error type. → Mitigation: Use `Result<T, &'static str>` or a simple enum in shared, let callers map to their own error type.
