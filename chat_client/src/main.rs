use std::io::{self, Write};
use tokio::net::UdpSocket;

use chat_shared::{Message, MessageTypes};

struct ChatClient {
    socket: UdpSocket,
    name: String,
    server_addr: String,
    sent_messages: Vec<SentMessage>,
    message_id_counter: u8,
}

pub struct SentMessage {
    pub id: u8,
    pub length: usize,
}

#[derive(Debug)]
pub enum ChatClientError {
    IoError(io::Error),
    JoinError(String),
}

impl ChatClient {
    async fn new(server_addr: &str, name: String) -> io::Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        Ok(ChatClient {
            socket,
            name,
            server_addr: server_addr.to_string(),
            sent_messages: vec![],
            message_id_counter: 0,
        })
    }

    async fn increment_message_id(&mut self) {
        self.message_id_counter = self.message_id_counter.checked_add(1).unwrap_or(0);
    }

    async fn send_message(
        &mut self,
        message_type: MessageTypes,
        content: &str,
    ) -> Result<(), ChatClientError> {
        self.increment_message_id().await;
        let content_owned = content.to_string().into_bytes();
        let message = Message::new(message_type, &content_owned, self.message_id_counter);
        let message_bytes: Vec<u8> = message.clone().into();
        let sent_length = self
            .socket
            .send_to(&message_bytes, &self.server_addr)
            .await
            .map_err(|e| ChatClientError::IoError(e))?;
        println!(
            "Sent message of {} bytes to {}: {}",
            sent_length,
            self.server_addr,
            message_bytes.len()
        );
        if sent_length != message_bytes.len() {
            Err(ChatClientError::JoinError(
                format!(
                    "Warning: Sent length {} does not match message length {}",
                    sent_length,
                    message_bytes.len(),
                )
                .to_string(),
            ))
        } else {
            self.sent_messages.push(SentMessage {
                id: message.id,
                length: message.length,
            });
            Ok(())
        }
    }

    async fn join_server(&mut self) -> Result<(), ChatClientError> {
        let message = format!("{}", self.name);
        self.send_message(MessageTypes::Join, &message).await
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let (chat_server, chat_name) = prompt_server_info()?;
    let mut client = ChatClient::new(&chat_server, chat_name).await?;
    let mut user_input = String::new();

    client
        .join_server()
        .await
        .expect(format!("Could not connect to: {}", chat_server).as_str());

    loop {
        user_input.clear();
        println!("Press Enter to send a message...");
        io::stdin().read_line(&mut user_input)?;
    }

    Ok(())
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
