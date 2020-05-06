use std::fmt::Formatter;
use std::option::NoneError;
use std::{error, fmt};

#[derive(Debug, Clone)]
pub struct ProcessingError {
    msg: String,
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Tree processing error {}", self.msg)
    }
}

impl error::Error for ProcessingError {}

impl From<std::option::NoneError> for ProcessingError {
    fn from(n: NoneError) -> Self {
        ProcessingError {
            msg: "None error".to_string(),
        }
    }
}
