use sharp_p2p_common::{job::Job, job_witness::JobWitness};

use crate::errors::ProverControllerError;

pub trait Prover {
    fn init() -> impl ProverController;
}

pub trait ProverController {
    async fn prove(&mut self, job: Job) -> Result<JobWitness, ProverControllerError>;
    async fn terminate(&mut self, job: &Job) -> Result<(), ProverControllerError>;
    async fn drop(self) -> Result<(), ProverControllerError>;
}
