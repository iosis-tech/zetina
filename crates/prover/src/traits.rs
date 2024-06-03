use crate::errors::ProverControllerError;
use zetina_common::{job_trace::JobTrace, job_witness::JobWitness, process::Process};

/*
    The `ProverController` trait defines the behavior for creating zkSTARK proofs from a `JobTrace` obtained from a `RunnerController`.
    It abstracts over the prover process for ease of management and allows for convenient abortion of execution.
    The `run` method takes a `JobTrace` as input and returns a `Result` containing a `Process` object,
    which serves as a handle for controlling the ongoing proving process.
    Upon successful completion, it yields a `JobWitness` object, representing the serialized proof of correctness of the `JobTrace`.
    This proof can be subsequently sent to a verifier for zkSTARK proof verification.
*/

pub trait ProverController {
    fn run(
        &self,
        job_trace: JobTrace,
    ) -> Result<Process<Result<JobWitness, ProverControllerError>>, ProverControllerError>;
}
