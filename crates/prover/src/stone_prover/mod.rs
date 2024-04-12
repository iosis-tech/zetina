use std::{collections::HashMap, io::Read};

use crate::{
    errors::ProverControllerError,
    traits::{Prover, ProverController},
};
use itertools::{chain, Itertools};
use sharp_p2p_common::{job::Job, job_witness::JobWitness, vec252::VecFelt252};
use std::io::Write;
use tempfile::NamedTempFile;
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
        let mut out_file = NamedTempFile::new()?;
        let mut private_input_file = NamedTempFile::new()?;
        let mut public_input_file = NamedTempFile::new()?;
        let mut prover_config_file = NamedTempFile::new()?;
        let mut parameter_file = NamedTempFile::new()?;

        private_input_file.write_all(&job.private_input)?;
        public_input_file.write_all(&job.public_input)?;
        prover_config_file.write_all(&job.cpu_air_prover_config)?;
        parameter_file.write_all(&job.cpu_air_params)?;
        trace!("task {} environment prepared", job);

        let task = Command::new("cpu_air_prover")
            .args(["out_file", out_file.path().to_string_lossy().as_ref()])
            .args(["private_input_file", private_input_file.path().to_string_lossy().as_ref()])
            .args(["public_input_file", public_input_file.path().to_string_lossy().as_ref()])
            .args(["prover_config_file", prover_config_file.path().to_string_lossy().as_ref()])
            .args(["parameter_file", parameter_file.path().to_string_lossy().as_ref()])
            .arg("--generate_annotations")
            .spawn()?;

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

        let mut input = String::new();
        out_file.read_to_string(&mut input)?;

        let parsed_proof = cairo_proof_parser::parse(input)
            .map_err(|e| ProverControllerError::ProofParseError(e.to_string()))?;

        let config: VecFelt252 = serde_json::from_str(&parsed_proof.config.to_string())?;
        let public_input: VecFelt252 =
            serde_json::from_str(&parsed_proof.public_input.to_string())?;
        let unsent_commitment: VecFelt252 =
            serde_json::from_str(&parsed_proof.unsent_commitment.to_string())?;
        let witness: VecFelt252 = serde_json::from_str(&parsed_proof.witness.to_string())?;

        let data = chain!(
            config.into_iter(),
            public_input.into_iter(),
            unsent_commitment.into_iter(),
            witness.into_iter()
        )
        .collect_vec();

        Ok(JobWitness { data })
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
