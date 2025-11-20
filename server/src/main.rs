use shared::logger;
use shared::message::ChatMessage;
use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::{env, io};
use tokio::net::TcpListener;
use tokio::sync::{RwLock, broadcast};

mod completer;
mod input;
mod readline_helper;
mod user_connection;
use input::ServerUserInput;
use user_connection::UserConnection;

#[derive(Debug, Clone)]
pub enum ServerCommand {
    Kick(String),
}

pub struct ChatServer {
    listener: TcpListener,
    broadcaster: broadcast::Sender<(ChatMessage, SocketAddr)>,
    server_commands: broadcast::Sender<ServerCommand>,
    connected_clients: Arc<RwLock<HashSet<String>>>,
    max_clients: usize,
    active_connections: Arc<AtomicUsize>,
}

impl ChatServer {
    async fn new(bind_addr: &str, max_clients: usize) -> io::Result<Self> {
        let (tx, _rx) = broadcast::channel(max_clients * 16); // Allow message buffering
        let (cmd_tx, _cmd_rx) = broadcast::channel(100); // Server commands channel
        let listener = TcpListener::bind(bind_addr).await?;

        Ok(ChatServer {
            listener,
            broadcaster: tx,
            server_commands: cmd_tx,
            connected_clients: Arc::new(RwLock::new(HashSet::new())),
            max_clients,
            active_connections: Arc::new(AtomicUsize::new(0)),
        })
    }

    async fn run(&mut self) -> io::Result<()> {
        // Spawn readline handler in a blocking thread (if TTY available)
        let mut readline_rx = readline_helper::spawn_readline_handler();

        if readline_rx.is_none() {
            logger::log_info("Running in non-interactive mode (no TTY)");
            logger::log_info("Server commands disabled - use docker exec for admin tasks");
        }

        loop {
            tokio::select! {
                // Handle incoming client connections
                result = self.listener.accept() => {
                    match result {
                        Ok((socket, addr)) => {
                            // Check connection limit
                            let current_connections = self.active_connections.load(Ordering::Relaxed);
                            if current_connections >= self.max_clients {
                                logger::log_warning(&format!(
                                    "Connection limit reached ({}/{}), rejecting connection from {}",
                                    current_connections, self.max_clients, addr
                                ));
                                continue;
                            }

                            // Increment connection count
                            self.active_connections.fetch_add(1, Ordering::Relaxed);

                            let tx_clone = self.broadcaster.clone();
                            let cmd_tx_clone = self.server_commands.clone();
                            let active_connections_clone = self.active_connections.clone();

                            let mut client_connection =
                                UserConnection::new(socket, addr, tx_clone, cmd_tx_clone, self.connected_clients.clone());

                            tokio::spawn(async move {
                                if let Err(e) = client_connection.handle().await {
                                    logger::log_error(&format!("Error handling client {}: {:?}", addr, e));
                                }

                                // Decrement connection count when done
                                active_connections_clone.fetch_sub(1, Ordering::Relaxed);
                                logger::log_info(&format!("Connection from {} closed", addr));
                            });
                        }
                        Err(e) => {
                            logger::log_error(&format!("Failed to accept connection: {:?}", e));
                        }
                    }
                }
                // Handle server commands from readline (only if TTY available)
                Some(line) = async {
                    match &mut readline_rx {
                        Some(rx) => rx.recv().await,
                        None => std::future::pending().await, // Never resolves if no TTY
                    }
                } => {
                    match line {
                        Some(input_line) => {
                            match ServerUserInput::try_from(input_line.as_str()) {
                                Ok(ServerUserInput::Quit) => {
                                    logger::log_info("Server shutting down...");
                                    return Ok(());
                                }
                                Ok(ServerUserInput::ListUsers) => {
                                    self.handle_list_users().await;
                                }
                                Ok(ServerUserInput::Kick(username)) => {
                                    self.handle_kick(username).await;
                                }
                                Ok(ServerUserInput::Help) => {
                                    self.handle_help();
                                }
                                Err(_) => {
                                    logger::log_error("Invalid command. Type /help for available commands.");
                                }
                            }
                        }
                        None => {
                            // EOF from readline
                            logger::log_info("Server shutting down...");
                            return Ok(());
                        }
                    }
                }
            }
        }
    }

    async fn handle_list_users(&self) {
        let clients = self.connected_clients.read().await;
        let count = clients.len();
        if count == 0 {
            logger::log_info("No users currently connected.");
        } else {
            logger::log_info(&format!("Connected users ({}):", count));
            for user in clients.iter() {
                logger::log_info(&format!("  - {}", user));
            }
        }
    }

    async fn handle_kick(&self, username: String) {
        let clients = self.connected_clients.read().await;
        if clients.contains(&username) {
            drop(clients);
            // Send kick command to all connections - the matching one will disconnect
            if self.server_commands.send(ServerCommand::Kick(username.clone())).is_ok() {
                logger::log_warning(&format!("Kicking user: {}", username));
            }
        } else {
            logger::log_error(&format!("User '{}' not found", username));
        }
    }

    fn handle_help(&self) {
        logger::log_info("Available server commands:");
        logger::log_info("  /list           - List all connected users");
        logger::log_info("  /kick <user>    - Kick a user from the server");
        logger::log_info("  /help           - Show this help message");
        logger::log_info("  /quit           - Shutdown the server");
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    const CHAT_SERVER_ADDR_ENV_VAR: &str = "CHAT_SERVER_ADDR";
    const CHAT_SERVER_MAX_CLIENTS_ENV_VAR: &str = "CHAT_SERVER_MAX_CLIENTS";
    let chat_server_addr = env::var(CHAT_SERVER_ADDR_ENV_VAR).unwrap_or("0.0.0.0:8080".to_string());
    let max_clients = env::var(CHAT_SERVER_MAX_CLIENTS_ENV_VAR)
        .unwrap_or("100".to_string())
        .parse::<usize>()
        .unwrap_or(100);
    let mut server = ChatServer::new(&chat_server_addr, max_clients).await?;
    logger::log_success(&format!("Chat Server started at {}", chat_server_addr));
    logger::log_info(&format!(
        "To change address, set {} environment variable",
        CHAT_SERVER_ADDR_ENV_VAR
    ));
    logger::log_info(&format!(
        "To change max clients, set {} environment variable",
        CHAT_SERVER_MAX_CLIENTS_ENV_VAR
    ));
    logger::log_info("Server commands: /help, /list, /quit");

    server.run().await
}
