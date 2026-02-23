## ADDED Requirements

### Requirement: File size cast uses TryFrom
The client SHALL convert `metadata.len()` (u64) to `usize` using `usize::try_from()` in `client/src/client.rs`, returning an error if the value exceeds `usize::MAX`.

#### Scenario: File size fits in usize
- **WHEN** a file's metadata length is within `usize::MAX`
- **THEN** the conversion succeeds and file processing continues

#### Scenario: File size exceeds usize on 32-bit target
- **WHEN** a file's metadata length exceeds `usize::MAX`
- **THEN** the client logs an error and returns without panicking

### Requirement: Parsed file size cast uses TryFrom
The client SHALL convert the parsed `u64` file size to `usize` using `usize::try_from()` in `client/src/file_transfer.rs`, instead of an unchecked `as usize` cast.

#### Scenario: Received file size fits in usize
- **WHEN** a file transfer request contains a file size within `usize::MAX`
- **THEN** the conversion succeeds and the request is processed

#### Scenario: Received file size overflows usize
- **WHEN** a file transfer request contains a file size exceeding `usize::MAX`
- **THEN** the client logs an error and discards the request without panicking

### Requirement: Length-prefixed string names validated before cast
All code that builds binary messages with length-prefixed strings SHALL validate that the string length fits in `u8` before casting with `as u8`. This applies to `client/src/client.rs`, `client/src/file_transfer.rs`, and `server/src/user_connection/file_transfer_handlers.rs`.

#### Scenario: Username within u8 length limit
- **WHEN** a username or filename is 255 bytes or fewer
- **THEN** the length is cast to `u8` and the message is built successfully

#### Scenario: Username exceeds u8 length limit
- **WHEN** a username or filename exceeds 255 bytes
- **THEN** the operation returns an error instead of silently truncating

### Requirement: File size bytes parsed without silent fallback
The client SHALL replace `unwrap_or([0u8; 8])` with a proper error return in `client/src/file_transfer.rs` when parsing the 8-byte file size from a transfer request.

#### Scenario: Valid 8-byte file size slice
- **WHEN** the content has exactly 8 bytes at the expected offset
- **THEN** the bytes are parsed into a `u64` file size successfully

#### Scenario: Malformed file size slice
- **WHEN** the content slice cannot be converted to `[u8; 8]`
- **THEN** the client logs an error and returns instead of defaulting to zero
