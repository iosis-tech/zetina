use crate::errors::RunnerControllerError;
use zetina_common::{job::Job, job_trace::JobTrace, process::Process};

/*
    The `RunnerController` trait defines the responsibility for executing a `Job` within a Cairo bootloader environment.
    It ensures the validity of the `Job` object and embeds a witness of this validation in the program output.
    This process guarantees that the `Job` was not maliciously created by any party.
    The `run` method takes a `Job` as input and returns a `Result` containing a `Process` object.
    Upon successful execution, it produces a `JobTrace` object,
    encapsulating the execution trace of the `Job`, which can later be handled by the zkSTARK Prover.
    The bootloader runs the `Job` as a task in proof mode with a selected layout to facilitate the creation of the `JobTrace`.
*/

pub trait RunnerController {
    fn run(
        &self,
        job: Job,
    ) -> Result<Process<Result<JobTrace, RunnerControllerError>>, RunnerControllerError>;
}
