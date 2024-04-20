use self::types::{config::Config, params::Params};
use crate::{errors::ProverControllerError, traits::ProverController};
use async_process::Stdio;
use futures::Future;
use itertools::{chain, Itertools};
use sharp_p2p_common::{
    hash, job_trace::JobTrace, job_witness::JobWitness, process::Process, vec252::VecFelt252,
};
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    io::{Read, Write},
    pin::Pin,
};
use tempfile::NamedTempFile;
use tokio::{process::Command, select, sync::mpsc};
use tracing::debug;

pub mod tests;
pub mod types;

pub struct StoneProver {
    cpu_air_prover_config: Config,
    cpu_air_params: Params,
}

impl StoneProver {
    pub fn new(cpu_air_prover_config: Config, cpu_air_params: Params) -> Self {
        Self { cpu_air_prover_config, cpu_air_params }
    }
}

impl ProverController for StoneProver {
    type ProcessResult = Result<JobWitness, ProverControllerError>;
    fn run(
        &self,
        job_trace: JobTrace,
    ) -> Result<Process<Self::ProcessResult>, ProverControllerError> {
        let (terminate_tx, mut terminate_rx) = mpsc::channel::<()>(10);
        let future: Pin<Box<dyn Future<Output = Self::ProcessResult> + '_>> =
            Box::pin(async move {
                let mut out_file = NamedTempFile::new()?;

                let mut cpu_air_prover_config = NamedTempFile::new()?;
                let mut cpu_air_params = NamedTempFile::new()?;

                cpu_air_prover_config
                    .write_all(&serde_json::to_string(&self.cpu_air_prover_config)?.into_bytes())?;
                cpu_air_params
                    .write_all(&serde_json::to_string(&self.cpu_air_params)?.into_bytes())?;

                let mut task = Command::new("cpu_air_prover")
                    .arg("--out_file")
                    .arg(out_file.path())
                    .arg("--private_input_file")
                    .arg(job_trace.air_private_input.path())
                    .arg("--public_input_file")
                    .arg(job_trace.air_public_input.path())
                    .arg("--prover_config_file")
                    .arg(cpu_air_prover_config.path())
                    .arg("--parameter_file")
                    .arg(cpu_air_params.path())
                    .arg("--generate_annotations")
                    .stdout(Stdio::null())
                    .spawn()?;

                let job_trace_hash = hash!(job_trace);

                debug!("task {} spawned", job_trace_hash);

                loop {
                    select! {
                        output = task.wait() => {
                            debug!("{:?}", output);
                            if !output?.success() {
                                return Err(ProverControllerError::TaskTerminated);
                            }
                            let output = task.wait_with_output().await?;
                            debug!("{:?}", output);
                            break;
                        }
                        Some(()) = terminate_rx.recv() => {
                            task.start_kill()?;
                        }
                    }
                }

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
            });

        Ok(Process::new(future, terminate_tx))
    }
}
