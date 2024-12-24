use embedded_recruitment_task::message::EchoMessage;
use embedded_recruitment_task::message::{client_message, ServerMessage};
use log::{error, info, warn};
use prost::Message;
use std::{
    io::{self, ErrorKind, Read, Write}, net::{TcpListener, TcpStream}, process, sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    }, thread, time::Duration
};

#[path = "../../tests/client.rs"]
mod client;

#[tokio::main]
async fn main() {
    // Server details
    let ip = "127.0.0.1";
    let port = 5000;
    let timeout_ms = 5000; // Timeout in milliseconds

    // Number of clients to simulate
    let client_count = 10;

    let mut tasks = vec![];

    for client_id in 1..=client_count {
        let client_task = tokio::spawn(async move {
            // Create a new client for each task
            let mut client = client::Client::new(ip, port, timeout_ms);

            // Connect to the server
            if let Err(e) = client.connect() {
                error!("Client {}: Failed to connect to server: {}", client_id, e);
                return;
            }

            let mut message_count = 1;

            // Loop to send multiple messages
            loop {
                // Create a message to send to the server
                let echo_message = EchoMessage {
                    content: format!("Client #{}: Hello, Server! This is Mostafa! Message #{}", client_id, message_count),
                };

                // Wrap the EchoMessage in the ClientMessage using the `Message` enum
                let message = client_message::Message::EchoMessage(echo_message);

                // Send the message to the server
                if let Err(e) = client.send(message) {
                    error!("Client {}: Failed to send message: {}", client_id, e);
                    if let Err(disconnect_error) = client.disconnect() {
                        error!("Client {}: Failed to disconnect after error: {}", client_id, disconnect_error);
                    }
                    return;
                }

                // Receive the server's response
                match client.receive() {
                    Ok(response) => {
                        info!("Client {}: Received response from server: {:?}", client_id, response);
                    }
                    Err(e) => {
                        error!("Client {}: Failed to receive response: {}", client_id, e);
                    }
                }

                // Increment the message count
                message_count += 1;

                // Add a condition to break the loop (for example, send 5 messages and then stop)
                if message_count > 5 {
                    break;
                }

                // Optional: Add a delay between messages
                tokio::time::sleep(Duration::from_secs(1)).await;
            }

            // Disconnect from the server
            if let Err(e) = client.disconnect() {
                error!("Client {}: Failed to disconnect: {}", client_id, e);
            }
        });

        tasks.push(client_task);
    }

    // Await all the client tasks to complete
    for task in tasks {
        if let Err(e) = task.await {
            error!("A client task failed: {}", e);
        }
    }
}