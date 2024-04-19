use sharp_p2p_common::{job::Job, job_trace::JobTrace};

use crate::errors::RunnerControllerError;

pub trait Runner {
    fn init() -> impl RunnerController;
}

pub trait RunnerController {
    async fn run(&mut self, job: Job) -> Result<JobTrace, RunnerControllerError>;
    async fn terminate(&mut self, job_hash: u64) -> Result<(), RunnerControllerError>;
    async fn drop(self) -> Result<(), RunnerControllerError>;
}
