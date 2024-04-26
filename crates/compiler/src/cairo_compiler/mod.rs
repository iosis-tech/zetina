use crate::{errors::CompilerControllerError, traits::CompilerController};
use async_process::Stdio;
use futures::Future;
use sharp_p2p_common::job::JobData;
use sharp_p2p_common::layout::Layout;
use sharp_p2p_common::{job::Job, process::Process};
use starknet_crypto::FieldElement;
use std::path::PathBuf;
use std::{io::Read, pin::Pin};
use tempfile::NamedTempFile;
use tokio::{process::Command, select, sync::mpsc};
use tracing::debug;

pub mod tests;

pub struct CairoCompiler {
    keypair: libp2p::identity::ecdsa::Keypair,
    registry_contract: FieldElement,
}

impl CairoCompiler {
    pub fn new(keypair: libp2p::identity::ecdsa::Keypair, registry_contract: FieldElement) -> Self {
        Self { keypair, registry_contract }
    }
}

impl CompilerController for CairoCompiler {
    fn run(
        &self,
        program_path: PathBuf,
        program_input_path: PathBuf,
    ) -> Result<Process<Result<Job, CompilerControllerError>>, CompilerControllerError> {
        let (terminate_tx, mut terminate_rx) = mpsc::channel::<()>(10);
        let future: Pin<Box<dyn Future<Output = Result<Job, CompilerControllerError>> + '_>> =
            Box::pin(async move {
                let layout: &str = Layout::RecursiveWithPoseidon.into();

                let output = NamedTempFile::new()?;

                let mut task = Command::new("cairo-compile")
                    .arg(program_path.as_path())
                    .arg("--output")
                    .arg(output.path())
                    .arg("--proof_mode")
                    .stdout(Stdio::null())
                    .spawn()?;

                debug!("program {:?} is compiling... ", program_path);

                loop {
                    select! {
                        output = task.wait() => {
                            debug!("{:?}", output);
                            if !output?.success() {
                                return Err(CompilerControllerError::TaskTerminated);
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

                // output
                let mut cairo_pie = NamedTempFile::new()?;

                let mut task = Command::new("cairo-run")
                    .arg("--program")
                    .arg(output.path())
                    .arg("--layout")
                    .arg(layout)
                    .arg("--program_input")
                    .arg(program_input_path.as_path())
                    .arg("--cairo_pie_output")
                    .arg(cairo_pie.path())
                    .arg("--print_output")
                    .stdout(Stdio::null())
                    .spawn()?;

                debug!("program {:?} is generating PIE... ", program_path);

                loop {
                    select! {
                        output = task.wait() => {
                            debug!("{:?}", output);
                            if !output?.success() {
                                return Err(CompilerControllerError::TaskTerminated);
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

                // cairo run had finished
                let mut cairo_pie_bytes = Vec::new();
                cairo_pie.read_to_end(&mut cairo_pie_bytes)?;

                Ok(Job::from_job_data(
                    JobData {
                        reward: 0,       // TODO: calculate this properly
                        num_of_steps: 0, // TODO: calculate this properly
                        cairo_pie_compressed: cairo_pie_bytes,
                        registry_address: self.registry_contract,
                    },
                    self.keypair.clone(),
                ))
            });

        Ok(Process::new(future, terminate_tx))
    }
}
