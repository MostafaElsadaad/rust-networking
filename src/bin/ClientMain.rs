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

fn main() {

    // Server details
    let ip = "127.0.0.1";
    let port = 5000;
    let timeout_ms = 5000; // Timeout in milliseconds

    // Create a new client
    let mut client = client::Client::new(ip, port, timeout_ms);

    // Connect to the server
    if let Err(e) = client.connect() {
        error!("Failed to connect to server: {}", e);
        process::exit(1);
    }

    // Create a message to send to the server
    let echo_message = EchoMessage {
        content: String::from("Hello, Server!, This is Mostafa!"), // Customize the message content
    };

    // Wrap the EchoMessage in the ClientMessage using the `Message` enum
    let message = client_message::Message::EchoMessage(echo_message);

    // Send the message to the server
    if let Err(e) = client.send(message) {
        error!("Failed to send message: {}", e);
        if let Err(disconnect_error) = client.disconnect() {
            error!("Failed to disconnect after error: {}", disconnect_error);
        }
        process::exit(1);
    }

    // Receive the server's response
    match client.receive() {
        Ok(response) => {
            info!("Received response from server: {:?}", response);
        }
        Err(e) => {
            error!("Failed to receive response: {}", e);
        }
    }

    // Disconnect from the server
    if let Err(e) = client.disconnect() {
        error!("Failed to disconnect: {}", e);
    }
}
