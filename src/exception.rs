use std::{fmt::Display, io};

/// Error messages strictly related to bunker's execution.
/// For public use, BunkerError::BadRequest is suggested.
/// Any eror message given to BunkerError::BadRequest 
/// will be written in the response unless directed otherwise.
pub enum BunkerError {
    BadRequest(String),
    InvalidThreadPoolSize(usize),
    IO(io::Error)
}

impl From<io::Error> for BunkerError { 
    fn from(err: io::Error) -> BunkerError { BunkerError::IO(err) }
}

impl Display for BunkerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BunkerError::BadRequest(msg) => write!(f, "EO1 Bad Request: {}", msg),
            BunkerError::InvalidThreadPoolSize(input) => write!(f, 
                "Invalid size assigned to the threadpool!\nGiven: {}\nExpected a number greater than 0", 
                &input),
            BunkerError::IO(err) => Display::fmt(err, f),
        }
    }
}