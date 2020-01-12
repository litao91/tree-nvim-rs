use std::error::Error;
use std::fmt;

macro_rules! define_error {
    ($($err:ident)*) => { $(
#[derive(Debug)]
pub struct $err {
    details: String,
}

impl $err {
    pub fn new(msg: &str) -> Self {
        Self {
            details: msg.to_string(),
        }
    }

    pub fn from_string(details: String) -> Self {
        Self {
            details
        }
    }
}

impl fmt::Display for $err {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for $err {
    fn description(&self) -> &str {
        &self.details
    }
}

)*}
}

define_error!(ArgError);
