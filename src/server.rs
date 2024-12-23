use log::{error, info, warn};
use prost::Message;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio::io;
use crate::message::EchoMessage;
use tokio::time;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::time::Duration;

// Client struct for handling individual client connections.
struct Client {
    stream: TcpStream,
}

impl Client {
    // Create a new client instance.
    pub fn new(stream: TcpStream) -> Self {
        Client { stream }
    }

    // Asynchronous method to handle the client's communication.
    pub async fn handle(&mut self) -> io::Result<()> {
        let mut buffer = [0; 512];

        // Read data from the client asynchronously.
        let bytes_read = self.stream.read(&mut buffer).await?;
        if bytes_read == 0 {
            info!("Client disconnected.");
            return Ok(());
        }

        if let Ok(message) = EchoMessage::decode(&buffer[..bytes_read]) {
            info!("Received: {}", message.content);
            println!("Received: {}", message.content);

            // Echo back the message asynchronously.
            let payload = message.encode_to_vec();
            self.stream.write_all(&payload).await?;
            self.stream.flush().await?;
        } else {
            error!("Failed to decode message");
        }

        Ok(())
    }
}

// Server struct for managing the listening and handling of incoming connections.
pub struct Server {
    listener: TcpListener,
    is_running: Arc<Mutex<bool>>, // Shared state for running status
}

impl Server {
    // Create a new server instance.
    pub async fn new(addr: &str) -> io::Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        let is_running = Arc::new(Mutex::new(false));
        Ok(Server {
            listener,
            is_running,
        })
    }

    // Asynchronous method to run the server.
    pub async fn run(&self) -> io::Result<()> {
        {
            let mut running = self.is_running.lock().unwrap();
            *running = true;
        }
        info!("Server is running on {}", self.listener.local_addr()?);

        loop {
            match self.listener.accept().await {
                Ok((stream, addr)) => {
                    info!("New client connected: {}", addr);

                    // Handle the client request asynchronously.
                    let mut client = Client::new(stream);
                    tokio::spawn(async move {
                        if let Err(e) = client.handle().await {
                            error!("Error handling client {}: {}", addr, e);
                        }
                    });
                }
                Err(e) => {
                    error!("Error accepting connection: {}", e);
                }
            }

            // Optional: Sleep briefly to reduce CPU usage (non-blocking in Tokio).
            time::sleep(Duration::from_millis(100)).await;
        }
    }

    // Stops the server by setting the `is_running` flag to `false`.
    pub async fn stop(&self) {
        let mut running = self.is_running.lock().unwrap();
        if *running {
            *running = false;
            info!("Shutdown signal sent.");
        } else {
            warn!("Server was already stopped or not running.");
        }
    }
}
