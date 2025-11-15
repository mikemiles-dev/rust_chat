use chat_shared::network::TcpMessageHandler;
use std::net::SocketAddr;
use std::{env, io};
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::broadcast;

use chat_shared::message::ChatMessage;

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
    rx: broadcast::Receiver<(ChatMessage, SocketAddr)>,
}

impl TcpMessageHandler for NewConnection {
    fn get_stream(&mut self) -> &mut tokio::net::TcpStream {
        &mut self.socket
    }
}

impl NewConnection {
    async fn handle(&mut self) -> Result<(), io::Error> {
        println!("New client connected: {}", self.addr);

        // Initial broadcast to all existing clients
        // let welcome_msg = format!(">>> {} has joined the chat.", addr);
        // let chat_msg = ChatMessage::try_new(MessageTypes::Join, Some(welcome_msg));
        // tx.send((chat_msg, addr)).ok();

        // Loop continuously, concurrently waiting for either:
        // 1) A message from this client's socket (socket_read)
        // 2) A message broadcast from another client (channel_recv)
        loop {
            tokio::select! {
                result = self.read_message_chunked() => {
                    let message = result?;
                    println!("Received {:?} from: {}", message, self.addr);
                    // Client disconnected or closed the connection
                }
                // result = rx.recv() => {
                //     // If a message is received from the channel...
                //     let msg = match result {
                //         Ok(m) => m,
                //         // Handle lagging: If the receiver falls too far behind, skip the message.
                //         Err(broadcast::error::RecvError::Lagged(_)) => continue,
                //         // If the sender has been dropped, this task can exit (shouldn't happen here).
                //         Err(broadcast::error::RecvError::Closed) => {
                //             eprintln!("Broadcast channel closed.");
                //             break;
                //         },
                //     };

                //     // Write the received message to this client's socket
                //     writer.write_all(b"\n").await?;
                // }
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
            let (mut socket, addr) = self.listener.accept().await?;
            let tx_clone = self.broadcaster.clone();
            let mut rx = self.broadcaster.subscribe(); // Get a receiver for this client

            let mut client_connection = NewConnection {
                socket,
                addr,
                tx: tx_clone,
                rx,
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
