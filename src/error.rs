use std::io;
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

pub fn is_broken_pipe(error: &anyhow::Error) -> bool {
    error
        .chain()
        .filter_map(|cause| cause.downcast_ref::<io::Error>())
        .any(|io_error| io_error.kind() == io::ErrorKind::BrokenPipe)
}
