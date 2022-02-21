use std::{fmt::Display, io};

/// Error messages strictly related to bunker's execution.
/// 
/// **WARNING: BunkerError is being phased out for external use since 0.2**
#[derive(Debug)]
#[deprecated(since="0.2", note="\nnow has no application externally, and is being replaced internally")]
pub enum BunkerError {
    NoControllerFound(String),
    InvalidThreadPoolSize(usize),
    IO(io::Error)
}

#[allow(deprecated)]
impl From<io::Error> for BunkerError { 
    fn from(err: io::Error) -> BunkerError { BunkerError::IO(err) }
}

#[allow(deprecated)]
impl Display for BunkerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BunkerError::NoControllerFound(msg) => write!(f, "EO1 Bad Request: {}", msg),
            BunkerError::InvalidThreadPoolSize(input) => write!(f, 
                "Invalid size assigned to the threadpool!\nGiven: {}\nExpected a number greater than 0", 
                &input),
            BunkerError::IO(err) => Display::fmt(err, f),
        }
    }
}

pub enum InternalError {
    BadRequest(u64),
    InvalidThreadPoolSize(usize),
    IO(io::Error)
}

impl From<io::Error> for InternalError { 
    fn from(err: io::Error) -> InternalError { InternalError::IO(err) }
}

#[allow(deprecated)]
impl Display for InternalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InternalError::BadRequest(order_number) => write!(f, 
                "Order Number {}'s Request could not be matched to any path! Please add a NotFound controller.", 
                order_number),
            InternalError::InvalidThreadPoolSize(input) => write!(f, 
                "Invalid size assigned to the threadpool!\nGiven: {}\nExpected a number greater than 0", 
                input),
            InternalError::IO(err) => Display::fmt(err, f),
        }
    }
}