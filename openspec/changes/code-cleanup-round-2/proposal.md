## Why

The first cleanup round addressed surface-level hygiene (dead code, naming, lock safety). Deeper issues remain: magic numbers scattered across binary parsing and network setup, unchecked integer casts that risk panics or silent truncation, inconsistent error handling in file transfer code, a 700+ line handler file begging to be split, and duplicated completer/readline logic between client and server.

## What Changes

- Extract all hardcoded numeric values (ports, timeouts, buffer sizes, message type bytes, channel capacities) into named constants.
- Replace unchecked `as u8` / `as usize` casts with validated conversions using `TryFrom` or explicit bounds checks.
- Standardize error handling in file transfer parsing — eliminate silent failures and unify the repeated extract-or-return pattern.
- Split `server/src/user_connection/handlers.rs` into focused submodules (user lifecycle, messaging, admin commands).
- Move shared completer and readline spawning logic into the `shared` crate, with client and server using configuration to customize behavior.

## Capabilities

### New Capabilities
- `named-constants`: Extract magic numbers and hardcoded values into well-named constants across all crates.
- `safe-integer-casting`: Replace unchecked `as` casts with validated conversions to prevent overflow and truncation.
- `error-handling-consistency`: Unify file transfer parsing error patterns and eliminate silent failures.
- `handler-module-split`: Break `handlers.rs` into smaller, focused submodules by responsibility.
- `shared-readline`: Consolidate duplicated completer and readline helper code into a shared, configurable module.

### Modified Capabilities

## Impact

- **client/src/client.rs**: Constants extraction, cast fixes.
- **client/src/file_transfer.rs**: Error handling refactor, cast fixes.
- **client/src/completer.rs**: Replaced by shared completer.
- **client/src/readline_helper.rs**: Replaced by shared readline helper.
- **server/src/main.rs**: Constants extraction.
- **server/src/user_connection/handlers.rs**: Split into submodules, error handling cleanup.
- **server/src/user_connection/file_transfer_handlers.rs**: Error handling refactor, cast fixes.
- **server/src/completer.rs**: Replaced by shared completer.
- **server/src/readline_helper.rs**: Replaced by shared readline helper.
- **shared/src/message.rs**: Constants for message type bytes.
- **shared/src/lib.rs**: New completer/readline modules.
- **shared/Cargo.toml**: Add `rustyline` and `tokio` dependencies.
