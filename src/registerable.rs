use crate::{exception::BunkerError};

/// Basic interface for accepting any request and returning a response.
/// 
/// Note: If an error occurs, it is not mandatary to return some variation of BunkerError.
/// `BunkerError::BadRequest` is a variant that you may use if you wish. 
pub trait Controller : Send + Sync {
    fn accept(&self, msg: String) -> Result<String, BunkerError>;
}