use thiserror::Error;
use zetina_common::job_witness::JobWitness;

pub mod behavior_handler;
pub mod cli;
pub mod swarm;
pub mod tonic;

pub use cli::run;

#[derive(Error, Debug)]
pub enum DelegatorError {
    #[error("broadcast_send_error")]
    BroadcastSendError(#[from] tokio::sync::broadcast::error::SendError<JobWitness>),

    #[error("io")]
    Io(#[from] std::io::Error),

    #[error("serde")]
    Serde(#[from] serde_json::Error),
}
