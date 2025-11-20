mod error;
mod handlers;
mod rate_limiting;

pub use error::UserConnectionError;
use handlers::MessageHandlers;
use rate_limiting::{RateLimiter, RATE_LIMIT_MESSAGES, RATE_LIMIT_WINDOW};

use crate::ServerCommand;
use shared::logger;
use shared::message::{ChatMessage, MessageTypes};
use shared::network::{TcpMessageHandler, TcpMessageHandlerError};
use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::{RwLock, broadcast};

pub struct UserConnection {
    socket: TcpStream,
    addr: SocketAddr,
    tx: broadcast::Sender<(ChatMessage, SocketAddr)>,
    server_commands: broadcast::Sender<ServerCommand>,
    connected_clients: Arc<RwLock<HashSet<String>>>,
    chat_name: Option<String>,
    rate_limiter: RateLimiter,
}

impl TcpMessageHandler for UserConnection {
    fn get_stream(&mut self) -> &mut tokio::net::TcpStream {
        &mut self.socket
    }
}

impl UserConnection {
    pub fn new(
        socket: TcpStream,
        addr: SocketAddr,
        tx: broadcast::Sender<(ChatMessage, SocketAddr)>,
        server_commands: broadcast::Sender<ServerCommand>,
        connected_clients: Arc<RwLock<HashSet<String>>>,
    ) -> Self {
        UserConnection {
            socket,
            addr,
            tx,
            server_commands,
            connected_clients,
            chat_name: None,
            rate_limiter: RateLimiter::new(RATE_LIMIT_MESSAGES, RATE_LIMIT_WINDOW),
        }
    }

    pub async fn handle(&mut self) -> Result<(), UserConnectionError> {
        logger::log_info(&format!("New client connected: {}", self.addr));

        let mut rx = self.tx.subscribe();
        let mut cmd_rx = self.server_commands.subscribe();

        loop {
            tokio::select! {
                // Branch 1: Receive from client
                result = self.read_message_chunked() => {
                    match result {
                        Ok(msg) => {
                            if let Err(e) = self.process_message(msg).await {
                                logger::log_error(&format!("Error handling message from {}: {:?}", self.addr, e));
                            }
                        }
                        Err(TcpMessageHandlerError::IoError(e)) => {
                            logger::log_error(&format!("IO error reading from {}: {:?}", self.addr, e));
                            break;
                        }
                        Err(TcpMessageHandlerError::Disconnect) => {
                            logger::log_warning(&format!("Client {} disconnected", self.addr));
                            break;
                        }
                    };
                }
                // Branch 2: Broadcast to other clients
                result = rx.recv() => {
                    match result {
                        Ok((msg, _src_addr)) => {
                            if let Err(e) = self.send_message_chunked(msg).await {
                                logger::log_warning(&format!("Failed to send message to {}: {:?}", self.addr, e));
                                // Client likely disconnected, break to clean up
                                break;
                            }
                        }
                        Err(e) => {
                            logger::log_error(&format!("Broadcast receive error for {}: {:?}", self.addr, e));
                            break;
                        }
                    }
                }
                // Branch 3: Server commands (kick, etc.)
                result = cmd_rx.recv() => {
                    match result {
                        Ok(ServerCommand::Kick(username)) => {
                            if let Some(chat_name) = &self.chat_name
                                && chat_name == &username {
                                logger::log_info(&format!("User {} kicked by server", chat_name));
                                // Send error message to client before disconnecting
                                if let Ok(kick_msg) = ChatMessage::try_new(
                                    MessageTypes::Error,
                                    Some("You have been kicked by the server".as_bytes().to_vec())
                                ) {
                                    let _ = self.send_message_chunked(kick_msg).await;
                                }
                                break;
                            }
                        }
                        Err(_) => {
                            // Channel closed, ignore
                        }
                    }
                }
            }
        }

        // Cleanup on disconnect
        if let Some(chat_name) = &self.chat_name {
            let mut clients = self.connected_clients.write().await;
            clients.remove(chat_name);
            if let Ok(leave_message) =
                ChatMessage::try_new(MessageTypes::Leave, Some(chat_name.clone().into_bytes()))
            {
                let _ = self.tx.send((leave_message, self.addr));
            }
            logger::log_system(&format!("{} has left the chat", chat_name));
        }

        Ok(())
    }

    async fn process_message(&mut self, message: ChatMessage) -> Result<(), UserConnectionError> {
        let handlers = MessageHandlers {
            addr: self.addr,
            tx: &self.tx,
            connected_clients: &self.connected_clients,
        };

        handlers
            .process_message(
                message,
                &mut self.rate_limiter,
                &mut self.socket,
                &mut self.chat_name,
            )
            .await
    }
}
