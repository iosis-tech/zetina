use thiserror::Error;

#[derive(Error, Debug)]
pub enum RunnerControllerError {
    #[error("task not found")]
    TaskNotFound,

    #[error("task not found")]
    TaskTerminated,

    #[error("io")]
    Io(#[from] std::io::Error),

    #[error("serde")]
    Serde(#[from] serde_json::Error),

    #[error("proof parsing error")]
    ProofParseError(String),
}
