use embedded_recruitment_task::message::EchoMessage;
use embedded_recruitment_task::message::{client_message, ServerMessage,AddRequest,AddResponse};
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
    let address = format!("{}:{}", ip, port);

    // Create a TCP connection to the server
    let mut stream = match TcpStream::connect(address) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to connect to server: {}", e);
            process::exit(1);
        }
    };

    // Set a timeout on the stream (optional)
    let timeout = Duration::from_secs(5);
    if let Err(e) = stream.set_read_timeout(Some(timeout)) {
        error!("Failed to set read timeout: {}", e);
        process::exit(1);
    }

    // Create the AddRequest message
    let mut add_request = AddRequest::default();
    add_request.a = 10;
    add_request.b = 25;

    // Encode the AddRequest to a byte buffer
    let mut buffer = Vec::new();
    if let Err(e) = add_request.encode(&mut buffer) {
        error!("Failed to encode AddRequest: {}", e);
        process::exit(1);
    }

    // Send the AddRequest to the server
    if let Err(e) = stream.write_all(&buffer) {
        error!("Failed to send message: {}", e);
        process::exit(1);
    }

    // Receive the response from the server
    info!("Receiving message from the server");
    let mut buffer = vec![0u8; 1024];
    let bytes_read = match stream.read(&mut buffer) {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Failed to read from the server: {}", e);
            process::exit(1);
        }
    };

    if bytes_read == 0 {
        info!("Server disconnected.");
        process::exit(1);
    }

    info!("Received {} bytes from the server", bytes_read);

    // Decode the received message
    match AddResponse::decode(&buffer[..bytes_read]) {
        Ok(add_response) => {
            info!("Received AddResponse from server: {:?}", add_response);
            // Handle the AddResponse here
            println!("AddResponse: {:?}", add_response);
        }
        Err(e) => {
            error!("Failed to decode AddResponse: {}", e);
            process::exit(1);
        }
    }
}
