use rand::Rng;
use shared::logger;
use shared::message::{ChatMessage, MessageTypes};
use shared::network::TcpMessageHandler;
use std::collections::{HashMap, HashSet};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::{RwLock, broadcast};

use super::error::UserConnectionError;
use super::rate_limiting::RateLimiter;

// Helper struct to implement TcpMessageHandler for any AsyncRead + AsyncWrite stream
pub(super) struct StreamWrapper<'a, S> {
    pub(super) stream: &'a mut S,
}

impl<'a, S: AsyncRead + AsyncWrite + Unpin> TcpMessageHandler for StreamWrapper<'a, S> {
    type Stream = S;
    fn get_stream(&mut self) -> &mut Self::Stream {
        self.stream
    }
}

// Security limits
pub const MAX_USERNAME_LENGTH: usize = 32;
pub const MAX_MESSAGE_LENGTH: usize = 1024; // 1KB max message content
pub const MAX_STATUS_LENGTH: usize = 128; // Max status message length

pub struct MessageHandlers<'a> {
    pub addr: SocketAddr,
    pub tx: &'a broadcast::Sender<(ChatMessage, SocketAddr)>,
    pub server_commands: &'a broadcast::Sender<crate::ServerCommand>,
    pub connected_clients: &'a Arc<RwLock<HashSet<String>>>,
    pub user_ips: &'a Arc<RwLock<HashMap<String, IpAddr>>>,
    pub user_statuses: &'a Arc<RwLock<HashMap<String, String>>>,
    pub user_sessions: &'a Arc<RwLock<HashMap<String, String>>>,
}

impl<'a> MessageHandlers<'a> {
    pub(super) fn require_username(
        &self,
        username: &Option<String>,
        action: &str,
    ) -> Result<String, UserConnectionError> {
        match username {
            Some(name) => Ok(name.clone()),
            None => {
                logger::log_warning(&format!(
                    "User at {} tried to {} before joining",
                    self.addr, action
                ));
                Err(UserConnectionError::InvalidMessage)
            }
        }
    }

    pub fn randomize_username(&self, username: &str) -> String {
        let mut rng = rand::thread_rng();
        let random_suffix: u32 = rng.gen_range(1000..9999);
        format!("{}_{}", username, random_suffix)
    }

    pub async fn process_message<S: AsyncRead + AsyncWrite + Unpin>(
        &self,
        message: ChatMessage,
        rate_limiter: &mut RateLimiter,
        stream: &mut S,
        username: &mut Option<String>,
    ) -> Result<(), UserConnectionError> {
        let mut tcp_handler = StreamWrapper { stream };
        // Rate limiting check (except for Join messages)
        if !matches!(message.msg_type, MessageTypes::Join) && !rate_limiter.check_and_consume() {
            logger::log_warning(&format!("Rate limit exceeded for {}", self.addr));
            let error_msg = ChatMessage::try_new(
                MessageTypes::Error,
                Some(b"Rate limit exceeded. Please slow down.".to_vec()),
            )
            .map_err(|_| UserConnectionError::InvalidMessage)?;
            tcp_handler
                .send_message_chunked(error_msg)
                .await
                .map_err(UserConnectionError::IoError)?;
            return Ok(());
        }

        match message.msg_type {
            MessageTypes::VersionCheck => {
                self.process_version_check(message.content_as_string(), &mut tcp_handler)
                    .await?;
            }
            MessageTypes::Join => {
                self.process_join(message.content_as_string(), &mut tcp_handler, username)
                    .await?;
            }
            MessageTypes::ChatMessage => {
                self.process_chat_message(message.content_as_string(), username)
                    .await?;
            }
            MessageTypes::ListUsers => {
                self.process_list_users(&mut tcp_handler).await?;
            }
            MessageTypes::DirectMessage => {
                self.process_direct_message(
                    message.content_as_string(),
                    &mut tcp_handler,
                    username,
                )
                .await?;
            }
            MessageTypes::RenameRequest => {
                self.process_rename_request(
                    message.content_as_string(),
                    &mut tcp_handler,
                    username,
                )
                .await?;
            }
            MessageTypes::FileTransfer => {
                self.process_file_transfer(message.get_content(), &mut tcp_handler, username)
                    .await?;
            }
            MessageTypes::FileTransferRequest => {
                self.process_file_transfer_request(
                    message.get_content(),
                    &mut tcp_handler,
                    username,
                )
                .await?;
            }
            MessageTypes::FileTransferResponse => {
                self.process_file_transfer_response(
                    message.get_content(),
                    &mut tcp_handler,
                    username,
                )
                .await?;
            }
            MessageTypes::SetStatus => {
                self.process_set_status(message.content_as_string(), &mut tcp_handler, username)
                    .await?;
            }
            MessageTypes::Leave => {
                // User explicitly quit - signal this to the connection handler
                return Err(UserConnectionError::ExplicitQuit);
            }
            _ => (),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_username_validation_valid() {
        // Valid usernames
        assert_eq!("alice".len(), 5);
        assert!(
            "alice"
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        );

        assert_eq!("Bob123".len(), 6);
        assert!(
            "Bob123"
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        );

        assert_eq!("user_name".len(), 9);
        assert!(
            "user_name"
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        );

        assert_eq!("user-name".len(), 9);
        assert!(
            "user-name"
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        );
    }

    #[test]
    fn test_username_validation_invalid_chars() {
        // Invalid characters
        assert!(
            !"user@name"
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        );
        assert!(
            !"user name"
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        );
        assert!(
            !"user!name"
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        );
        assert!(
            !"user.name"
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        );
    }

    #[test]
    fn test_username_validation_length() {
        // Too short
        let empty = "";
        assert!(empty.is_empty());

        // Valid length
        let valid = "a".repeat(32);
        assert_eq!(valid.len(), 32);
        assert!(valid.len() <= MAX_USERNAME_LENGTH);

        // Too long
        let too_long = "a".repeat(33);
        assert!(too_long.len() > MAX_USERNAME_LENGTH);
    }

    #[test]
    fn test_message_length_validation() {
        // Valid message
        let valid = "Hello, World!";
        assert!(!valid.is_empty());
        assert!(valid.len() <= MAX_MESSAGE_LENGTH);

        // Empty message
        let empty = "";
        assert!(empty.is_empty());

        // Too long message
        let too_long = "x".repeat(MAX_MESSAGE_LENGTH + 1);
        assert!(too_long.len() > MAX_MESSAGE_LENGTH);
    }
}
