use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct BlueZError {
    message: String,
}

impl BlueZError {
    pub fn new(message: String) -> BlueZError {
        BlueZError{message}
    }
}

impl Error for BlueZError {
    fn description(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for BlueZError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.message)
    }
}
