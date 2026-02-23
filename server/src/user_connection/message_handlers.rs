use shared::logger;
use shared::message::{ChatMessage, MessageTypes};
use shared::network::TcpMessageHandler;
use tokio::io::{AsyncRead, AsyncWrite};

use super::error::UserConnectionError;
use super::handlers::{MessageHandlers, StreamWrapper, MAX_MESSAGE_LENGTH, MAX_STATUS_LENGTH};

impl<'a> MessageHandlers<'a> {
    pub(super) async fn process_list_users<S: AsyncRead + AsyncWrite + Unpin>(
        &self,
        tcp_handler: &mut StreamWrapper<'_, S>,
    ) -> Result<(), UserConnectionError> {
        let user_list = {
            let clients = self.connected_clients.read().await;
            let statuses = self.user_statuses.read().await;

            clients
                .iter()
                .map(|username| {
                    if let Some(status) = statuses.get(username) {
                        format!("{} - {}", username, status)
                    } else {
                        username.clone()
                    }
                })
                .collect::<Vec<String>>()
        };

        let list_message = ChatMessage::try_new(
            MessageTypes::ListUsers,
            Some(user_list.join("\n").into_bytes()),
        )
        .map_err(|_| UserConnectionError::InvalidMessage)?;
        tcp_handler
            .send_message_chunked(list_message)
            .await
            .map_err(UserConnectionError::IoError)?;
        Ok(())
    }

    pub(super) async fn process_chat_message(
        &self,
        content: Option<String>,
        username: &Option<String>,
    ) -> Result<(), UserConnectionError> {
        let chat_content = content.ok_or(UserConnectionError::InvalidMessage)?;

        // Validate message length
        if chat_content.is_empty() || chat_content.len() > MAX_MESSAGE_LENGTH {
            logger::log_warning(&format!(
                "Invalid message length from {}: {} chars",
                self.addr,
                chat_content.len()
            ));
            return Err(UserConnectionError::InvalidMessage);
        }

        let username = self.require_username(username, "send chat message")?;
        let full_message = format!("{}: {}", username, chat_content);
        logger::log_chat(&full_message);
        let broadcast_message =
            ChatMessage::try_new(MessageTypes::ChatMessage, Some(full_message.into_bytes()))
                .map_err(|_| UserConnectionError::InvalidMessage)?;
        self.tx
            .send((broadcast_message, self.addr))
            .map_err(UserConnectionError::BroadcastError)?;
        Ok(())
    }

    pub(super) async fn process_direct_message<S: AsyncRead + AsyncWrite + Unpin>(
        &self,
        content: Option<String>,
        tcp_handler: &mut StreamWrapper<'_, S>,
        username: &Option<String>,
    ) -> Result<(), UserConnectionError> {
        let content = content.ok_or(UserConnectionError::InvalidMessage)?;

        if let Some((recipient, message)) = content.split_once('|') {
            // Validate message length
            if message.is_empty() || message.len() > MAX_MESSAGE_LENGTH {
                logger::log_warning(&format!(
                    "Invalid DM length from {}: {} chars",
                    self.addr,
                    message.len()
                ));
                return Err(UserConnectionError::InvalidMessage);
            }
            let sender = self.require_username(username, "send direct message")?;
            // Check if recipient exists
            let recipient_exists = {
                let clients = self.connected_clients.read().await;
                clients.contains(recipient)
            };
            if !recipient_exists {
                let error_msg = format!("User '{}' not found", recipient);
                logger::log_warning(&format!(
                    "[DM] {} -> {} (user not found)",
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

            // Log that a DM is happening, but don't show the content
            logger::log_system(&format!("[DM] {} -> {}", sender, recipient));

            // Format: sender|recipient|message for client filtering
            let dm_content = format!("{}|{}|{}", sender, recipient, message);
            let dm_message = ChatMessage::try_new(
                MessageTypes::DirectMessage,
                Some(dm_content.into_bytes()),
            )
            .map_err(|_| UserConnectionError::InvalidMessage)?;

            // Broadcast to all clients (clients will filter)
            self.tx
                .send((dm_message, self.addr))
                .map_err(UserConnectionError::BroadcastError)?;
            Ok(())
        } else {
            Err(UserConnectionError::InvalidMessage)
        }
    }

    pub(super) async fn process_set_status<S: AsyncRead + AsyncWrite + Unpin>(
        &self,
        status: Option<String>,
        tcp_handler: &mut StreamWrapper<'_, S>,
        username: &Option<String>,
    ) -> Result<(), UserConnectionError> {
        let username = self.require_username(username, "set status")?;

        let status_text = status.unwrap_or_default();

        // Validate status length
        if status_text.len() > MAX_STATUS_LENGTH {
            let error_msg = ChatMessage::try_new(
                MessageTypes::Error,
                Some(
                    format!("Status too long (max {} characters)", MAX_STATUS_LENGTH).into_bytes(),
                ),
            )
            .map_err(|_| UserConnectionError::InvalidMessage)?;
            tcp_handler
                .send_message_chunked(error_msg)
                .await
                .map_err(UserConnectionError::IoError)?;
            return Ok(());
        }

        // Update or remove status
        {
            let mut statuses = self.user_statuses.write().await;
            if status_text.is_empty() {
                statuses.remove(&username);
                logger::log_system(&format!("{} cleared their status", username));
            } else {
                statuses.insert(username.clone(), status_text.clone());
                logger::log_system(&format!("{} set status: {}", username, status_text));
            }
        }

        // Send confirmation back to client
        let confirm_msg = if status_text.is_empty() {
            "Status cleared".to_string()
        } else {
            format!("Status set to: {}", status_text)
        };
        let response =
            ChatMessage::try_new(MessageTypes::SetStatus, Some(confirm_msg.into_bytes()))
                .map_err(|_| UserConnectionError::InvalidMessage)?;
        tcp_handler
            .send_message_chunked(response)
            .await
            .map_err(UserConnectionError::IoError)?;

        Ok(())
    }
}
