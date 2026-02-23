use crate::ServerCommand;
use shared::logger;
use shared::message::{ChatMessage, MessageTypes};
use shared::network::TcpMessageHandler;
use shared::version::{self, VERSION};
use tokio::io::{AsyncRead, AsyncWrite};

use super::error::UserConnectionError;
use super::handlers::{MAX_USERNAME_LENGTH, MessageHandlers, StreamWrapper};

impl<'a> MessageHandlers<'a> {
    pub(super) async fn process_join<S: AsyncRead + AsyncWrite + Unpin>(
        &self,
        join_content: Option<String>,
        tcp_handler: &mut StreamWrapper<'_, S>,
        username: &mut Option<String>,
    ) -> Result<(), UserConnectionError> {
        let content = join_content.ok_or(UserConnectionError::InvalidMessage)?;

        // Parse username and session token (format: username|session_token)
        let (requested_username, session_token) =
            if let Some((user, token)) = content.split_once('|') {
                (user.to_string(), Some(token.to_string()))
            } else {
                // Backwards compatibility: if no session token, just use the username
                (content, None)
            };

        // Validate username length
        if requested_username.is_empty() || requested_username.len() > MAX_USERNAME_LENGTH {
            logger::log_warning(&format!(
                "Invalid username length from {}: {} chars",
                self.addr,
                requested_username.len()
            ));
            return Err(UserConnectionError::InvalidMessage);
        }

        // Validate username characters (alphanumeric, underscore, hyphen only)
        if !requested_username
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            logger::log_warning(&format!(
                "Invalid username characters from {}: {}",
                self.addr, requested_username
            ));
            return Err(UserConnectionError::InvalidMessage);
        }

        let connected_clients = self.connected_clients.clone();
        {
            let mut clients = connected_clients.write().await;

            // Check if username already exists
            if clients.contains(&requested_username) {
                // Username exists - check if this is a valid reconnection (same session token and IP)
                let can_reclaim = if let Some(ref token) = session_token {
                    let (session_matches, ip_matches) = {
                        let sessions = self.user_sessions.read().await;
                        let ips = self.user_ips.read().await;
                        (
                            sessions
                                .get(&requested_username)
                                .is_some_and(|t| t == token),
                            ips.get(&requested_username)
                                .is_some_and(|ip| *ip == self.addr.ip()),
                        )
                    };

                    session_matches && ip_matches
                } else {
                    false
                };

                if can_reclaim {
                    // This is a valid reconnection - reclaim the ghost session
                    logger::log_success(&format!(
                        "User '{}' reclaiming ghost session from {} (same token and IP)",
                        requested_username, self.addr
                    ));

                    // Signal the old connection to disconnect silently
                    let _ = self
                        .server_commands
                        .send(ServerCommand::SessionTakeover(requested_username.clone()));

                    // The username is already in the set, so we just claim it for this connection
                    *username = Some(requested_username.clone());
                } else {
                    // Not a valid reconnection - rename the user
                    logger::log_warning(&format!(
                        "User '{}' already exists, renaming...",
                        requested_username
                    ));
                    let new_name = self.randomize_username(&requested_username);
                    if !clients.insert(new_name.clone()) {
                        logger::log_error(&format!(
                            "Failed to assign random username to '{}'",
                            requested_username
                        ));
                        return Err(UserConnectionError::JoinError);
                    }
                    logger::log_success(&format!(
                        "User '{}' renamed to '{}'",
                        requested_username, new_name
                    ));
                    let rename_message = ChatMessage::try_new(
                        MessageTypes::UserRename,
                        Some(new_name.clone().into_bytes()),
                    )
                    .map_err(|_| UserConnectionError::InvalidMessage)?;
                    tcp_handler
                        .send_message_chunked(rename_message)
                        .await
                        .map_err(UserConnectionError::IoError)?;
                    *username = Some(new_name.clone());

                    // Store session token for the new name
                    if let Some(token) = session_token {
                        let mut sessions = self.user_sessions.write().await;
                        sessions.insert(new_name, token);
                    }
                }
            } else {
                // Username is available - claim it
                clients.insert(requested_username.clone());
                *username = Some(requested_username.clone());

                // Store session token for this username
                if let Some(token) = session_token {
                    drop(clients); // Release clients lock before acquiring sessions lock
                    let mut sessions = self.user_sessions.write().await;
                    sessions.insert(requested_username.clone(), token);
                }
            }
        }

        if let Some(username) = &username {
            {
                let mut ips = self.user_ips.write().await;
                ips.insert(username.clone(), self.addr.ip());
            }

            let join_message =
                ChatMessage::try_new(MessageTypes::Join, Some(username.clone().into_bytes()))
                    .map_err(|_| UserConnectionError::InvalidMessage)?;
            self.tx
                .send((join_message, self.addr))
                .map_err(UserConnectionError::BroadcastError)?;
            logger::log_system(&format!("{} has joined the chat", username));
        }
        Ok(())
    }

    pub(super) async fn process_rename_request<S: AsyncRead + AsyncWrite + Unpin>(
        &self,
        new_name: Option<String>,
        tcp_handler: &mut StreamWrapper<'_, S>,
        username: &mut Option<String>,
    ) -> Result<(), UserConnectionError> {
        let new_name = new_name.ok_or(UserConnectionError::InvalidMessage)?;

        // Validate new username length
        if new_name.is_empty() || new_name.len() > MAX_USERNAME_LENGTH {
            logger::log_warning(&format!(
                "Invalid username length for rename from {}: {} chars",
                self.addr,
                new_name.len()
            ));
            let error_msg = ChatMessage::try_new(
                MessageTypes::Error,
                Some(b"Invalid username length (1-32 characters)".to_vec()),
            )
            .map_err(|_| UserConnectionError::InvalidMessage)?;
            tcp_handler
                .send_message_chunked(error_msg)
                .await
                .map_err(UserConnectionError::IoError)?;
            return Ok(());
        }

        // Validate username characters (alphanumeric, underscore, hyphen only)
        if !new_name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            logger::log_warning(&format!(
                "Invalid username characters for rename from {}: {}",
                self.addr, new_name
            ));
            let error_msg = ChatMessage::try_new(
                MessageTypes::Error,
                Some(
                    b"Invalid characters (only alphanumeric, underscore, hyphen allowed)".to_vec(),
                ),
            )
            .map_err(|_| UserConnectionError::InvalidMessage)?;
            tcp_handler
                .send_message_chunked(error_msg)
                .await
                .map_err(UserConnectionError::IoError)?;
            return Ok(());
        }

        let old_name = self.require_username(username, "rename")?;

        // Try to claim the new name
        {
            let mut clients = self.connected_clients.write().await;

            // Check if new name is already taken
            if clients.contains(&new_name) {
                drop(clients);
                let error_msg = ChatMessage::try_new(
                    MessageTypes::Error,
                    Some(format!("Username '{}' is already taken", new_name).into_bytes()),
                )
                .map_err(|_| UserConnectionError::InvalidMessage)?;
                tcp_handler
                    .send_message_chunked(error_msg)
                    .await
                    .map_err(UserConnectionError::IoError)?;
                return Ok(());
            }

            // Remove old name and add new name
            clients.remove(&old_name);
            clients.insert(new_name.clone());
        }

        {
            let mut ips = self.user_ips.write().await;
            if let Some(ip) = ips.remove(&old_name) {
                ips.insert(new_name.clone(), ip);
            }
        }

        // Update the username
        *username = Some(new_name.clone());

        logger::log_success(&format!("User '{}' renamed to '{}'", old_name, new_name));

        // Send UserRename message back to the client
        let rename_message = ChatMessage::try_new(
            MessageTypes::UserRename,
            Some(new_name.clone().into_bytes()),
        )
        .map_err(|_| UserConnectionError::InvalidMessage)?;
        tcp_handler
            .send_message_chunked(rename_message)
            .await
            .map_err(UserConnectionError::IoError)?;

        // Broadcast rename announcement to all clients
        let announcement = format!("{} is now known as {}", old_name, new_name);
        let broadcast_message =
            ChatMessage::try_new(MessageTypes::ChatMessage, Some(announcement.into_bytes()))
                .map_err(|_| UserConnectionError::InvalidMessage)?;
        self.tx
            .send((broadcast_message, self.addr))
            .map_err(UserConnectionError::BroadcastError)?;

        Ok(())
    }

    pub(super) async fn process_version_check<S: AsyncRead + AsyncWrite + Unpin>(
        &self,
        client_version: Option<String>,
        tcp_handler: &mut StreamWrapper<'_, S>,
    ) -> Result<(), UserConnectionError> {
        let client_version = client_version.ok_or(UserConnectionError::InvalidMessage)?;

        if !version::versions_compatible(&client_version, VERSION) {
            logger::log_warning(&format!(
                "Version mismatch from {}: client v{} != server v{}",
                self.addr, client_version, VERSION
            ));

            // Send version mismatch error with details
            let mismatch_content = format!(
                "{}|{}|{}",
                client_version,
                VERSION,
                version::GITHUB_README_URL
            );
            let mismatch_msg = ChatMessage::try_new(
                MessageTypes::VersionMismatch,
                Some(mismatch_content.into_bytes()),
            )
            .map_err(|_| UserConnectionError::InvalidMessage)?;
            tcp_handler
                .send_message_chunked(mismatch_msg)
                .await
                .map_err(UserConnectionError::IoError)?;

            return Err(UserConnectionError::VersionMismatch);
        }

        logger::log_info(&format!(
            "Version check passed for {}: v{}",
            self.addr, client_version
        ));
        Ok(())
    }
}
