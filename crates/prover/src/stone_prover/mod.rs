use crate::{
    errors::ProverControllerError,
    traits::{Prover, ProverController},
};
use itertools::{chain, Itertools};
use sharp_p2p_common::{hash, job_trace::JobTrace, job_witness::JobWitness, vec252::VecFelt252};
use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    io::Read,
};
use tempfile::NamedTempFile;
use tokio::process::{Child, Command};
use tracing::{debug, trace};

pub struct StoneProver {
    tasks: HashMap<u64, Child>,
}

impl Prover for StoneProver {
    fn init() -> impl ProverController {
        Self { tasks: HashMap::new() }
    }
}

impl ProverController for StoneProver {
    async fn prove(&mut self, job_trace: JobTrace) -> Result<JobWitness, ProverControllerError> {
        let mut out_file = NamedTempFile::new()?;

        let task = Command::new("cpu_air_prover")
            .args(["--out_file", out_file.path().to_string_lossy().as_ref()])
            .args([
                "--air_private_input",
                job_trace.air_private_input.path().to_string_lossy().as_ref(),
            ])
            .args([
                "--air_public_input",
                job_trace.air_public_input.path().to_string_lossy().as_ref(),
            ])
            .args([
                "--cpu_air_prover_config",
                job_trace.cpu_air_prover_config.path().to_string_lossy().as_ref(),
            ])
            .args(["--cpu_air_params", job_trace.cpu_air_params.path().to_string_lossy().as_ref()])
            .arg("--generate_annotations")
            .spawn()?;

        let job_trace_hash = hash!(job_trace);

        debug!("task {} spawned", job_trace_hash);
        self.tasks.insert(job_trace_hash.to_owned(), task);

        let task_status = self
            .tasks
            .get_mut(&job_trace_hash)
            .ok_or(ProverControllerError::TaskNotFound)?
            .wait()
            .await?;

        trace!("task {} woke up", job_trace_hash);
        if !task_status.success() {
            debug!("task terminated {}", job_trace_hash);
            return Err(ProverControllerError::TaskTerminated);
        }

        let task_output = self
            .tasks
            .remove(&job_trace_hash)
            .ok_or(ProverControllerError::TaskNotFound)?
            .wait_with_output()
            .await?;
        trace!("task {} output {:?}", job_trace_hash, task_output);

        let mut raw_proof = String::new();
        out_file.read_to_string(&mut raw_proof)?;

        let parsed_proof = cairo_proof_parser::parse(raw_proof)
            .map_err(|e| ProverControllerError::ProofParseError(e.to_string()))?;

        let config: VecFelt252 = serde_json::from_str(&parsed_proof.config.to_string())?;
        let public_input: VecFelt252 =
            serde_json::from_str(&parsed_proof.public_input.to_string())?;
        let unsent_commitment: VecFelt252 =
            serde_json::from_str(&parsed_proof.unsent_commitment.to_string())?;
        let witness: VecFelt252 = serde_json::from_str(&parsed_proof.witness.to_string())?;

        let proof = chain!(
            config.into_iter(),
            public_input.into_iter(),
            unsent_commitment.into_iter(),
            witness.into_iter()
        )
        .collect_vec();

        Ok(JobWitness { proof })
    }

    fn terminate(&mut self, job_trace_hash: u64) -> Result<(), ProverControllerError> {
        self.tasks
            .get_mut(&job_trace_hash)
            .ok_or(ProverControllerError::TaskNotFound)?
            .start_kill()?;
        trace!("task scheduled for termination {}", job_trace_hash);
        Ok(())
    }

    fn drop(mut self) -> Result<(), ProverControllerError> {
        let keys: Vec<u64> = self.tasks.keys().cloned().collect();
        for job_trace_hash in keys.iter() {
            self.tasks
                .get_mut(job_trace_hash)
                .ok_or(ProverControllerError::TaskNotFound)?
                .start_kill()?;
            trace!("task scheduled for termination {}", job_trace_hash);
        }
        Ok(())
    }
}
