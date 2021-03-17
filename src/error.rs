use std::{
    error::Error,
    fmt::{Display, Formatter},
};

#[derive(Debug, Clone)]
pub struct FlasherError(pub String);

impl FlasherError {
    pub fn new(message: impl ToString) -> Self {
        Self(message.to_string())
    }
}

impl Error for FlasherError {}

impl Display for FlasherError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
