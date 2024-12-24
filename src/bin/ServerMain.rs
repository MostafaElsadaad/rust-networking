use embedded_recruitment_task::server::Server;
use log::{info, error};
use std::process;

#[tokio::main]  // Set up Tokio runtime
async fn main() {
    env_logger::init(); // Initialize logging
    
    let addr = "127.0.0.1:5000"; // Server address
    info!("Starting server at {}", addr);

    // Create the server asynchronously
    let server = match Server::new(addr).await {
        Ok(server) => server,
        Err(e) => {
            error!("Failed to create server: {}", e);
            process::exit(1);
        }
    };

    // Run the server asynchronously
    if let Err(e) = server.run().await {
        error!("Server failed: {}", e);
        process::exit(1);
    }
}