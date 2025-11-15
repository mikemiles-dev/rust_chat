use chat_shared::network::TcpMessageHandler;
use std::io::{self, Write};
use std::net::{AddrParseError, SocketAddr};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

use chat_shared::message::{ChatMessage, ChatMessageError, MessageTypes};
use tokio::net::TcpStream;

struct ChatClient {
    connection: TcpStream,
    chat_name: String,
}

#[derive(Debug)]
pub enum ChatClientError {
    InvalidAddress(AddrParseError),
    IoError(io::Error),
    JoinError(String),
    TokioError(tokio::io::Error),
    ChatMessageError(ChatMessageError),
}

impl ChatClient {
    async fn new(server_addr: &str, name: String) -> Result<Self, ChatClientError> {
        let server_addr: SocketAddr = server_addr
            .parse()
            .map_err(ChatClientError::InvalidAddress)?;

        let stream = TcpStream::connect(server_addr)
            .await
            .map_err(ChatClientError::IoError)?;

        Ok(ChatClient {
            connection: stream,
            chat_name: name,
        })
    }

    async fn join_server(&mut self) -> Result<(), ChatClientError> {
        let chat_message =
            ChatMessage::try_new(MessageTypes::Join, Some(self.chat_name.as_bytes().to_vec()))
                .map_err(ChatClientError::ChatMessageError)?;
        self.send_message_chunked(chat_message)
            .await
            .map_err(ChatClientError::TokioError)?;
        Ok(())
    }

    async fn get_user_input() -> Option<String> {
        let stdin = tokio::io::stdin();
        let mut reader = tokio::io::BufReader::new(stdin);
        let mut input_line = String::new();

        match reader.read_line(&mut input_line).await {
            Ok(0) => {
                // EOF (e.g., Ctrl+D or stream closed)
                println!("**[Input]** EOF received. Exiting...");
            }
            Ok(_) => {
                let trimmed_input = input_line.trim();
                println!("**[Input]** You typed: {}", trimmed_input);

                if trimmed_input.eq_ignore_ascii_case("quit") {
                    println!("**[Input]** Quitting application.");
                    return None;
                }
                // IMPORTANT: Clear the buffer for the next read
                input_line.clear();
            }
            Err(e) => {
                eprintln!("Input error: {}", e);
            }
        }
        Some(input_line)
    }

    async fn run(&mut self) -> io::Result<()> {
        loop {
            // 3. Use tokio::select! to concurrently wait for either operation
            tokio::select! {
                // Branch 1: Receive
                result = self.read_message_chunked() => {
                    match result {
                        Ok(message) => {
                            println!("Received message: {:?}", message);
                        }
                        Err(chat_shared::network::TcpMessageHandlerError::IoError(e)) => {
                            eprintln!("IO error reading from server: {:?}", e);
                            return Err(e);
                        }
                        Err(chat_shared::network::TcpMessageHandlerError::Disconnect) => {
                            println!("Disconnected from server.");
                            return Ok(());
                        }
                    }
                }
                // Branch 2: User Input
                result = ChatClient::get_user_input() => {
                    if let Some(_input_line) = result {
                        //let trimmed_input = input_line.trim();
                        // if !trimmed_input.is_empty() {
                        //     self.udp_wrapper
                        //         .send_data(self.server_addr, trimmed_input.as_bytes().to_vec())
                        //         .await?;
                        // }
                    } else {
                        // User chose to quit
                        return Ok(());
                    }
                }
            }
        }
    }
}

impl TcpMessageHandler for ChatClient {
    fn get_stream(&mut self) -> &mut tokio::net::TcpStream {
        &mut self.connection
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let (chat_server, chat_name) = prompt_server_info()?;
    let mut client = ChatClient::new(&chat_server, chat_name)
        .await
        .unwrap_or_else(|e| {
            panic!(
                "Failed to create ChatClient for server {}: {:?}",
                chat_server, e
            )
        });

    client
        .join_server()
        .await
        .unwrap_or_else(|_| panic!("Could not connect to: {}", chat_server));

    client.run().await
}

fn prompt_server_info() -> io::Result<(String, String)> {
    let server_default = "127.0.0.1:8080";
    let name_default = "Guest";
    let mut chat_server = String::new();
    let mut chat_name = String::new();
    println!("Press Enter Chat Server (default: {}):", server_default);
    io::stdout().flush()?;
    io::stdin().read_line(&mut chat_server)?;
    let chat_server = chat_server.trim();
    println!("Press Enter Chat Name (default: {}):", name_default);
    io::stdout().flush()?;
    io::stdin().read_line(&mut chat_name)?;
    let chat_name = chat_name.trim();
    let chat_server = if chat_server.is_empty() {
        server_default.to_string()
    } else {
        chat_server.to_string()
    };
    let chat_name = if chat_name.is_empty() {
        name_default.to_string()
    } else {
        chat_name.to_string()
    };
    Ok((chat_server, chat_name))
}
