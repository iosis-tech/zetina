use crate::errors::ProverControllerError;
use sharp_p2p_common::{job_trace::JobTrace, job_witness::JobWitness};

pub trait Prover {
    fn init() -> impl ProverController;
}

pub trait ProverController {
    async fn prove(&mut self, job_trace: JobTrace) -> Result<JobWitness, ProverControllerError>;
    async fn terminate(&mut self, job_trace_hash: u64) -> Result<(), ProverControllerError>;
    async fn drop(self) -> Result<(), ProverControllerError>;
}
