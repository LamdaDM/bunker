use std::{sync::Arc, io::{stdout, Write, ErrorKind, stderr}, collections::BTreeMap, fmt};

use crate::{registerable::{self, DebugFmt, Route}};

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum DebugSetting {
    None,
    Standard,
    Error
}

impl fmt::Display for DebugSetting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DebugSetting::None => write!(f, "None"),
            DebugSetting::Standard => write!(f, "Standard"),
            DebugSetting::Error => write!(f, "Error"),
        }
    }
}

pub struct Debug {
    state: DebugSetting,
    w: Box<dyn registerable::DebugFmt>
}

impl Debug {

    pub fn state_equals(&self, other: DebugSetting) -> bool { self.state >= other }

    pub fn new(w: Box<dyn registerable::DebugFmt>) -> Debug { Debug{ state: DebugSetting::Standard, w } }

    pub fn replace_writer(&mut self, w: Box<dyn registerable::DebugFmt>) { self.w = w }
    
    pub fn off(&mut self) { self.state = DebugSetting::None; }
    pub fn standard(&mut self) { self.state = DebugSetting::Standard }
    pub fn error(&mut self) { self.state = DebugSetting::Error }

    pub fn get_setting(&self) -> String { self.state.to_string() }

    pub fn write(&self, origin: &str, message: &str) {
        if self.state > DebugSetting::None {
            match stdout().write_all(self.w.debug(origin, message).as_bytes()) {
                Ok(_) => (),
                Err(err) => match err.kind() {
                    ErrorKind::Interrupted => (),
                    _other => self.write_err("cfg::Debug::write", &format!("IO Error: {}", &err)),
                }
            };
        }
    }

    pub fn write_err(&self, origin: &str, message: &str) {
        if self.state > DebugSetting::Standard {
            match stderr().write_all(self.w.debug(origin, message).as_bytes()) {
                Ok(_) => (),
                Err(err) => match err.kind() {
                    ErrorKind::Interrupted => (),
                    _other => self.write_err("cfg::Debug::write", &format!("IO Error: {}", &err)),
                }
            };        
        }
    }
}

pub struct DefaultDebugger;

impl DebugFmt for DefaultDebugger {
    fn debug(&self, origin: &str, message: &str) -> String{
        format!("{}:\n{}", origin, message)
    }

    fn debug_err(&self, origin: &str, message: &str) -> String {
        format!("{}:\nERROR! {}", origin, message)
    }
}

pub type ConfigAlias = Arc<Config>;
pub type RouteMap = BTreeMap<Route, Box<dyn registerable::Controller>>;

pub struct Config {
    pub port: u16,
    pub addr: [u8; 4],
    pub threads: usize,
    pub read_buffer_size: usize,
    pub endconn_msg: String,
    pub parse_options: ParseOptions,
    pub debug: Debug,
    pub rm: RouteMap,
    pub mrl: usize,
    pub er: String
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