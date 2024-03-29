use std::fmt;
use std::error;

#[derive(Debug, Clone)]
pub struct InvalidSessionError {
    pub msg: String,
}

impl error::Error for InvalidSessionError {}

impl fmt::Display for InvalidSessionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // FIXME: This message should be adjusted once the home dir is configurable
        write!(f, "{}", self.msg)
    }
}
