use self::types::{
    config::Config,
    params::{Fri, Params, Stark},
};
use crate::{errors::ProverControllerError, traits::ProverController};
use async_process::Stdio;
use futures::Future;
use serde_json::Value;
use std::{
    fs,
    hash::{DefaultHasher, Hash, Hasher},
    io::{Read, Write},
    pin::Pin,
};
use tempfile::NamedTempFile;
use tokio::{process::Command, select, sync::mpsc};
use tracing::debug;
use zetina_common::{hash, job_trace::JobTrace, job_witness::JobWitness, process::Process};

pub mod tests;
pub mod types;

pub struct StoneProver {}

impl StoneProver {
    pub fn new() -> Self {
        Self {}
    }
}

impl ProverController for StoneProver {
    fn run(
        &self,
        job_trace: JobTrace,
    ) -> Result<Process<Result<JobWitness, ProverControllerError>>, ProverControllerError> {
        let (terminate_tx, mut terminate_rx) = mpsc::channel::<()>(10);
        let future: Pin<
            Box<dyn Future<Output = Result<JobWitness, ProverControllerError>> + Send + '_>,
        > = Box::pin(async move {
            let mut out_file = NamedTempFile::new()?;

            let mut cpu_air_prover_config = NamedTempFile::new()?;
            let mut cpu_air_params = NamedTempFile::new()?;

            let n_steps: u64 = serde_json::from_str::<Value>(
                fs::read_to_string(job_trace.air_public_input.path())?.as_str(),
            )?["n_steps"]
                .as_u64()
                .ok_or(ProverControllerError::NumberOfStepsUnavailable)?;

            cpu_air_prover_config
                .write_all(&serde_json::to_string(&config(n_steps))?.into_bytes())?;
            cpu_air_params.write_all(&serde_json::to_string(&params(n_steps))?.into_bytes())?;

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

            let mut proof = Vec::new();
            out_file.read_to_end(&mut proof)?;

            Ok(JobWitness { job_hash: job_trace.job_hash, proof })
        });

        Ok(Process::new(future, terminate_tx))
    }
}

pub fn config(_n_steps: u64) -> Config {
    Config::default()
}

const fn num_bits<T>() -> usize {
    std::mem::size_of::<T>() * 8
}

fn log_2(x: u64) -> u64 {
    num_bits::<u64>() as u64 - x.leading_zeros() as u64 - 1
}

pub fn params(n_steps: u64) -> Params {
    // log₂(last_layer_degree_bound) + ∑fri_step_list = log₂(#steps) + 4
    // ∑fri_step_list = log₂(#steps) + 4 - log₂(last_layer_degree_bound)

    let last_layer_degree_bound = 128;
    let fri_step_list_sum = log_2(n_steps) + 4 - log_2(last_layer_degree_bound);
    Params {
        stark: Stark {
            fri: Fri {
                fri_step_list: std::iter::once(0)
                    .chain(
                        std::iter::repeat(4)
                            .take((fri_step_list_sum / 4) as usize)
                            .chain(std::iter::once(fri_step_list_sum % 4)),
                    )
                    .collect(),
                last_layer_degree_bound,
                n_queries: 1,
                proof_of_work_bits: 1,
            },
            log_n_cosets: 1,
        },
        ..Default::default()
    }
}

impl Default for StoneProver {
    fn default() -> Self {
        Self::new()
    }
}
