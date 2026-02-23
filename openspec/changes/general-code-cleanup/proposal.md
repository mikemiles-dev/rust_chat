## Why

The codebase has accumulated dead code, duplicated logic, and inconsistent patterns that reduce readability and maintainability. A cleanup pass now — before adding new features — will make the code easier to work with and reduce the surface area for bugs.

## What Changes

- Remove dead code: unused struct fields with `#[allow(dead_code)]`, unused enum variants (`InvalidUser`), and unused function parameters
- Extract duplicated binary message parsing logic (file transfer format validation) into shared helpers
- Replace explicit `drop()` calls on lock guards with scoped blocks for improved readability
- Standardize naming conventions (mixed use of `chat_name` vs `username` for user identifiers)
- Decompose large files: extract file transfer handlers from `client/src/client.rs` (~1,150 lines) and `server/src/user_connection/handlers.rs` (~1,035 lines) into dedicated modules
- Improve lock unwrap safety: replace bare `.unwrap()` on lock acquisitions with `.expect()` messages

## Capabilities

### New Capabilities

- `code-hygiene`: Standards for dead code removal, naming conventions, and lock safety patterns across the codebase
- `binary-parsing`: Shared helpers for extracting and validating fields from binary wire-format messages (file transfers)

### Modified Capabilities

_None — this is a refactor with no behavior changes._

## Impact

- **Code affected**: `client/src/client.rs`, `client/src/completer.rs`, `client/src/readline_helper.rs`, `server/src/main.rs`, `server/src/user_connection/handlers.rs`, `shared/src/input.rs`
- **APIs**: No public API changes — all modifications are internal
- **Dependencies**: None
- **Risk**: Low — pure refactor, no functional changes. All existing behavior preserved.
