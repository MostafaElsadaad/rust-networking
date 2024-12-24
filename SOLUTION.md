# Solution

## **Note** 
I had no prior experience in rust but i have experience in Networking and backend development which kinda gave me a hand in doing this task. 
Estimated time to complete: 8 hours , mostly studying rust 😅

## How To Evaluate my work
First of all I've made a playground to test communication and debug freely by making three scripts
- bin\ServerMain.rs
- bin\ClientMain.rs
- bin\ClientMainAddRequest.rs

you can run these to see if everything is working correctly. Also, you can run the **tests** but run each test indvidualy to avoid shared state problems (but don't run it with one thread)

you can run each one indvidualy using the following commands
- cargo test test_client_connection
- cargo test test_client_echo_message
- cargo test test_multiple_echo_messages
- cargo test test_multiple_clients

The **test_client_add_request** Test didn't pass due to problems in encoding or decoding the message. but i think the problem could be server side (didn't have enough time to debug it though). You can test it buy running servermain.rs and ClientMainAddRequest.rs together each one in a terminal
- cargo run --bin ServerMain
- cargo run --bin ClientMainAddRequest

## Design Flaws
1. Single-Threaded Design
    - the server cannot accept new connections or handle other clients concurrently.
2. Blocking Operations in Non Blocking Mode
    - Even though the listener is set to non blocking mode, the client handling logic (stream.read and stream.write) is blocking

``` rust
let bytes_read = self.stream.read(&mut buffer)?;
self.stream.write_all(&payload)?;
```
These operations will block until data is available or the write buffer is free, which can lead to delays when handling multiple clients or multiple messages, and this is the reason of why some tests fail.

3. No Graceful Handling of WouldBlock for Client Reads/Writes
4. Shared State Management
    - **Data Overwrites** : The same buffer ([0; 512]) is reused for all clients without ensuring thread safety or client isolation.
    - **Race Conditions** :  concurrent connections and disconnections or communication may result in unexpected behaviour as partial reads/writes


## My Changes
- Introduced Multithreading
    - Updated server to spawn a thread for each connection so each client is handled in a separate thread
- Switched to Asynchronous Networking
    - to induce concurrency i used tokio to let the server handle requests conccurrently for better scalability
- Improved Buffer Management
    - Used buffer per client to avoid data being overwritting or shared across threads


## Design Concept: Message Queuing and Event Bus

If there had been more time to implement the full solution, the system would have incorporated a Message Queuing and Event Bus architecture. This approach would enhance communication between the server and clients by enabling more flexible, scalable, and asynchronous message handling.

The concept would draw inspiration from widely used patterns or message brokers such as ROS (Robot Operating System) and RabbitMQ, which allow clients to subscribe to specific message types and handle them in an efficient and decoupled manner.