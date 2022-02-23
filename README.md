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
- Debug Formatter: Registers custom formatters for debugging, otherwise uses a default formatter.
- Route map: Used for matching the request's path to any controller registered to the route map.
- Max response length: Sets the maximum length of the response given from a controller. Also used for knowing how many characters long the message length prefix should be.
- Response on error: Response sent to the client in the event of an internal error occurring.

## Example

``` rust
# main.rs

fn main() {
    bunker::server::Builder::new()
        .port(1505)
        .addr([127, 0, 0, 1])
        // Bunker set to bind on 127.0.0.1:1505
        .threads(12) // Up to 12 connections
        .max_response_length(4096) // Responses cannot be longer than 4096 characters
        .read_buffer_size(1024) // Up to 1024 characters read from requests
        .parse_separator(&vec!['\n']) // Separates path from message by a new line
        .endconn_msg("kd9ascMm..a/c:Dw1[]".to_string())
        // Connection will be terminated if controller returns this string
        .response_on_error(CustomError::InternalServerError.to_string())
        // If internal error occurs, will reply with given string
        .configure_routes(routes) 
        // Routes "someroute" to SomeSpecificHandler, and a default handler 
        .debugger_level_error()
        .set_custom_debugger(Box::new(CustomDebugWriter::new()))
        // Custom debug writer that only writes if internal error occurs
        .build()
        .run() 
        // Server starts
} 

fn routes(builder: bunker::server::RouteMapBuilder) -> bunker::server::RouteMapBuilder {
    let builder = builder.register(
        SomeDefaultHandler::new(), 
        bunker::registerable::Route::NotFound
    ): // If no path matches any routes, will route to SomeDefaultHandler
        
    builder.register(
        SomeSpecificHandler::new(),
        bunker::registerable::Route::Path("someroute".to_string())
    ) // Routes "someroute" to SomeSpecificHandler
}
```