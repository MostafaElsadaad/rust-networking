use log::{error, info, warn};
use prost::Message;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio::io;
use crate::message::{EchoMessage,AddRequest,AddResponse};
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
    
        loop {
            // Read data from the stream asynchronously
            let bytes_read = match self.stream.read(&mut buffer).await {
                Ok(n) if n > 0 => n, // Successfully read data
                Ok(_) => {
                    // No data was read (client closed connection)
                    info!("Client disconnected.");
                    return Ok(()); // Exit the loop and return
                }
                Err(e) => {
                    // Error occurred while reading
                    error!("Failed to read from client: {}", e);
                    return Err(e); // Return error
                }
            };
    
            // Try to decode AddRequest message
            if let Ok(add_request) = AddRequest::decode(&buffer[..bytes_read]) {
                // Perform the addition
                let sum = add_request.a + add_request.b;
    
                // Log the result
                info!("Adding {} + {} = {}", add_request.a, add_request.b, sum);
    
                // Create an AddResponse with the result
                let add_response = AddResponse {
                    result: sum,
                };
    
                // Encode the AddResponse message to a buffer
                let mut buffer = Vec::new();
                add_response.encode(&mut buffer);
    
                // Send the response asynchronously
                if let Err(e) = self.stream.write_all(&buffer).await {
                    error!("Failed to send response: {}", e);
                    return Err(e); // Return error if write fails
                }
    
                // Make sure all data is sent
                if let Err(e) = self.stream.flush().await {
                    error!("Failed to flush stream: {}", e);
                    return Err(e); // Return error if flush fails
                }
            }
            // Try to decode EchoMessage
            else if let Ok(echo_message) = EchoMessage::decode(&buffer[..bytes_read]) {
                // Process EchoMessage
                info!("Received EchoMessage: {}", echo_message.content);
                println!("Received EchoMessage: {}", echo_message.content);
    
                // Echo back the message asynchronously
                let payload = echo_message.encode_to_vec();
                if let Err(e) = self.stream.write_all(&payload).await {
                    error!("Failed to send response: {}", e);
                    return Err(e); // Return error if write fails
                }
    
                // Make sure all data is sent
                if let Err(e) = self.stream.flush().await {
                    error!("Failed to flush stream: {}", e);
                    return Err(e); // Return error if flush fails
                }
            }
            // Handle unexpected message types
            else {
                error!("Failed to decode message");
            }
        }
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
