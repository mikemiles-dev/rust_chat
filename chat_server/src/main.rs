use std::io;
use tokio::net::UdpSocket;

#[tokio::main]
async fn main() -> io::Result<()> {
    // 1. Bind the socket to a local address (e.g., all interfaces on port 8080)
    let sock = UdpSocket::bind("0.0.0.0:8080").await?;
    println!("UDP Listener bound to 0.0.0.0:8080");

    let mut buf = [0; 1024]; // Buffer to hold incoming data

    loop {
        // 2. Wait for a datagram
        // recv_from returns the number of bytes read (len) and the sender's address (addr)
        let (len, addr) = sock.recv_from(&mut buf).await?;

        println!(
            "Received {} bytes from: {}: {}",
            len,
            addr,
            String::from_utf8_lossy(&buf[..len])
        );

        // Optional: Echo the data back to the sender
        let sent_len = sock.send_to(&buf[..len], addr).await?;
        println!("Sent {} bytes back to: {}", sent_len, addr);
    }
}
