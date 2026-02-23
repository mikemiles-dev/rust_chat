use shared::logger;
use shared::message::{ChatMessage, MessageTypes, extract_length_prefixed_string, validate_binary_length};
use shared::network::TcpMessageHandler;

use crate::client::{ChatClient, ChatClientError, PendingIncomingTransfer};

impl ChatClient {
    pub(crate) fn handle_file_transfer(&self, message: &ChatMessage) {
        let content = match message.get_content() {
            Some(c) => c,
            None => {
                logger::log_error("Received empty file transfer");
                return;
            }
        };

        // Parse binary format: recipient_len(1)|recipient|sender_len(1)|sender|filename_len(1)|filename|filedata
        let (recipient, offset) = match extract_length_prefixed_string(content, 0) {
            Ok(v) => v,
            Err(_) => {
                logger::log_error("Invalid file transfer format");
                return;
            }
        };

        if recipient != self.chat_name {
            return;
        }

        let (sender, offset) = match extract_length_prefixed_string(content, offset) {
            Ok(v) => v,
            Err(_) => {
                logger::log_error("Invalid sender name in file transfer");
                return;
            }
        };

        let (filename, offset) = match extract_length_prefixed_string(content, offset) {
            Ok(v) => v,
            Err(_) => {
                logger::log_error("Invalid filename in file transfer");
                return;
            }
        };

        let file_data = &content[offset..];

        logger::log_warning(&format!(
            "[FILE from {}]: '{}' ({} bytes)",
            sender,
            filename,
            file_data.len()
        ));

        let save_path = format!("downloads/{}", filename);

        if let Err(e) = std::fs::create_dir_all("downloads") {
            logger::log_error(&format!("Failed to create downloads directory: {}", e));
            return;
        }

        match std::fs::write(&save_path, file_data) {
            Ok(_) => {
                logger::log_success(&format!("File saved to: {}", save_path));
            }
            Err(e) => {
                logger::log_error(&format!("Failed to save file: {}", e));
            }
        }
    }

    pub(crate) fn handle_file_transfer_request(&mut self, message: &ChatMessage) {
        let content = match message.get_content() {
            Some(c) => c,
            None => {
                logger::log_error("Received empty file transfer request");
                return;
            }
        };

        // Parse binary format: recipient_len(1)|recipient|sender_len(1)|sender|filename_len(1)|filename|filesize(8 bytes)
        let (recipient, offset) = match extract_length_prefixed_string(content, 0) {
            Ok(v) => v,
            Err(_) => {
                logger::log_error("Invalid file transfer request format");
                return;
            }
        };

        if recipient != self.chat_name {
            return;
        }

        let (sender, offset) = match extract_length_prefixed_string(content, offset) {
            Ok(v) => v,
            Err(_) => {
                logger::log_error("Invalid sender name in file transfer request");
                return;
            }
        };

        let (filename, offset) = match extract_length_prefixed_string(content, offset) {
            Ok(v) => v,
            Err(_) => {
                logger::log_error("Invalid filename in file transfer request");
                return;
            }
        };

        if validate_binary_length(content, offset, 8).is_err() {
            logger::log_error("Invalid file transfer request format");
            return;
        }

        let file_size = u64::from_be_bytes(
            content[offset..offset + 8]
                .try_into()
                .unwrap_or([0u8; 8]),
        ) as usize;

        self.pending_incoming.insert(
            sender.to_string(),
            PendingIncomingTransfer {
                file_name: filename.to_string(),
            },
        );

        let size_display = if file_size >= 1024 * 1024 {
            format!("{:.1} MB", file_size as f64 / (1024.0 * 1024.0))
        } else if file_size >= 1024 {
            format!("{:.1} KB", file_size as f64 / 1024.0)
        } else {
            format!("{} bytes", file_size)
        };

        logger::log_warning(&format!(
            "[FILE REQUEST from {}]: '{}' ({})",
            sender, filename, size_display
        ));
        logger::log_info(&format!(
            "Use /accept {} to accept or /reject {} to decline",
            sender, sender
        ));
    }

    pub(crate) async fn handle_file_transfer_response(&mut self, message: &ChatMessage) -> bool {
        let content = match message.get_content() {
            Some(c) => c,
            None => {
                logger::log_error("Received empty file transfer response");
                return true;
            }
        };

        // Parse format: recipient_len(1)|recipient|sender_len(1)|sender|accepted(1)
        let (recipient, offset) = match extract_length_prefixed_string(content, 0) {
            Ok(v) => v,
            Err(_) => {
                logger::log_error("Invalid file transfer response format");
                return true;
            }
        };

        if recipient != self.chat_name {
            return true;
        }

        let (responder, offset) = match extract_length_prefixed_string(content, offset) {
            Ok(v) => v,
            Err(_) => {
                logger::log_error("Invalid sender name in file transfer response");
                return true;
            }
        };

        if validate_binary_length(content, offset, 1).is_err() {
            logger::log_error("Invalid file transfer response format");
            return true;
        }

        let accepted = content[offset] == 1;

        if accepted {
            if let Some(transfer) = self.pending_outgoing.remove(responder) {
                logger::log_success(&format!(
                    "{} accepted file transfer for '{}'",
                    responder, transfer.file_name
                ));
                if let Err(e) = self
                    .send_file_data(&transfer.recipient, &transfer.file_path)
                    .await
                {
                    logger::log_error(&format!("Failed to send file: {:?}", e));
                }
            } else {
                logger::log_warning(&format!(
                    "Received acceptance from {} but no pending transfer found",
                    responder
                ));
            }
        } else {
            if let Some(transfer) = self.pending_outgoing.remove(responder) {
                logger::log_warning(&format!(
                    "{} rejected file transfer for '{}'",
                    responder, transfer.file_name
                ));
            } else {
                logger::log_warning(&format!(
                    "Received rejection from {} but no pending transfer found",
                    responder
                ));
            }
        }

        true
    }

    pub(crate) async fn accept_file_transfer(&mut self, sender: &str) -> Result<(), ChatClientError> {
        if let Some(transfer) = self.pending_incoming.remove(sender) {
            logger::log_info(&format!(
                "Accepting file '{}' from {}...",
                transfer.file_name, sender
            ));

            let mut content = Vec::new();
            content.push(sender.len() as u8);
            content.extend_from_slice(sender.as_bytes());
            content.push(1u8);

            let message = ChatMessage::try_new(MessageTypes::FileTransferResponse, Some(content))?;
            self.send_message_chunked(message).await?;
            Ok(())
        } else {
            logger::log_error(&format!("No pending file transfer from '{}'", sender));
            Ok(())
        }
    }

    pub(crate) async fn reject_file_transfer(&mut self, sender: &str) -> Result<(), ChatClientError> {
        if let Some(transfer) = self.pending_incoming.remove(sender) {
            logger::log_info(&format!(
                "Rejecting file '{}' from {}",
                transfer.file_name, sender
            ));

            let mut content = Vec::new();
            content.push(sender.len() as u8);
            content.extend_from_slice(sender.as_bytes());
            content.push(0u8);

            let message = ChatMessage::try_new(MessageTypes::FileTransferResponse, Some(content))?;
            self.send_message_chunked(message).await?;
            Ok(())
        } else {
            logger::log_error(&format!("No pending file transfer from '{}'", sender));
            Ok(())
        }
    }
}
