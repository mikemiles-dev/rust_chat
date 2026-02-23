use shared::logger;
use shared::message::{ChatMessage, MessageTypes, extract_length_prefixed_string, validate_binary_length};
use shared::network::TcpMessageHandler;
use tokio::io::{AsyncRead, AsyncWrite};

use super::error::UserConnectionError;
use super::handlers::{MessageHandlers, StreamWrapper};

impl<'a> MessageHandlers<'a> {
    pub(super) async fn process_file_transfer<S: AsyncRead + AsyncWrite + Unpin>(
        &self,
        content: Option<&[u8]>,
        tcp_handler: &mut StreamWrapper<'_, S>,
        username: &Option<String>,
    ) -> Result<(), UserConnectionError> {
        let content = content.ok_or(UserConnectionError::InvalidMessage)?;

        let sender = match username {
            Some(name) => name.clone(),
            None => {
                logger::log_warning(&format!(
                    "User at {} tried to send file before joining",
                    self.addr
                ));
                return Err(UserConnectionError::InvalidMessage);
            }
        };

        // Parse binary format: recipient_len(1)|recipient|filename_len(1)|filename|filedata
        let (recipient, offset) = extract_length_prefixed_string(content, 0)
            .map_err(|_| {
                logger::log_warning(&format!("Invalid file transfer format from {}", self.addr));
                UserConnectionError::InvalidMessage
            })?;

        let (filename, offset) = extract_length_prefixed_string(content, offset)
            .map_err(|_| {
                logger::log_warning(&format!("Invalid file transfer format from {}", self.addr));
                UserConnectionError::InvalidMessage
            })?;

        let file_data = &content[offset..];

        // Check if recipient exists
        let recipient_exists = {
            let clients = self.connected_clients.read().await;
            clients.contains(recipient)
        };
        if !recipient_exists {
            let error_msg = format!("User '{}' not found", recipient);
            logger::log_warning(&format!(
                "[FILE] {} -> {} (user not found)",
                sender, recipient
            ));
            let error_message =
                ChatMessage::try_new(MessageTypes::Error, Some(error_msg.into_bytes()))
                    .map_err(|_| UserConnectionError::InvalidMessage)?;
            tcp_handler
                .send_message_chunked(error_message)
                .await
                .map_err(UserConnectionError::IoError)?;
            return Ok(());
        }

        logger::log_system(&format!(
            "[FILE] {} -> {} ('{}', {} bytes)",
            sender,
            recipient,
            filename,
            file_data.len()
        ));

        // Build outgoing message: recipient_len(1)|recipient|sender_len(1)|sender|filename_len(1)|filename|filedata
        let mut final_content = Vec::new();
        final_content.push(recipient.len() as u8);
        final_content.extend_from_slice(recipient.as_bytes());
        final_content.push(sender.len() as u8);
        final_content.extend_from_slice(sender.as_bytes());
        final_content.push(filename.len() as u8);
        final_content.extend_from_slice(filename.as_bytes());
        final_content.extend_from_slice(file_data);

        let file_message = ChatMessage::try_new(MessageTypes::FileTransfer, Some(final_content))
            .map_err(|_| UserConnectionError::InvalidMessage)?;

        self.tx
            .send((file_message, self.addr))
            .map_err(UserConnectionError::BroadcastError)?;

        Ok(())
    }

    pub(super) async fn process_file_transfer_request<S: AsyncRead + AsyncWrite + Unpin>(
        &self,
        content: Option<&[u8]>,
        tcp_handler: &mut StreamWrapper<'_, S>,
        username: &Option<String>,
    ) -> Result<(), UserConnectionError> {
        let content = content.ok_or(UserConnectionError::InvalidMessage)?;

        let sender = match username {
            Some(name) => name.clone(),
            None => {
                logger::log_warning(&format!(
                    "User at {} tried to send file request before joining",
                    self.addr
                ));
                return Err(UserConnectionError::InvalidMessage);
            }
        };

        // Parse binary format: recipient_len(1)|recipient|filename_len(1)|filename|filesize(8 bytes)
        let (recipient, offset) = extract_length_prefixed_string(content, 0)
            .map_err(|_| {
                logger::log_warning(&format!(
                    "Invalid file transfer request format from {}",
                    self.addr
                ));
                UserConnectionError::InvalidMessage
            })?;

        let (filename, offset) = extract_length_prefixed_string(content, offset)
            .map_err(|_| {
                logger::log_warning(&format!(
                    "Invalid file transfer request format from {}",
                    self.addr
                ));
                UserConnectionError::InvalidMessage
            })?;

        validate_binary_length(content, offset, 8).map_err(|_| {
            logger::log_warning(&format!(
                "Invalid file transfer request format from {}",
                self.addr
            ));
            UserConnectionError::InvalidMessage
        })?;

        let file_size = u64::from_be_bytes(
            content[offset..offset + 8]
                .try_into()
                .map_err(|_| UserConnectionError::InvalidMessage)?,
        );

        // Check if recipient exists
        let recipient_exists = {
            let clients = self.connected_clients.read().await;
            clients.contains(recipient)
        };
        if !recipient_exists {
            let error_msg = format!("User '{}' not found", recipient);
            logger::log_warning(&format!(
                "[FILE REQUEST] {} -> {} (user not found)",
                sender, recipient
            ));
            let error_message =
                ChatMessage::try_new(MessageTypes::Error, Some(error_msg.into_bytes()))
                    .map_err(|_| UserConnectionError::InvalidMessage)?;
            tcp_handler
                .send_message_chunked(error_message)
                .await
                .map_err(UserConnectionError::IoError)?;
            return Ok(());
        }

        logger::log_system(&format!(
            "[FILE REQUEST] {} -> {} ('{}', {} bytes)",
            sender, recipient, filename, file_size
        ));

        // Build outgoing message with sender info
        let mut outgoing_content = Vec::new();
        outgoing_content.push(recipient.len() as u8);
        outgoing_content.extend_from_slice(recipient.as_bytes());
        outgoing_content.push(sender.len() as u8);
        outgoing_content.extend_from_slice(sender.as_bytes());
        outgoing_content.push(filename.len() as u8);
        outgoing_content.extend_from_slice(filename.as_bytes());
        outgoing_content.extend_from_slice(&file_size.to_be_bytes());

        let request_message =
            ChatMessage::try_new(MessageTypes::FileTransferRequest, Some(outgoing_content))
                .map_err(|_| UserConnectionError::InvalidMessage)?;

        self.tx
            .send((request_message, self.addr))
            .map_err(UserConnectionError::BroadcastError)?;

        Ok(())
    }

    pub(super) async fn process_file_transfer_response<S: AsyncRead + AsyncWrite + Unpin>(
        &self,
        content: Option<&[u8]>,
        tcp_handler: &mut StreamWrapper<'_, S>,
        username: &Option<String>,
    ) -> Result<(), UserConnectionError> {
        let content = content.ok_or(UserConnectionError::InvalidMessage)?;

        let responder = match username {
            Some(name) => name.clone(),
            None => {
                logger::log_warning(&format!(
                    "User at {} tried to send file response before joining",
                    self.addr
                ));
                return Err(UserConnectionError::InvalidMessage);
            }
        };

        // Parse binary format: sender_len(1)|sender|accepted(1)
        let (original_sender, offset) = extract_length_prefixed_string(content, 0)
            .map_err(|_| {
                logger::log_warning(&format!(
                    "Invalid file transfer response format from {}",
                    self.addr
                ));
                UserConnectionError::InvalidMessage
            })?;

        validate_binary_length(content, offset, 1).map_err(|_| {
            logger::log_warning(&format!(
                "Invalid file transfer response format from {}",
                self.addr
            ));
            UserConnectionError::InvalidMessage
        })?;

        let accepted = content[offset] == 1;

        // Check if original sender exists
        let sender_exists = {
            let clients = self.connected_clients.read().await;
            clients.contains(original_sender)
        };
        if !sender_exists {
            let error_msg = format!("User '{}' not found", original_sender);
            logger::log_warning(&format!(
                "[FILE RESPONSE] {} -> {} (user not found)",
                responder, original_sender
            ));
            let error_message =
                ChatMessage::try_new(MessageTypes::Error, Some(error_msg.into_bytes()))
                    .map_err(|_| UserConnectionError::InvalidMessage)?;
            tcp_handler
                .send_message_chunked(error_message)
                .await
                .map_err(UserConnectionError::IoError)?;
            return Ok(());
        }

        logger::log_system(&format!(
            "[FILE RESPONSE] {} {} file from {}",
            responder,
            if accepted { "accepted" } else { "rejected" },
            original_sender
        ));

        // Build outgoing message
        let mut outgoing_content = Vec::new();
        outgoing_content.push(original_sender.len() as u8);
        outgoing_content.extend_from_slice(original_sender.as_bytes());
        outgoing_content.push(responder.len() as u8);
        outgoing_content.extend_from_slice(responder.as_bytes());
        outgoing_content.push(if accepted { 1u8 } else { 0u8 });

        let response_message =
            ChatMessage::try_new(MessageTypes::FileTransferResponse, Some(outgoing_content))
                .map_err(|_| UserConnectionError::InvalidMessage)?;

        self.tx
            .send((response_message, self.addr))
            .map_err(UserConnectionError::BroadcastError)?;

        Ok(())
    }
}
