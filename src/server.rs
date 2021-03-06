use crate::{exception::InternalError, internal::Threadpool, registerable::{self, Route, DebugSetting}, cfg::{self, DefaultDebugger, RouteMap}};

use std::{cell::{Cell, RefCell}, io::{ErrorKind, Read, Write}, net::{SocketAddr, TcpListener}, rc::Rc, sync::Arc};

pub struct RouteMapBuilder(RouteMap);

impl RouteMapBuilder {
    fn new() -> RouteMapBuilder {
        RouteMapBuilder(RouteMap::new())
    }

    /// Registers a `registerable::Controller` in the route map, with the path being used as the key to find that controller.
    /// For a client to access an endpoint, the route after being split must match the path given here. 
    pub fn register(mut self, controller: Box<dyn registerable::Controller>, path: Route) -> RouteMapBuilder {
        if self.0.contains_key(&path) {
            self.0.remove(&path);
        }

        self.0.insert(path, controller);
        self
    }

    fn build(self) -> RouteMap { self.0 }
}

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
/// debug_writer: Default Writer
/// debug: On
/// max_response_length: 9999
/// ```
#[allow(dead_code)]
pub struct Builder {
    port: u16,
    threads: usize,
    read_buffer_size: usize,
    addr: [u8; 4],
    endconn_msg: String,
    parse_options: registerable::ParseOptions,
    debug: cfg::Debug,
    rmb: RouteMapBuilder,
    max_response_length: usize,
    response_on_error: String
}

impl Builder {
    pub fn new() -> Builder {
        Builder { 
            port: 3055, 
            threads: 1, 
            read_buffer_size: 1024,
            addr: [127, 0, 0, 1],
            endconn_msg: "CCONN".to_string(),
            parse_options: registerable::ParseOptions::position(1),
            debug: cfg::Debug::new(Box::new(DefaultDebugger)),
            rmb: RouteMapBuilder::new(),
            max_response_length: 9999,
            response_on_error: String::new()
        }
    }

    #[deprecated(since="0.2", note="use Builder::debugger_level_*")]
    /// Will stop Bunker from writing debugging information to the standard output.
    pub fn debugger_off(mut self) -> Builder {
        self.debug.off();
        self
    }

    /// Registers a custom implementation of `registerable::DebugFmt` for debugging. 
    pub fn set_custom_debugger(mut self, debugger: Box<dyn registerable::DebugFmt>) -> Builder {
        self.debug.replace_writer(debugger);
        self
    }

    /// Sets debugger to never write.
    pub fn debugger_level_none(mut self) -> Builder {
        self.debug.off();
        self
    }

    /// Sets debugger to write anything.
    pub fn debugger_level_standard(mut self) -> Builder {
        self.debug.standard();
        self
    }

    /// Sets debugger to write errors.
    pub fn debugger_level_error(mut self) -> Builder {
        self.debug.error();
        self
    }

    pub fn port(self, port: u16) -> Builder { 
        Builder{ port, ..self } 
    }

    pub fn addr(self, addr: [u8; 4]) -> Builder {
        Builder{ addr, ..self }
    }

    pub fn max_response_length(self, max_response_length: usize) -> Builder {
        Builder{ max_response_length, ..self }
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

    /// Sets the message to be sent in the event of an internal error.
    pub fn response_on_error(self, error_response: String) -> Builder {
        Builder{ response_on_error: error_response, ..self }
    }

    /// Registers a `registerable::Controller` in the route map, with the path being used as the key to find that controller.
    /// For a client to access an endpoint, the route after being split must match the path given here. 
    pub fn register(self, controller: Box<dyn registerable::Controller>, path: Route) -> Builder {
        let rmb = self.rmb.register(controller, path);
        Builder{rmb, ..self}
    }

    pub fn configure_routes<F>(self, f: F) -> Builder
        where 
            F : FnOnce(RouteMapBuilder) -> RouteMapBuilder + 'static
    {
        let rmb = f(self.rmb);
        Builder{rmb, ..self}
    }

    /// Configures the server to split incoming messages at the first instance of a character matching one of the given separators.
    /// The first string will be used as the path to pass the second string down to any matching controllers. 
    pub fn parse_separator(self, separator: &Vec<char>) -> Builder { 
        Builder{ parse_options: registerable::ParseOptions::separator(separator.clone()), ..self } 
    }

    /// Configures the server to split incoming messages at the given position.
    /// The first string will be used as the path to pass the second string down to any matching controllers. 
    pub fn parse_position(self, position: usize) -> Builder { 
        Builder{ parse_options: registerable::ParseOptions::position(position), ..self } 
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
            rm: self.rmb.build(),
            mrl: self.max_response_length,
            er: self.response_on_error
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
    pub fn get_parse_option(&self) -> registerable::ParseOptions {
        self.cfg.parse_options.clone()
    }
    pub fn get_endconn_msg(&self) -> &str { &self.cfg.endconn_msg }
    
    pub fn get_debugger_level(&self) -> DebugSetting { self.cfg.debug.get_setting() }

    #[deprecated(since="0.2", note="use get_debugger_level instead and compare variants")]
    pub fn is_debugger_on(&self) -> bool { self.cfg.debug.is_state(DebugSetting::Standard) }

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
        const DEBUG_HANDLE: &str = "server::Host::run";
        
        self.cfg.debug.write(DEBUG_HANDLE, "Server initialized.");

        let cfg = Arc::clone(&self.cfg);

        let sock_addr = SocketAddr::from((cfg.addr, cfg.port));
        let listener = TcpListener::bind(sock_addr).unwrap();

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let cfg = Arc::clone(&cfg);

                    // Increments original order number, then clones copy.
                    // No need for atomic as number only changes in single-threaded context.
                    Rc::clone(&self.ordern)
                        .set(Rc::clone(&self.ordern).get() + 1); 
                    let ordern_copy = Rc::clone(&self.ordern).get();
                    
                    // String indicating origin, with a number corresponding to the open connection thats being served.
                    let local_debug_handle = format!("{}::ordern({})", DEBUG_HANDLE, ordern_copy);

                    cfg.debug.write(&local_debug_handle, "Connection initiated.");

                    self.threadpool.execute(move|| {

                        // Buffer for data received from client.
                        let mut buff = vec![0 as u8; cfg.read_buffer_size];
                    
                        let mut end = false;

                        loop {
                            let size = stream.read(&mut buff[..]).unwrap();

                            cfg.debug.write(&local_debug_handle, 
                                &format!("(size: {}) Raw incoming data: {:?}", size, &buff[0..size]));

                            let req = String::from_utf8_lossy(&buff[0..size]);
                            let req = req.trim(); // Removes whitespace for whitespace-sensitive parsing options.

                            cfg.debug.write(&local_debug_handle, 
                                &format!("Received message: {}", req));

                            let error_b = Rc::new(RefCell::new(String::new()));

                            let mut res: String = {
                                
                                // Parses data according to which flag is set.
                                let (path, msg) = match &cfg.parse_options {
                                    registerable::ParseOptions::Position(pos) => {
                                        let (path, msg) = req.split_at(*pos);
                                        cfg.debug.write(
                                            &local_debug_handle,
                                            &format!(
                                                "Parsed message with the given position ({})!\nPath: {}\nMessage: {}", 
                                                pos, 
                                                path, 
                                                msg
                                            )
                                        );

                                        (Route::Path(path.to_string()), msg)
                                    },
                                    registerable::ParseOptions::Separators(chars) => {
                                        match req.split_once(&chars[..]) {
                                            Some((path, msg)) => (Route::Path(path.to_string()), msg),
                                            None => (Route::NotFound, req),
                                        }
                                    },
                                };
                                
                                // Matches the result of the parse.
                                if let Some(controller) = cfg.rm.get(&path) {
                                        controller.serve(msg.to_string(), Rc::clone(&error_b))
                                } else { 

                                    if let Some(controller) = cfg.rm.get(&Route::NotFound) {
                                        controller.serve(msg.to_string(), Rc::clone(&error_b))
                                    } else {
                                        // Error results from the path not matching any key in the map.
                                        let err = InternalError::NoControllerFound(ordern_copy);
                                        error_b.replace(err.to_string());
                                        cfg.er.to_owned()
                                    }
                                }
                            };

                            let error = error_b.take();

                            if !error.is_empty() {
                                cfg.debug.write_err(&local_debug_handle, &error);
                                res = cfg.er.to_owned();
                            }

                            if res == cfg.endconn_msg { 
                                
                                cfg.debug.write(
                                    &local_debug_handle, 
                                    "Controller ending connection."
                                );

                                res = String::from("Closing connection...");
                                end = true;
                            } // Writes confirmation for closing the connection and signals to break;

                            // Prepend length of message to response according to mrl
                            let res = Host::prepend_length(&res, cfg.mrl).unwrap();

                            // Writes to the stream and then handles buffers.
                            cfg.debug.write(&local_debug_handle, &format!("Writing response: {}", res));

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
                        _ => { println!("UNEXPECTED ERROR - {:?}", err) }
                    }
                }
            }
        }
        
        self.cfg.debug.write(DEBUG_HANDLE, "Shutting down server...");
    }

    /// Prepends length of the message to the response, according to the given response length.
    /// Returns an error if the message exceeds the max response length.
    fn prepend_length(message: &str, mrl: usize) -> Result<String, ()> {
        let len = message.len();
        if len > mrl { return Err(()); }

        let len_str = len.to_string();
        let len_str_len = len_str.len();
        
        let mut mrl = mrl;

        let mut digit_count = 0;
        while mrl > 0 {
            mrl /= 10;
            digit_count += 1;
        }

        let leading = digit_count - len_str_len;
        
        let mut out = String::new();
        for _ in 0..leading { out += "0"; }

        Ok(out + &len_str + &message)
    }
}
