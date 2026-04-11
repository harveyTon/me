use thiserror::Error;

#[derive(Debug, Error)]
pub enum MeError {
    #[error("{0}")]
    Message(String),
}

impl MeError {
    pub fn message(value: impl Into<String>) -> Self {
        Self::Message(value.into())
    }
}
