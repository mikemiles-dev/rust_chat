use std::io;
use tokio::net::UdpSocket;

pub struct ChatServer {
    socket: UdpSocket,
}

impl ChatServer {
    async fn new(bind_addr: &str) -> io::Result<Self> {
        let socket = UdpSocket::bind(bind_addr).await?;
        Ok(ChatServer { socket })
    }

    async fn run(&mut self) -> io::Result<()> {
        let mut buf = [0; 1024]; // Buffer to hold incoming data

        loop {
            let (len, addr) = self.socket.recv_from(&mut buf).await?;

            println!(
                "Received {} bytes from: {}: {}",
                len,
                addr,
                String::from_utf8_lossy(&buf[..len])
            );

            // Optional: Echo the data back to the sender
            let sent_len = self.socket.send_to(&buf[..len], addr).await?;
            println!("Sent {} bytes back to: {}", sent_len, addr);
        }
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    // 1. Bind the socket to a local address (e.g., all interfaces on port 8080)
    let mut server = ChatServer::new("0.0.0.0:8080").await?;
    println!("UDP Listener bound to 0.0.0.0:8080");

    server.run().await
}
