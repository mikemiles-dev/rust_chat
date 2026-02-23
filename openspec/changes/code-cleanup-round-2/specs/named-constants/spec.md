## ADDED Requirements

### Requirement: Default port constant
The client SHALL use a named constant `DEFAULT_PORT` (value `8080`) in `client/src/client.rs` instead of a hardcoded literal when no port is specified in `parse_server_addr()`.

#### Scenario: Default port used when none specified
- **WHEN** a server address is parsed without a port component
- **THEN** the client uses the `DEFAULT_PORT` constant value `8080`

### Requirement: Reconnection delay constant
The client SHALL use a named constant `RECONNECT_DELAY` (value `Duration::from_millis(100)`) in `client/src/client.rs` instead of an inline literal in the reconnection logic.

#### Scenario: Reconnect uses named delay
- **WHEN** the client initiates a reconnection attempt
- **THEN** the pre-reconnect sleep uses the `RECONNECT_DELAY` constant

### Requirement: File transfer overhead constant
The client SHALL use a named constant `FILE_HEADER_OVERHEAD` (value `1024`) in `client/src/client.rs` instead of a hardcoded literal when calculating maximum content size.

#### Scenario: File size limit accounts for overhead
- **WHEN** a file is checked against the transfer size limit
- **THEN** the overhead subtracted from `MAX_FILE_SIZE` uses the `FILE_HEADER_OVERHEAD` constant

### Requirement: TLS handshake timeout constant
The server SHALL use a named constant `TLS_HANDSHAKE_TIMEOUT` (value `Duration::from_secs(30)`) in `server/src/main.rs` instead of an inline literal.

#### Scenario: TLS handshake uses named timeout
- **WHEN** the server awaits a TLS handshake from a connecting client
- **THEN** the timeout duration is `TLS_HANDSHAKE_TIMEOUT`

### Requirement: Broadcast channel capacity constants
The server SHALL use named constants `BROADCAST_BUFFER_MULTIPLIER` (value `16`) and `SERVER_CMD_CHANNEL_SIZE` (value `100`) in `server/src/main.rs` instead of inline literals when creating broadcast channels.

#### Scenario: Broadcast channel sized by constant
- **WHEN** the server creates the message broadcast channel
- **THEN** the capacity is `max_clients * BROADCAST_BUFFER_MULTIPLIER`

#### Scenario: Server command channel sized by constant
- **WHEN** the server creates the server commands broadcast channel
- **THEN** the capacity is `SERVER_CMD_CHANNEL_SIZE`

### Requirement: Message type byte constants
The `MessageTypes` enum in `shared/src/message.rs` SHALL define associated byte constants (e.g., `MessageTypes::CHAT_MESSAGE_BYTE = 1`) and use them in both `From<u8>` and `From<ChatMessage> for Vec<u8>` implementations, eliminating duplicated magic number literals.

#### Scenario: Serialization uses named byte constants
- **WHEN** a `ChatMessage` is serialized to `Vec<u8>`
- **THEN** each message type maps to its named byte constant, not a raw literal

#### Scenario: Deserialization uses named byte constants
- **WHEN** a byte is parsed into a `MessageTypes` variant
- **THEN** the match arms use named byte constants, not raw literals

### Requirement: Protocol header size constant
The `ChatMessage` module in `shared/src/message.rs` SHALL define a constant `HEADER_SIZE` (value `5`) representing the 4-byte length + 1-byte type prefix, and use it in `try_new()` and `From<Vec<u8>>`.

#### Scenario: Header size referenced by constant
- **WHEN** `ChatMessage::try_new()` calculates `msg_len`
- **THEN** it adds `HEADER_SIZE` instead of the literal `5`

#### Scenario: Parsing checks use header constant
- **WHEN** a `Vec<u8>` is parsed into a `ChatMessage`
- **THEN** length checks and content slicing reference `HEADER_SIZE`
