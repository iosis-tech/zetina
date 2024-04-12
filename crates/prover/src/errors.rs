use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProverControllerError {
    #[error("task not found")]
    TaskNotFound,

    #[error("task not found")]
    TaskTerminated,

    #[error("io")]
    Io(#[from] std::io::Error),
}
