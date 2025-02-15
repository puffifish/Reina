// File: src/networking/network.rs
//! Minimal Networking Module for Reina Phase 1.
//!
//! This module simulates basic P2P networking using TCP. It provides a NetworkNode
//! that listens on a specified port, a function to send messages to peers, and a simple
//! connection handler that logs incoming messages. Future versions will expand these
//! capabilities for block propagation and consensus. 

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

/// A network node that listens for incoming TCP connections.
pub struct NetworkNode {
    /// The TCP listener bound to a port.
    listener: TcpListener,
}

impl NetworkNode {
    /// Creates a new NetworkNode listening on the specified port.
    ///
    /// # Arguments
    ///
    /// * `port` - The port number to bind the listener.
    pub fn new(port: u16) -> std::io::Result<Self> {
        let addr = format!("0.0.0.0:{}", port);
        let listener = TcpListener::bind(addr)?;
        Ok(Self { listener })
    }

    /// Runs the network node, accepting and handling incoming connections.
    ///
    /// For each connection, a new thread is spawned to handle messages.
    pub fn run(&self) {
        println!("NetworkNode listening on {}", self.listener.local_addr().unwrap());
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    thread::spawn(move || {
                        if let Err(e) = handle_connection(stream) {
                            eprintln!("Error handling connection: {}", e);
                        }
                    });
                }
                Err(e) => eprintln!("Connection failed: {}", e),
            }
        }
    }

    /// Sends a message to a peer at the given address.
    ///
    /// # Arguments
    ///
    /// * `peer_addr` - The peer's address (e.g., "127.0.0.1:8000").
    /// * `message` - The message to send.
    ///
    /// # Returns
    ///
    /// Ok(()) on success; otherwise, an error.
    pub fn send_message(peer_addr: &str, message: &str) -> std::io::Result<()> {
        let mut stream = TcpStream::connect(peer_addr)?;
        stream.write_all(message.as_bytes())?;
        Ok(())
    }
}

/// Handles an incoming connection by reading messages and logging them.
///
/// Returns Ok(()) when the connection is closed or an error occurs.
fn handle_connection(mut stream: TcpStream) -> std::io::Result<()> {
    let mut buffer = [0u8; 512];
    loop {
        let bytes_read = stream.read(&mut buffer)?;
        if bytes_read == 0 {
            break; // Connection closed.
        }
        let msg = String::from_utf8_lossy(&buffer[..bytes_read]);
        println!("Received message: {}", msg);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_network_node_send_receive() {
        // Start a listener on an available port.
        let node = NetworkNode::new(0).expect("Failed to bind listener");
        let addr = node.listener.local_addr().unwrap();

        // Spawn the node in a separate thread.
        thread::spawn(move || {
            node.run();
        });

        // Allow the listener to initialize.
        thread::sleep(Duration::from_millis(100));

        // Send a dummy message.
        let test_msg = "Test block data from Reina";
        NetworkNode::send_message(&addr.to_string(), test_msg)
            .expect("Failed to send message");

        // Connect to the node to verify connection; our handler prints received messages.
        // Here we simply check that connecting does not error.
        let mut stream = TcpStream::connect(addr).expect("Failed to connect to self");
        stream.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
        let mut buf = [0u8; 512];
        let _ = stream.read(&mut buf).unwrap_or(0);
    }
}