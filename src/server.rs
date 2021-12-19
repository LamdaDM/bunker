use crate::{exception::BunkerError, internal::Threadpool, registerable, cfg};

use std::{cell::Cell, collections::BTreeMap, io::{ErrorKind, Read, Write}, net::{SocketAddr, TcpListener}, rc::Rc, sync::Arc};

pub type RouteMap = BTreeMap<String, Box<dyn registerable::Controller>>;

/// Builder for configuring server options. 
/// After setting the options, call `bunker::server::Builder::build`, which will consume the Builder and return a `bunker::server::Instance`.
/// 
/// **DEFAULTS:**
/// ```
/// port: 3055
/// addr: [127, 0, 0, 1]
/// threads: 1
/// read_buffer_size: 1024
/// shutdown_msg: "CCONN"
/// route_map: Empty
/// parse_options: position(1) 
/// debug: On
/// ```
#[allow(dead_code)]
pub struct Builder {
    port: u16,
    threads: usize,
    read_buffer_size: usize,
    addr: [u8; 4],
    endconn_msg: String,
    parse_options: cfg::ParseOptions,
    debug: cfg::Debug,
    rm: RouteMap
}

impl Builder {
    pub fn new() -> Builder {
        Builder { 
            port: 3055, 
            threads: 1, 
            read_buffer_size: 1024,
            addr: [127, 0, 0, 1],
            endconn_msg: "CCONN".to_string(),
            parse_options: cfg::ParseOptions::position(1),
            debug: cfg::Debug::on(),
            rm: RouteMap::new()
        }
    }

    /// Will stop Bunker from writing debugging information to the standard output.
    pub fn debugger_off(self) -> Builder {
        Builder{ debug: cfg::Debug::off(), ..self }
    }

    /// Sets the port for the TCP server.
    pub fn port(self, port: u16) -> Builder { 
        Builder{ port, ..self } 
    }

    pub fn addr(self, addr: [u8; 4]) -> Builder {
        Builder{ addr, ..self }
    }

    /// Sets the number of threads given to the internal threadpool.
    ///  
    /// *Must be greater than 0, or else the server will panic on initialization*.
    pub fn threads(self, threads: usize) -> Builder { 
        Builder{ threads, ..self } 
    }
    
    /// Sets the maximum number of characters for a single incoming message.
    pub fn read_buffer_size(self, read_buffer_size: usize) -> Builder { 
        Builder{ read_buffer_size, ..self }
    }
    
    /// Sets the string that will be checked for to close the connection.
    pub fn endconn_msg(self, endconn_msg: String) -> Builder { 
        Builder{endconn_msg, ..self} 
    }

    /// Registers a `registerable::Controller` in the route map, with the path being used as the key to find that controller.
    /// For a client to access an endpoint, the first string after being split must match the path given here. 
    pub fn register(mut self, controller: Box<dyn registerable::Controller>, path: String) -> Builder {
        self.rm.insert(path, controller);
        self
    
    }

    /// Configures the server to split incoming messages at the first instance of a character matching one of the given separators.
    /// The first string will be used as the path to pass the second string down to any matching controllers. 
    pub fn parse_separator(self, separator: &Vec<char>) -> Builder { 
        Builder{ parse_options: cfg::ParseOptions::separator(separator.clone()), ..self } 
    }

    /// Configures the server to split incoming messages at the given position.
    /// The first string will be used as the path to pass the second string down to any matching controllers. 
    pub fn parse_position(self, position: usize) -> Builder { 
        Builder{ parse_options: cfg::ParseOptions::position(position), ..self } 
    }

    /// Converts the builder into a `server::Config`, for creating an Instance.
    fn create_cfg(self) -> cfg::ConfigAlias {
        Arc::new(cfg::Config {
            port: self.port, 
            addr: self.addr,
            threads: self.threads, 
            read_buffer_size: self.read_buffer_size, 
            endconn_msg: self.endconn_msg, 
            parse_options: self.parse_options,
            debug: self.debug,
            rm: self.rm,
        })
    }
    
    /// Consumes the Builder and a RouteMapBuilder to construct an Instance. After the Instance is created, call `server::Host::run` to start the server.
    pub fn build(self) -> Host
    { 
        Host::new(self.create_cfg())
    }
}

/// A multi-threaded server. All fields are immutable 
/// from the Host's creation and onwards.
/// Can only be created through `server::Builder`. 
/// Call `Host::run()` to start the server.
pub struct Host {
    threadpool: Threadpool,
    cfg: cfg::ConfigAlias,
    ordern: Rc<Cell<u64>>,
}

impl Host {
    fn new(cfg: cfg::ConfigAlias) -> Host {
        Host{ 
            threadpool: Threadpool::new(cfg.threads), 
            ordern: Rc::new(Cell::new(0)),
            cfg
        }
    }

    pub fn get_port(&self) -> u16 { self.cfg.port }
    pub fn get_thread_count(&self) -> usize { self.threadpool.get_size() }
    pub fn get_read_buffer_size(&self) -> usize { self.cfg.read_buffer_size }
    pub fn get_parse_option(&self) -> (Option<usize>, &Option<Vec<char>>) {
        self.cfg.parse_options.get_prop()
    }
    pub fn get_enconn_msg(&self) -> &str { &self.cfg.endconn_msg }
    pub fn is_debugger_on(&self) -> bool { self.cfg.debug.state() }

    /// Initializes the TCP socket server, binding to the assigned port, 
    /// and starts listening for connections. Once a connection is found,
    /// the stream is passed onto a new thread and the request/response cycle
    /// starts.
    /// 
    /// The request is parsed and passed down to any matching controllers 
    /// according to the given options and route map set in the builder. 
    /// Afterwards the buffers are flushed and the cycle continues until the
    /// client ends the connection or a shutdown message is received from the 
    /// controller.
    /// 
    /// If the debugger is set to on, debugging messages will be printed
    /// on initialization, and during communication with clients where
    /// the order number for that connection is appended for identification.
    pub fn run(self) {
        const DEBUG_HANDLE: &str = "server::Instance::run";
        
        self.cfg.debug.write(DEBUG_HANDLE, "Server initialized.");

        let cfg = Arc::clone(&self.cfg);

        let sock_addr = SocketAddr::from((cfg.addr, cfg.port));
        let listener = TcpListener::bind(sock_addr).unwrap();

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {            
                    let cfg = Arc::clone(&self.cfg);

                    // Increments original order number, then clones copy.
                    // No need for atomic as number only changes in single-threaded context.
                    Rc::clone(&self.ordern)
                        .set(Rc::clone(&self.ordern).get() + 1); 
                    let ordern_copy = Rc::clone(&self.ordern).get();
                    
                    // String indicating origin, with a number corresponding to the open connection thats being served.
                    let local_debug_handle = format!("{}::ordern({})", DEBUG_HANDLE, ordern_copy);

                    cfg.debug.write(&local_debug_handle, "Connection initiated.");

                    self.threadpool.execute(move|| {

                        let separator_parse = cfg.parse_options.is_separators();

                        // Buffer for data received from client.
                        let mut buff = vec![0 as u8; cfg.read_buffer_size];
                    
                        let mut end = false;

                        loop {
                            let size = stream.read(&mut buff[..]).unwrap();

                            cfg.debug.write(&local_debug_handle, &format!("(size: {}) Raw incoming data: {:?}", size, &buff[0..size]));

                            let req = String::from_utf8_lossy(&buff[0..size]);
                            let req = req.trim(); // Removes whitespace for whitespace-sensitive parsing options.

                            cfg.debug.write(&local_debug_handle, &format!("Received message: {}", req));

                            let res: Result<String, BunkerError> = {
                                
                                // Parses data according to which flag is set.
                                match if separator_parse {
                                    match req.split_once(
                                        &cfg.parse_options.separators
                                            .as_ref()
                                            .unwrap()
                                            [..]
                                    ) {
                                        Some(matched) => Ok(matched),
                                        None => {
                                            cfg.debug.write_err(
                                                &local_debug_handle, 
                                                &format!(
                                                    "Could not parse incoming message with the given separators ({:?})\nMessage: {}", 
                                                    &cfg.parse_options.separators,
                                                    &req
                                                )
                                            );
                                            Err(BunkerError::BadRequest(
                                                "No valid path found in message.".to_string()
                                            ))
                                        },
                                    }
                                } else {
                                    let (path, msg) = req.split_at(cfg.parse_options.position.unwrap());
                                    cfg.debug.write(
                                        &local_debug_handle, 
                                        &format!(
                                            "Parsed message with the given position ({})!\nPath: {}\nMessage: {}", 
                                            cfg.parse_options.position.unwrap(), 
                                            path, 
                                            msg
                                        )
                                    );
                                    Ok((path, msg))
                                } {
                                    // Matches the result of the parse.
                                    Ok((path, msg)) => {
                                        if let Some(req)
                                            = cfg.rm.get(path) {
                                                req.accept(msg.to_string())
                                        } else { 
                                            // Error results from the path not matching any key in the map.
                                            Err(BunkerError::BadRequest("Could not find matching path.".to_string())) 
                                        }
                                    },
                                    Err(err) => Err(err),
                                }
                            };

                            // Ensures all results are a String.
                            let mut res = match res {
                                Ok(msg) => msg,
                                Err(err) => err.to_string(),
                            };

                            if res == cfg.endconn_msg { 
                                
                                cfg.debug.write(
                                    &local_debug_handle, 
                                    "Controller ending connection."
                                );

                                res = String::from("Closing connection...");
                                end = true;
                            } // Writes confirmation for closing the connection and signals to break;

                            // Writes to the stream and then handles buffers.
                            cfg.debug.write(&local_debug_handle, &format!("Writing response: {}", res));
                            res.push('\n');
                            stream.write(res.as_bytes()).unwrap();
                            stream.flush().unwrap();
                            buff.flush().unwrap();

                            if end { break }
                        }
                    
                        cfg.debug.write(&local_debug_handle, "Closing connection.");
                    })
                },
                Err(err) => {
                    match err.kind() {
                        ErrorKind::Interrupted => continue,
                        _ => { panic!("UNEXPECTED ERROR - {:?}", err) }
                    }
                }
            }
        }
        
        self.cfg.debug.write(DEBUG_HANDLE, "Shutting down server...");
    }
}