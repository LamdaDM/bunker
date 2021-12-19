# Description
**Bunker** is a library for building multi-threaded TCP servers in Rust. It handles the usual bootstrap required for server execution and handling of the request/response cycle. 

You may set the configure the server through `bunker::server::Builder`. The configurable options are:
- Port: The port which the socket will bind to.
- Addr: The address which the socket will bind to.
- Threads: The number of threads assigned to the threadpool.
- Read buffer size: The maximum number of bytes read into the buffer.
- End-connection message: The string received from the controller that signals Bunker to end the connection with the client.
- Parse options: Informs Bunker how it should split the incoming data for the path and the message.
- Debug: Determines state of the debugger (on/off).
- Route map: Used for matching the request's path to any controller registered to the route map.

## Example

``` rust
# main.rs

fn main() {
    bunker::server::Builder::new()
        .port(1505)
        .addr([127, 0, 0, 1]) // This sets Bunker to bind on 127.0.0.1:1505
        .build()
        .run() // Server starts
} 
```