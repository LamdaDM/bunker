use std::{sync::Arc, io::{stdout, Write, ErrorKind, stderr}, collections::BTreeMap};

use crate::{registerable::{self, DebugFmt, Route, ParseOptions, DebugSetting}};

pub struct Debug {
    state: DebugSetting,
    w: Box<dyn registerable::DebugFmt>
}

impl Debug {

    pub fn is_state(&self, other: DebugSetting) -> bool { self.state >= other }

    pub fn new(w: Box<dyn registerable::DebugFmt>) -> Debug { Debug{ state: DebugSetting::Standard, w } }

    pub fn replace_writer(&mut self, w: Box<dyn registerable::DebugFmt>) { self.w = w }
    
    pub fn off(&mut self) { self.state = DebugSetting::None; }
    pub fn standard(&mut self) { self.state = DebugSetting::Standard }
    pub fn error(&mut self) { self.state = DebugSetting::Error }

    pub fn get_setting(&self) -> DebugSetting { self.state.clone() }

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