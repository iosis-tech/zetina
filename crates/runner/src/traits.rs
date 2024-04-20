use crate::errors::RunnerControllerError;
use sharp_p2p_common::{job::Job, process::Process};

pub trait RunnerController {
    type ProcessResult;
    fn run(&self, job: Job) -> Result<Process<Self::ProcessResult>, RunnerControllerError>;
}
