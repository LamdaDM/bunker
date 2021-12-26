use crate::{exception::BunkerError};

/// Basic interface for accepting any request and returning a response.
/// 
/// Note: If an error occurs, it is not mandatary to return some variation of BunkerError.
/// `BunkerError::BadRequest` is a variant that you may use if you wish. 
pub trait Controller : Send + Sync {
    fn accept(&self, msg: String) -> Result<String, BunkerError>;
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