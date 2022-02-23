use std::{cell::RefCell, rc::Rc, fmt};

use crate::exception::BunkerError;

#[derive(PartialEq, PartialOrd, Eq, Ord)]
pub enum Route {
    NotFound,
    Path(String)
}

/// Basic interface for accepting any request and returning a response.
/// `Controller::serve` will be called if a path matches this controller, so
/// provide an implementation for response-writing logic.
/// 
/// Expects a response even if an error occurred.
/// Any error should be saved in `out_debug`, which would then be passed to the debugger.
/// 
/// **WARNING:** *Default implementation will panic.*
#[allow(unused_variables)]
pub trait Controller : Send + Sync {
    #[deprecated(since="0.2", note="use `Controller::serve` instead.")]
    fn accept(&self, msg: String) -> Result<String, BunkerError> {
        panic!("Provide implementation of Controller::serve.")
    }

    /// Is called if a request's path matches this controller's route.
    /// 
    /// - `msg` The request received from the client, with the path prefix removed.
    /// - `out_debug` Any errors should be converted to a string and stored in here.
    ///     If filled, the inner string will be passed to `Debug::write_err`.
    fn serve(&self, msg: String, out_debug: Rc<RefCell<String>>) -> String {
        match self.accept(msg) {
            Ok(res) => res,
            Err(err) => {
                out_debug.replace(err.to_string());
                String::new()
            },
        }
    }
}

/// For custom implementations of Bunker's formatter for debugging..
/// 
/// Its two methods, `debug` and `debug_err` return a string that is used to write to 
/// either stdout or stderr respectively. `debug_err` has a default implementation which only calls `debug`.
/// Unless a custom implementation of `debug_err` is provided, 
/// the only difference is it will write to stderr instead of stdout.
pub trait DebugFmt: Send + Sync {
    fn debug(&self, origin: &str, message: &str) -> String;
    fn debug_err(&self, origin: &str, message: &str) -> String { self.debug(origin, message) }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
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

#[derive(Clone)]
pub enum ParseOptions {
    Position(usize),
    Separators(Vec<char>)
}

impl ParseOptions {
    pub fn position(position: usize) -> ParseOptions { 
        ParseOptions::Position(position)
    }
    pub fn separator(separator: Vec<char>) -> ParseOptions { 
        ParseOptions::Separators(separator)
    }

    /// Checks if option is set to using separators. If it is not,
    /// it is set to using position. 
    pub fn is_separators(&self) -> bool {
        match self {
            ParseOptions::Position(_) => false,
            ParseOptions::Separators(_) => true,
        }
    }
}