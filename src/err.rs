use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ParseError {
    pub msg: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error for ParseError {
    fn description(&self) -> &str {
        self.msg.as_str()
    }
}


#[derive(Debug)]
pub struct RuntimeError {
    pub msg: String,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error for RuntimeError {
    fn description(&self) -> &str {
        self.msg.as_str()
    }
}
