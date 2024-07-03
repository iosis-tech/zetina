pub mod behavior_handler;
pub mod cli;
pub mod swarm;
pub mod tonic;

pub use cli::run;
use thiserror::Error;
use zetina_prover::errors::ProverControllerError;
use zetina_runner::errors::RunnerControllerError;

#[derive(Error, Debug)]
pub enum ExecutorError {
    #[error("prover_controller_error")]
    ProverControllerError(#[from] ProverControllerError),

    #[error("runner_controller_error")]
    RunnerControllerError(#[from] RunnerControllerError),

    #[error("mpsc_send_error")]
    MpscSendError(#[from] tokio::sync::mpsc::error::SendError<Vec<u8>>),

    #[error("io")]
    Io(#[from] std::io::Error),

    #[error("serde")]
    Serde(#[from] serde_json::Error),
}
