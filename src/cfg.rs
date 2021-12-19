use std::{sync::Arc};

use crate::server::RouteMap;
pub struct Debug {
    state: bool
}

#[allow(dead_code)]
impl Debug {
    pub fn on() -> Debug { Debug{ state: true } }

    pub fn off() -> Debug { Debug{ state: false } }

    pub fn state(&self) -> bool { self.state }

    pub fn flip(&mut self) { self.state = !self.state; }

    pub fn write(&self, origin: &str, msg: &str) {
        if self.state {
            println!("{o}:\n{m}", o = &origin, m = &msg);
        }
    }

    pub fn write_err(&self, origin: &str, msg: &str) {
        if self.state {
            eprintln!("{o}:\n{m}", o = &origin, m = &msg);
        }
    }
}

pub type ConfigAlias = Arc<Config>;

pub struct Config {
    pub port: u16,
    pub addr: [u8; 4],
    pub threads: usize,
    pub read_buffer_size: usize,
    pub endconn_msg: String,
    pub parse_options: ParseOptions,
    pub debug: Debug,
    pub rm: RouteMap
}

pub struct ParseOptions {
    pub position: Option<usize>,
    pub separators: Option<Vec<char>>
}

impl ParseOptions {
    pub fn position(position: usize) -> ParseOptions { 
        ParseOptions{ position: Some(position), separators: None } 
    }
    pub fn separator(separator: Vec<char>) -> ParseOptions { 
        ParseOptions{ position: None, separators: Some(separator) } 
    }
    pub fn is_empty(&self) -> bool { 
        self.position.is_none() && self.separators.is_none() 
    }
    pub fn is_full(&self) -> bool {
        self.position.is_some() && self.separators.is_some()
    }
    pub fn get_prop(&self) -> (Option<usize>, &Option<Vec<char>>) { 
        (self.position, &self.separators) 
    }

    /// Checks if option is set to using separators. If it is not,
    /// it is set to using position. 
    /// 
    /// Will panic if somehow both or neither are set.
    pub fn is_separators(&self) -> bool {
        
        if self.is_full() || self.is_empty() {
            panic!(
                "INTERNAL ERROR: Parse options are invalid. 
                    Both position and separators are {}",
                if self.is_full() { "set" }
                else { "empty" }
            );
        }

        self.separators.is_some() 
            && self.position.is_none()
    }
}