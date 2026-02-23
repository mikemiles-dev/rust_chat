/// Protocol header size: 4 bytes for message length + 1 byte for message type.
pub const HEADER_SIZE: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MessageTypes {
    ChatMessage,
    Join,
    Leave,
    UserRename,
    ListUsers,
    DirectMessage,
    Error,
    RenameRequest,
    FileTransfer,         // File data being sent: recipient|sender|filename|data
    FileTransferAck,      // Acknowledgment that file was received
    FileTransferRequest,  // Request to send file: recipient|filename|filesize
    FileTransferResponse, // Response to request: sender|accepted (0/1)
    SetStatus,            // Set user's status message
    Ping,                 // Server heartbeat to check if client is alive
    Pong,                 // Client response to Ping
    VersionCheck,         // Client sends version to server on connection: version string
    VersionMismatch, // Server responds with mismatch error: client_version|server_version|readme_url
    Unknown(u8),
}

impl MessageTypes {
    pub const CHAT_MESSAGE_BYTE: u8 = 1;
    pub const JOIN_BYTE: u8 = 2;
    pub const LEAVE_BYTE: u8 = 3;
    pub const USER_RENAME_BYTE: u8 = 4;
    pub const LIST_USERS_BYTE: u8 = 5;
    pub const DIRECT_MESSAGE_BYTE: u8 = 6;
    pub const ERROR_BYTE: u8 = 7;
    pub const RENAME_REQUEST_BYTE: u8 = 8;
    pub const FILE_TRANSFER_BYTE: u8 = 9;
    pub const FILE_TRANSFER_ACK_BYTE: u8 = 10;
    pub const FILE_TRANSFER_REQUEST_BYTE: u8 = 11;
    pub const FILE_TRANSFER_RESPONSE_BYTE: u8 = 12;
    pub const SET_STATUS_BYTE: u8 = 13;
    pub const PING_BYTE: u8 = 14;
    pub const PONG_BYTE: u8 = 15;
    pub const VERSION_CHECK_BYTE: u8 = 16;
    pub const VERSION_MISMATCH_BYTE: u8 = 17;
}

impl From<u8> for MessageTypes {
    fn from(value: u8) -> Self {
        match value {
            Self::CHAT_MESSAGE_BYTE => MessageTypes::ChatMessage,
            Self::JOIN_BYTE => MessageTypes::Join,
            Self::LEAVE_BYTE => MessageTypes::Leave,
            Self::USER_RENAME_BYTE => MessageTypes::UserRename,
            Self::LIST_USERS_BYTE => MessageTypes::ListUsers,
            Self::DIRECT_MESSAGE_BYTE => MessageTypes::DirectMessage,
            Self::ERROR_BYTE => MessageTypes::Error,
            Self::RENAME_REQUEST_BYTE => MessageTypes::RenameRequest,
            Self::FILE_TRANSFER_BYTE => MessageTypes::FileTransfer,
            Self::FILE_TRANSFER_ACK_BYTE => MessageTypes::FileTransferAck,
            Self::FILE_TRANSFER_REQUEST_BYTE => MessageTypes::FileTransferRequest,
            Self::FILE_TRANSFER_RESPONSE_BYTE => MessageTypes::FileTransferResponse,
            Self::SET_STATUS_BYTE => MessageTypes::SetStatus,
            Self::PING_BYTE => MessageTypes::Ping,
            Self::PONG_BYTE => MessageTypes::Pong,
            Self::VERSION_CHECK_BYTE => MessageTypes::VersionCheck,
            Self::VERSION_MISMATCH_BYTE => MessageTypes::VersionMismatch,
            other => MessageTypes::Unknown(other),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    msg_len: u32,
    pub msg_type: MessageTypes,
    content: Option<Vec<u8>>,
}

impl ChatMessage {
    pub fn get_content(&self) -> Option<&[u8]> {
        self.content.as_deref()
    }

    pub fn content_as_string(&self) -> Option<String> {
        self.content
            .as_ref()
            .and_then(|data| String::from_utf8(data.clone()).ok())
    }
}

#[derive(Debug)]
pub enum ChatMessageError {
    InvalidFormat,
    InvalidLength,
}

impl ChatMessage {
    pub fn try_new(
        msg_type: MessageTypes,
        content: Option<Vec<u8>>,
    ) -> Result<Self, ChatMessageError> {
        let msg_len = match &content {
            Some(data) => data
                .len()
                .checked_add(HEADER_SIZE)
                .ok_or(ChatMessageError::InvalidLength)?,
            None => HEADER_SIZE,
        };
        Ok(ChatMessage {
            msg_len: u32::try_from(msg_len).map_err(|_| ChatMessageError::InvalidLength)?,
            msg_type,
            content,
        })
    }
}

// Protocol: [msg_len (4 bytes)][msg_type (1 byte)][content (msg_len - HEADER_SIZE bytes)]
impl From<Vec<u8>> for ChatMessage {
    fn from(buffer: Vec<u8>) -> Self {
        if buffer.len() < HEADER_SIZE {
            return ChatMessage {
                msg_len: HEADER_SIZE as u32,
                msg_type: MessageTypes::Unknown(0),
                content: None,
            };
        }
        let msg_len = u32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
        let msg_type = MessageTypes::from(buffer[4]);
        let content = if buffer.len() > HEADER_SIZE {
            Some(buffer[HEADER_SIZE..].to_vec())
        } else {
            None
        };

        ChatMessage {
            msg_len,
            msg_type,
            content,
        }
    }
}

impl From<ChatMessage> for Vec<u8> {
    fn from(message: ChatMessage) -> Self {
        let mut buffer = Vec::new();
        buffer.extend_from_slice(&message.msg_len.to_be_bytes());
        buffer.push(match message.msg_type {
            MessageTypes::ChatMessage => MessageTypes::CHAT_MESSAGE_BYTE,
            MessageTypes::Join => MessageTypes::JOIN_BYTE,
            MessageTypes::Leave => MessageTypes::LEAVE_BYTE,
            MessageTypes::UserRename => MessageTypes::USER_RENAME_BYTE,
            MessageTypes::ListUsers => MessageTypes::LIST_USERS_BYTE,
            MessageTypes::DirectMessage => MessageTypes::DIRECT_MESSAGE_BYTE,
            MessageTypes::Error => MessageTypes::ERROR_BYTE,
            MessageTypes::RenameRequest => MessageTypes::RENAME_REQUEST_BYTE,
            MessageTypes::FileTransfer => MessageTypes::FILE_TRANSFER_BYTE,
            MessageTypes::FileTransferAck => MessageTypes::FILE_TRANSFER_ACK_BYTE,
            MessageTypes::FileTransferRequest => MessageTypes::FILE_TRANSFER_REQUEST_BYTE,
            MessageTypes::FileTransferResponse => MessageTypes::FILE_TRANSFER_RESPONSE_BYTE,
            MessageTypes::SetStatus => MessageTypes::SET_STATUS_BYTE,
            MessageTypes::Ping => MessageTypes::PING_BYTE,
            MessageTypes::Pong => MessageTypes::PONG_BYTE,
            MessageTypes::VersionCheck => MessageTypes::VERSION_CHECK_BYTE,
            MessageTypes::VersionMismatch => MessageTypes::VERSION_MISMATCH_BYTE,
            MessageTypes::Unknown(val) => val,
        });
        if let Some(content) = message.content {
            buffer.extend_from_slice(&content);
        }
        buffer
    }
}

/// Push a length-prefixed string onto a buffer. Returns an error if the string exceeds 255 bytes.
pub fn push_length_prefixed(buf: &mut Vec<u8>, s: &str) -> Result<(), &'static str> {
    let len = u8::try_from(s.len()).map_err(|_| "string exceeds 255 bytes for length prefix")?;
    buf.push(len);
    buf.extend_from_slice(s.as_bytes());
    Ok(())
}

/// Validate that a byte slice has at least `required` bytes remaining from `offset`.
pub fn validate_binary_length(
    data: &[u8],
    offset: usize,
    required: usize,
) -> Result<(), &'static str> {
    if data.len().saturating_sub(offset) < required {
        return Err("insufficient data length");
    }
    Ok(())
}

/// Extract a length-prefixed string from `data` at `offset`.
/// Format: `[1-byte length][string bytes]`.
/// Returns the extracted string slice and the new offset past the string.
pub fn extract_length_prefixed_string(
    data: &[u8],
    offset: usize,
) -> Result<(&str, usize), &'static str> {
    validate_binary_length(data, offset, 1)?;
    let len = data[offset] as usize;
    let start = offset + 1;
    validate_binary_length(data, start, len)?;
    let s = std::str::from_utf8(&data[start..start + len])
        .map_err(|_| "invalid UTF-8 in length-prefixed string")?;
    Ok((s, start + len))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation_valid() {
        let content = b"Hello, World!".to_vec();
        let msg = ChatMessage::try_new(MessageTypes::ChatMessage, Some(content.clone()));
        assert!(msg.is_ok());
        let msg = msg.unwrap();
        assert_eq!(msg.msg_type, MessageTypes::ChatMessage);
        assert_eq!(msg.content, Some(content));
    }

    #[test]
    fn test_message_creation_none_content() {
        let msg = ChatMessage::try_new(MessageTypes::ListUsers, None);
        assert!(msg.is_ok());
        let msg = msg.unwrap();
        assert_eq!(msg.msg_len, HEADER_SIZE as u32);
        assert_eq!(msg.content, None);
    }

    #[test]
    fn test_message_serialization() {
        let content = b"Test".to_vec();
        let msg = ChatMessage::try_new(MessageTypes::ChatMessage, Some(content.clone())).unwrap();
        let serialized: Vec<u8> = msg.clone().into();

        // Check structure: [4 bytes len][1 byte type][content]
        assert_eq!(serialized.len(), 4 + 1 + content.len());
        assert_eq!(serialized[4], 1); // ChatMessage type
        assert_eq!(&serialized[5..], content.as_slice());
    }

    #[test]
    fn test_message_deserialization() {
        let mut buffer = vec![];
        buffer.extend_from_slice(&9u32.to_be_bytes()); // length (4 + 1 + 4 = 9)
        buffer.push(1); // ChatMessage type
        buffer.extend_from_slice(b"Test");

        let msg = ChatMessage::from(buffer);
        assert_eq!(msg.msg_type, MessageTypes::ChatMessage);
        assert_eq!(msg.content_as_string(), Some("Test".to_string()));
    }

    #[test]
    fn test_message_roundtrip() {
        let original_content = b"Hello, World!".to_vec();
        let original_msg =
            ChatMessage::try_new(MessageTypes::DirectMessage, Some(original_content.clone()))
                .unwrap();

        let serialized: Vec<u8> = original_msg.into();
        let deserialized = ChatMessage::from(serialized);

        assert_eq!(deserialized.msg_type, MessageTypes::DirectMessage);
        assert_eq!(deserialized.content, Some(original_content));
    }

    #[test]
    fn test_message_types_from_u8() {
        assert!(matches!(MessageTypes::from(1), MessageTypes::ChatMessage));
        assert!(matches!(MessageTypes::from(2), MessageTypes::Join));
        assert!(matches!(MessageTypes::from(3), MessageTypes::Leave));
        assert!(matches!(MessageTypes::from(4), MessageTypes::UserRename));
        assert!(matches!(MessageTypes::from(5), MessageTypes::ListUsers));
        assert!(matches!(MessageTypes::from(6), MessageTypes::DirectMessage));
        assert!(matches!(MessageTypes::from(7), MessageTypes::Error));
        assert!(matches!(MessageTypes::from(99), MessageTypes::Unknown(99)));
    }

    #[test]
    fn test_empty_buffer_deserialization() {
        let msg = ChatMessage::from(vec![]);
        assert_eq!(msg.msg_len, 5);
        assert!(matches!(msg.msg_type, MessageTypes::Unknown(0)));
        assert_eq!(msg.content, None);
    }

    #[test]
    fn test_short_buffer_deserialization() {
        let msg = ChatMessage::from(vec![0, 1]); // Too short
        assert_eq!(msg.msg_len, 5);
        assert!(matches!(msg.msg_type, MessageTypes::Unknown(0)));
    }

    #[test]
    fn test_content_as_string_valid_utf8() {
        let msg =
            ChatMessage::try_new(MessageTypes::ChatMessage, Some(b"Valid UTF-8".to_vec())).unwrap();
        assert_eq!(msg.content_as_string(), Some("Valid UTF-8".to_string()));
    }

    #[test]
    fn test_content_as_string_invalid_utf8() {
        let msg = ChatMessage::try_new(
            MessageTypes::ChatMessage,
            Some(vec![0xFF, 0xFE, 0xFD]), // Invalid UTF-8
        )
        .unwrap();
        assert_eq!(msg.content_as_string(), None);
    }

    #[test]
    fn test_validate_binary_length_sufficient() {
        assert!(validate_binary_length(&[0, 1, 2, 3], 0, 4).is_ok());
        assert!(validate_binary_length(&[0, 1, 2, 3], 2, 2).is_ok());
        assert!(validate_binary_length(&[0, 1, 2, 3], 4, 0).is_ok());
    }

    #[test]
    fn test_validate_binary_length_insufficient() {
        assert!(validate_binary_length(&[0, 1], 0, 3).is_err());
        assert!(validate_binary_length(&[0, 1], 2, 1).is_err());
        assert!(validate_binary_length(&[], 0, 1).is_err());
    }

    #[test]
    fn test_extract_length_prefixed_string_success() {
        // "hi" prefixed with length 2
        let data = [2, b'h', b'i', 99];
        let (s, offset) = extract_length_prefixed_string(&data, 0).unwrap();
        assert_eq!(s, "hi");
        assert_eq!(offset, 3);
    }

    #[test]
    fn test_extract_length_prefixed_string_at_offset() {
        let data = [0xFF, 3, b'f', b'o', b'o'];
        let (s, offset) = extract_length_prefixed_string(&data, 1).unwrap();
        assert_eq!(s, "foo");
        assert_eq!(offset, 5);
    }

    #[test]
    fn test_extract_length_prefixed_string_empty() {
        let data = [0];
        let (s, offset) = extract_length_prefixed_string(&data, 0).unwrap();
        assert_eq!(s, "");
        assert_eq!(offset, 1);
    }

    #[test]
    fn test_extract_length_prefixed_string_no_length_byte() {
        let data: [u8; 0] = [];
        assert!(extract_length_prefixed_string(&data, 0).is_err());
    }

    #[test]
    fn test_extract_length_prefixed_string_insufficient_content() {
        // Says 5 bytes but only 2 available
        let data = [5, b'a', b'b'];
        assert!(extract_length_prefixed_string(&data, 0).is_err());
    }

    #[test]
    fn test_extract_length_prefixed_string_invalid_utf8() {
        let data = [2, 0xFF, 0xFE];
        assert!(extract_length_prefixed_string(&data, 0).is_err());
    }
}
