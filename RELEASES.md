# 0.1.13
 * Removed dead code: unused struct fields, error variants, and `#[allow(dead_code)]` annotations.
 * Renamed `chat_name` to `username` throughout the server crate for consistency.
 * Added shared binary parsing helpers (`extract_length_prefixed_string`, `validate_binary_length`) to reduce duplicated parsing logic.
 * Replaced bare `.unwrap()` on lock acquisitions with descriptive `.expect()` messages.
 * Replaced explicit `drop()` calls with scoped blocks for clearer lock lifetimes.
 * Extracted file transfer logic into dedicated modules on both client and server.
 * Replaced magic numbers with named constants across client, server, and shared crates (`DEFAULT_PORT`, `RECONNECT_DELAY`, `FILE_HEADER_OVERHEAD`, `TLS_HANDSHAKE_TIMEOUT`, `HEADER_SIZE`, `MessageTypes` byte constants).
 * Replaced unsafe `as u8` and `as usize` casts with safe `TryFrom` conversions and added a `push_length_prefixed` helper for length-prefixed string encoding.
 * Added `extract_field!` macro (client) and `parse_field` helper (server) to reduce repetitive binary parsing boilerplate.
 * Added `require_username` helper on the server to eliminate duplicated username-check match blocks.
 * Implemented `Display` for `ChatClientError`.
 * Split `handlers.rs` (724 lines) into `handlers.rs`, `user_handlers.rs`, and `message_handlers.rs` (246 + 294 + 188 lines).
 * Consolidated duplicate `completer.rs` and `readline_helper.rs` from client and server into shared crate (`shared::completer`, `shared::readline`).

# 0.1.12
 * Ghost session reclaim: Reconnecting clients can now reclaim their own "ghost" session instead of being renamed. If you disconnect and reconnect quickly (before the 60s timeout), and your old session is still active, the server will recognize you and let you take over your username seamlessly.

# 0.1.11
 * Fixed terminal cursor disappearing after `/quit` command.

# 0.1.10
 * Increased file transfer size limit from 10MB to 100MB.
 * File transfers now require recipient acceptance. Sender uses `/send <user> <file>`, recipient must `/accept <sender>` or `/reject <sender>`.

# 0.1.9
 * Added client/server version checking. Clients with mismatched versions are disconnected with a link to upgrade instructions.

# 0.1.8
 * Status now persists across reconnections but is cleared on explicit `/quit`, kick, or ban.

# 0.1.7
 * Refactored command parsing to use shared command constants, eliminating duplication between command definitions and input parsing.

# 0.1.6
 * Better client reconnection.

# 0.1.5
 * More robust dead connection logic.
 * Fixed issues with input cursor and ctrl + c

# 0.1.4
 * Centralize completer logic.

# 0.1.3
 * Added User Status Command
 * Server checks for disconnect cleanup

# 0.1.2
 * Added file transfer 

