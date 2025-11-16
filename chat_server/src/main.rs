use chat_shared::network::TcpMessageHandler;
use std::net::SocketAddr;
use std::{env, io};
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::broadcast;

use chat_shared::message::{ChatMessage, MessageTypes};

pub struct ConnectedClient {
    pub addr: String,
    pub name: String,
}

pub struct ChatServer {
    listener: TcpListener,
    broadcaster: broadcast::Sender<(ChatMessage, SocketAddr)>,
}

pub struct NewConnection {
    socket: TcpStream,
    addr: SocketAddr,
    tx: broadcast::Sender<(ChatMessage, SocketAddr)>,
}

impl TcpMessageHandler for NewConnection {
    fn get_stream(&mut self) -> &mut tokio::net::TcpStream {
        &mut self.socket
    }
}

impl NewConnection {
    async fn handle(&mut self) -> Result<(), io::Error> {
        println!("New client connected: {}", self.addr);

        let mut rx = self.tx.subscribe();

        // Initial broadcast to all existing clients
        let welcome_msg = format!(">>> {} has joined the chat.", self.addr)
            .as_bytes()
            .to_vec();
        let chat_msg =
            ChatMessage::try_new(MessageTypes::Join, Some(welcome_msg)).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Failed to create join message: {:?}", e),
                )
            })?;
        self.tx.send((chat_msg, self.addr)).ok();

        loop {
            tokio::select! {
                result = self.read_message_chunked() => {
                    let message = match result {
                        Ok(msg) => msg,
                        Err(chat_shared::network::TcpMessageHandlerError::IoError(e)) => {
                            eprintln!("IO error reading from {}: {:?}", self.addr, e);
                            break;
                        }
                        Err(chat_shared::network::TcpMessageHandlerError::Disconnect) => {
                            println!("Client {} disconnected.", self.addr);
                            break;
                        }
                    };
                    println!("Received {:?} from: {}", message, self.addr);
                    // Client disconnected or closed the connection
                }
                result = rx.recv() => {
                    match result {
                        Ok((msg, src_addr)) => {
                            // Avoid sending the message back to the sender
                            if src_addr != self.addr {
                                self.send_message_chunked(msg).await?;
                            }
                        }
                        Err(e) => {
                            eprintln!("Broadcast receive error for {}: {:?}", self.addr, e);
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl ChatServer {
    async fn new(bind_addr: &str) -> io::Result<Self> {
        let (tx, _rx) = broadcast::channel(100); // 100 is the capacity
        let listener = TcpListener::bind(bind_addr).await?;

        Ok(ChatServer {
            listener,
            broadcaster: tx,
        })
    }

    // async fn process_message(&mut self, message: ChatMessage, src_addr: SocketAddr) {
    //     match message.msg_type {
    //         MessageTypes::Join => {
    //             let content = message.get_content().unwrap_or_default();
    //             println!("**[Join]** {} has joined the chat.", content);
    //         }
    //         MessageTypes::Leave => {
    //             let content = message.get_content().unwrap_or_default();
    //             println!("**[Leave]** {} has left the chat.", content);
    //         }
    //         MessageTypes::ChatMessage => {
    //             let content = message.get_content().unwrap_or_default();
    //             println!("**[Message]** {} says: {}", src_addr, content);
    //         }
    //         MessageTypes::UserRename => {
    //             let content = message.get_content().unwrap_or_default();
    //             println!(
    //                 "**[Rename]** {} has changed their name to {}.",
    //                 src_addr, content
    //             );
    //         }
    //         _ => (),
    //     }
    // }

    async fn run(&mut self) -> io::Result<()> {
        loop {
            let (socket, addr) = self.listener.accept().await?;
            let tx_clone = self.broadcaster.clone();

            let mut client_connection = NewConnection {
                socket,
                addr,
                tx: tx_clone,
            };

            // Spawn a task to handle the client
            tokio::spawn(async move {
                if let Err(e) = client_connection.handle().await {
                    eprintln!("Error handling client {}: {:?}", addr, e);
                }
            });
        }
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    const CHAT_SERVER_ADDR_ENV_VAR: &str = "CHAT_SERVER_ADDR";
    let chat_server_addr = env::var(CHAT_SERVER_ADDR_ENV_VAR).unwrap_or("0.0.0.0:8080".to_string());
    let mut server = ChatServer::new(&chat_server_addr).await?;
    println!("Chat Server Started at {}", chat_server_addr);
    println!(
        "To change the address, set the {} environment variable to change.",
        CHAT_SERVER_ADDR_ENV_VAR
    );

    server.run().await
}
