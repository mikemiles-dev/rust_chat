## ADDED Requirements

### Requirement: Shared length-prefixed string extraction
The `shared` crate SHALL provide a function `extract_length_prefixed_string` that reads a 1-byte length prefix followed by that many bytes of UTF-8 string data from a byte slice at a given offset. The function SHALL return the extracted string and the new offset, or an error if the data is malformed.

#### Scenario: Successful extraction
- **WHEN** `extract_length_prefixed_string` is called with valid data containing `[len_byte][string_bytes...]` at the given offset
- **THEN** it SHALL return `Ok((extracted_str, new_offset))` where `new_offset = offset + 1 + len_byte`

#### Scenario: Insufficient data for length byte
- **WHEN** the offset is at or beyond the end of the byte slice
- **THEN** the function SHALL return an error indicating insufficient data

#### Scenario: Insufficient data for string content
- **WHEN** the length byte indicates N bytes but fewer than N bytes remain after the length byte
- **THEN** the function SHALL return an error indicating insufficient data

#### Scenario: Invalid UTF-8
- **WHEN** the extracted bytes are not valid UTF-8
- **THEN** the function SHALL return an error indicating invalid string data

### Requirement: Binary format validation helper
The `shared` crate SHALL provide a function `validate_binary_length` that checks whether a byte slice has at least the required number of bytes remaining from a given offset.

#### Scenario: Sufficient length
- **WHEN** `validate_binary_length` is called and `data.len() - offset >= required`
- **THEN** it SHALL return `Ok(())`

#### Scenario: Insufficient length
- **WHEN** `data.len() - offset < required`
- **THEN** it SHALL return an error indicating the data is too short

### Requirement: Callers use shared helpers
All file transfer binary parsing in `client/src/client.rs` (or `client/src/file_transfer.rs` after extraction) and `server/src/user_connection/handlers.rs` (or `file_transfer_handlers.rs`) SHALL use the shared parsing helpers instead of inline parsing logic. Duplicated format validation code SHALL be removed.

#### Scenario: Server file transfer parsing uses shared helper
- **WHEN** `process_file_transfer`, `process_file_transfer_request`, or `process_file_transfer_response` parses binary content
- **THEN** it SHALL call `extract_length_prefixed_string` from the `shared` crate instead of inline byte manipulation

#### Scenario: Client file transfer parsing uses shared helper
- **WHEN** `handle_file_transfer`, `handle_file_transfer_request`, or `handle_file_transfer_response` parses binary content
- **THEN** it SHALL call `extract_length_prefixed_string` from the `shared` crate instead of inline byte manipulation

#### Scenario: Error mapping preserved
- **WHEN** the shared helper returns an error
- **THEN** the caller SHALL map it to its crate-specific error type (e.g., `UserConnectionError` or `ChatClientError`) with a message equivalent to the original inline error
