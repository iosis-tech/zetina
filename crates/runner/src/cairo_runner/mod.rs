use self::types::input::SimpleBootloaderInput;
use crate::{errors::RunnerControllerError, traits::RunnerController};
use async_process::Stdio;
use futures::Future;
use libp2p::identity::ecdsa::PublicKey;
use sharp_p2p_common::{hash, job::Job, job_trace::JobTrace, layout::Layout, process::Process};
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    pin::Pin,
};
use std::{io::Write, path::PathBuf};
use tempfile::NamedTempFile;
use tokio::{process::Command, select, sync::mpsc};
use tracing::debug;

pub mod tests;
pub mod types;

pub struct CairoRunner<'identity> {
    program_path: PathBuf,
    public_key: &'identity PublicKey,
}

impl<'identity> CairoRunner<'identity> {
    pub fn new(program_path: PathBuf, public_key: &'identity PublicKey) -> Self {
        Self { program_path, public_key }
    }
}

impl<'identity> RunnerController for CairoRunner<'identity> {
    fn run(
        &self,
        job: Job,
    ) -> Result<Process<Result<JobTrace, RunnerControllerError>>, RunnerControllerError> {
        let (terminate_tx, mut terminate_rx) = mpsc::channel::<()>(10);
        let future: Pin<Box<dyn Future<Output = Result<JobTrace, RunnerControllerError>> + '_>> =
            Box::pin(async move {
                let job_hash = hash!(job);
                let layout: &str = Layout::RecursiveWithPoseidon.into();

                let mut cairo_pie = NamedTempFile::new()?;
                cairo_pie.write_all(&job.job_data.cairo_pie_compressed)?;

                let input = SimpleBootloaderInput {
                    public_key: self.public_key.to_bytes(),
                    job,
                    single_page: true,
                };

                let mut program_input = NamedTempFile::new()?;
                program_input.write_all(&serde_json::to_string(&input)?.into_bytes())?;

                // outputs
                let air_public_input = NamedTempFile::new()?;
                let air_private_input = NamedTempFile::new()?;
                let trace = NamedTempFile::new()?;
                let memory = NamedTempFile::new()?;

                let mut task = Command::new("cairo-run")
                    .arg("--program")
                    .arg(self.program_path.as_path())
                    .arg("--layout")
                    .arg(layout)
                    .arg("--program_input")
                    .arg(program_input.path())
                    .arg("--air_public_input")
                    .arg(air_public_input.path())
                    .arg("--air_private_input")
                    .arg(air_private_input.path())
                    .arg("--trace_file")
                    .arg(trace.path())
                    .arg("--memory_file")
                    .arg(memory.path())
                    .arg("--proof_mode")
                    .arg("--print_output")
                    .stdout(Stdio::null())
                    .spawn()?;

                debug!("task {} spawned", job_hash);

                loop {
                    select! {
                        output = task.wait() => {
                            debug!("{:?}", output);
                            if !output?.success() {
                                return Err(RunnerControllerError::TaskTerminated);
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
                Ok(JobTrace { air_public_input, air_private_input, memory, trace })
            });

        Ok(Process::new(future, terminate_tx))
    }
}
