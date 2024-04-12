use std::collections::HashMap;

use crate::{
    errors::ProverControllerError,
    traits::{Prover, ProverController},
};
use sharp_p2p_common::{job::Job, job_witness::JobWitness};
use tokio::process::{Child, Command};
use tracing::{debug, trace};

pub struct StoneProver {
    tasks: HashMap<Job, Child>,
}

impl Prover for StoneProver {
    fn init() -> impl ProverController {
        Self { tasks: HashMap::new() }
    }
}

impl ProverController for StoneProver {
    async fn prove(&mut self, job: Job) -> Result<JobWitness, ProverControllerError> {
        let task = Command::new("sleep 20").spawn()?;
        debug!("task {} spawned", job);
        self.tasks.insert(job.to_owned(), task);

        let task_status =
            self.tasks.get_mut(&job).ok_or(ProverControllerError::TaskNotFound)?.wait().await?;

        trace!("task {} woke up", job);
        if !task_status.success() {
            debug!("task terminated {}", job);
            return Err(ProverControllerError::TaskTerminated);
        }

        let task_output = self
            .tasks
            .remove(&job)
            .ok_or(ProverControllerError::TaskNotFound)?
            .wait_with_output()
            .await?;
        trace!("task {} output {:?}", job, task_output);

        todo!()
    }

    async fn terminate(&mut self, job: &Job) -> Result<(), ProverControllerError> {
        self.tasks.get_mut(job).ok_or(ProverControllerError::TaskNotFound)?.start_kill()?;
        trace!("task scheduled for termination {}", job);
        Ok(())
    }

    async fn drop(mut self) -> Result<(), ProverControllerError> {
        let keys: Vec<Job> = self.tasks.keys().cloned().collect();
        for job in keys.iter() {
            self.terminate(job).await?;
        }
        Ok(())
    }
}
