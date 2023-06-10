use std::{
    error::Error,
    fmt::Display,
    num::{ParseFloatError, ParseIntError},
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct InError {
    pub a: String,
}

impl From<ParseIntError> for InError {
    fn from(value: ParseIntError) -> Self {
        Self {
            a: value.to_string(),
        }
    }
}

impl From<std::io::Error> for InError {
    fn from(value: std::io::Error) -> Self {
        Self {
            a: value.to_string(),
        }
    }
}
impl From<serde_json::Error> for InError {
    fn from(value: serde_json::Error) -> Self {
        Self {
            a: value.to_string(),
        }
    }
}
impl From<ParseFloatError> for InError {
    fn from(value: ParseFloatError) -> Self {
        Self {
            a: value.to_string(),
        }
    }
}

impl Display for InError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.a)
    }
}
impl Error for InError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}
